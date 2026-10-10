//! Machine load watch: on a timer, reads the 1-minute load average, normalises it by core
//! count, and while it is over `[machine] load_per_core` holds new agent spawns (like the
//! budget hold; a held spawn is refused, not queued). The orchestrator is messaged once per
//! crossing, naming the load and the top CPU consumers (ticket 58c9). Running agents are
//! left alone: this slice only stops adding work. Notes are rate-limited (ticket tnyt): after
//! one, the next needs a quiet stretch under the threshold, and a machine-wide stamp file in
//! the bridle home keeps several daemons on one machine from each sending their own.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bridle_api::types::{LoadStatus, MessageKind, When, event_kind};

use crate::supervisor::{AgentManager, ToTarget};

pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(30);
/// 70 on 16 cores (the incident behind the ticket) is 4.4; a busy but healthy build box
/// sits around 1 to 2.
pub const DEFAULT_LOAD_PER_CORE: f64 = 2.5;
/// After a note, the next waits until the load has stayed under the threshold this long...
pub const DEFAULT_QUIET_BELOW: Duration = Duration::from_secs(10 * 60);
/// ...and never comes sooner than this after the last note sent by any daemon on the machine.
pub const DEFAULT_NOTE_GAP: Duration = Duration::from_secs(30 * 60);
const STAMP_FILE: &str = "load-note.stamp";

/// What a sample needs from the machine; tests inject a fake.
pub trait LoadSource: Send + Sync {
    /// 1-minute load average and the core count.
    fn sample(&self) -> Option<(f64, usize)>;
    /// The top CPU consumers as a short human string, or empty if unknown.
    fn top_consumers(&self) -> String;
}

/// Reads `/proc/loadavg` (Linux) or `sysctl vm.loadavg` (macOS): no libc call, so no unsafe.
pub struct SystemLoad;

impl LoadSource for SystemLoad {
    fn sample(&self) -> Option<(f64, usize)> {
        let raw = std::fs::read_to_string("/proc/loadavg").ok().or_else(|| {
            let out = std::process::Command::new("sysctl")
                .args(["-n", "vm.loadavg"])
                .output()
                .ok()?;
            String::from_utf8(out.stdout).ok()
        })?;
        let cores = std::thread::available_parallelism().ok()?.get();
        Some((parse_load1(&raw)?, cores))
    }

    fn top_consumers(&self) -> String {
        let Ok(out) = std::process::Command::new("ps")
            .args(["-A", "-o", "pcpu=,comm="])
            .output()
        else {
            return String::new();
        };
        top_from_ps(&String::from_utf8_lossy(&out.stdout), 3)
    }
}

/// The first number of `0.52 0.58 0.59 1/467 8223` or `{ 3.17 2.90 2.85 }`.
fn parse_load1(raw: &str) -> Option<f64> {
    raw.split_whitespace()
        .find_map(|w| w.trim_matches(['{', '}']).parse::<f64>().ok())
}

/// `ps -o pcpu=,comm=` lines folded by command name, busiest first, as `name 120%, ...`.
fn top_from_ps(ps: &str, n: usize) -> String {
    let mut by_name: Vec<(String, f64)> = Vec::new();
    for line in ps.lines() {
        let Some((cpu, comm)) = line.trim().split_once(char::is_whitespace) else {
            continue;
        };
        let (Ok(cpu), name) = (
            cpu.parse::<f64>(),
            comm.trim().rsplit('/').next().unwrap_or("").to_string(),
        ) else {
            continue;
        };
        match by_name.iter_mut().find(|(k, _)| *k == name) {
            Some(e) => e.1 += cpu,
            None => by_name.push((name, cpu)),
        }
    }
    by_name.sort_by(|a, b| b.1.total_cmp(&a.1));
    by_name
        .iter()
        .take(n)
        .map(|(k, v)| format!("{k} {v:.0}%"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Clone)]
pub struct LoadWatch(Arc<Inner>);

struct Inner {
    threshold: f64,
    source: Box<dyn LoadSource>,
    manager: AgentManager,
    /// Whether the last tick was over the threshold, so the orchestrator is told once per
    /// crossing, not every tick the load stays high.
    holding: Mutex<bool>,
    quiet_below: Duration,
    note_gap: Duration,
    /// Where the last-sent time lives, shared by every daemon on the machine.
    stamp: PathBuf,
    rate: Mutex<Rate>,
    hold: Mutex<Hold>,
}

/// Hold history for the `load.hold.*` events and the long-hold escalation.
#[derive(Default)]
struct Hold {
    started: Option<SystemTime>,
    /// Finished holds inside the escalation window.
    done: Vec<(SystemTime, SystemTime)>,
    last_escalation: Option<SystemTime>,
}

/// Escalate when spawns were held longer than this in the last `ESCALATE_WINDOW`...
const ESCALATE_HELD: Duration = Duration::from_secs(60 * 60);
const ESCALATE_WINDOW: Duration = Duration::from_secs(2 * 60 * 60);
/// ...and no more than once per this.
const ESCALATE_EVERY: Duration = Duration::from_secs(60 * 60);

impl Hold {
    /// Seconds held inside `[now - window, now]`, counting an open hold up to `now`.
    fn held_within(&self, now: SystemTime) -> Duration {
        let from = now - ESCALATE_WINDOW;
        self.done
            .iter()
            .copied()
            .chain(self.started.map(|s| (s, now)))
            .map(|(s, e)| e.duration_since(s.max(from)).unwrap_or_default())
            .sum()
    }
}

/// Command names of bridle's own processes: the cause of a load hold may be bridle itself
/// (ticket n4w4: test daemons polling `ps`).
fn bridle_owned(name: &str) -> bool {
    // `top_from_ps` keeps only the basename, so a test binary under a worktree's target/debug
    // shows as `bridle_daemon-<hash>`.
    matches!(name, "ps" | "fake-claude" | "claude") || name.starts_with("bridle")
}

/// The line naming a bridle-owned process among the first three of `top` (`name 120%, ...`).
fn bridle_process_line(top: &str) -> Option<String> {
    top.split(", ").take(3).find_map(|c| {
        let (name, pct) = c.rsplit_once(' ')?;
        bridle_owned(name).then(|| {
            format!(
                " A bridle process is a top consumer: {name} {pct}. Find the cause now, do not wait."
            )
        })
    })
}

struct Rate {
    /// A note may be sent; cleared by a note (ours or another daemon's), restored once the
    /// load has been under the threshold for `quiet_below`.
    armed: bool,
    below_since: Option<SystemTime>,
}

/// Seconds since the epoch in the stamp, or None if missing or unreadable.
fn read_stamp(path: &std::path::Path) -> Option<SystemTime> {
    let secs: u64 = std::fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(secs))
}

fn write_stamp(path: &std::path::Path, now: SystemTime) {
    let secs = now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let tmp = path.with_extension("stamp.tmp");
    if let Err(e) = std::fs::write(&tmp, secs.to_string()).and_then(|_| std::fs::rename(&tmp, path))
    {
        tracing::warn!("load note stamp not written: {e}");
    }
}

impl LoadWatch {
    pub fn new(
        threshold: f64,
        quiet_below: Duration,
        note_gap: Duration,
        home: PathBuf,
        source: Box<dyn LoadSource>,
        manager: AgentManager,
    ) -> Self {
        LoadWatch(Arc::new(Inner {
            threshold,
            source,
            manager,
            holding: Mutex::new(false),
            quiet_below,
            note_gap,
            stamp: home.join(STAMP_FILE),
            rate: Mutex::new(Rate {
                armed: true,
                below_since: None,
            }),
            hold: Mutex::new(Hold::default()),
        }))
    }

    pub async fn tick(&self) {
        self.tick_at(SystemTime::now()).await
    }

    /// One sample, with the clock passed in so tests can move it.
    async fn tick_at(&self, now: SystemTime) {
        let this = self.clone();
        let sampled = tokio::task::spawn_blocking(move || this.0.source.sample()).await;
        let Ok(Some((load1, cores))) = sampled else {
            return tracing::warn!("load sample failed");
        };
        let per_core = load1 / cores.max(1) as f64;
        // A threshold of zero or less turns the hold off but keeps `bridle status` fed.
        let over = self.0.threshold > 0.0 && per_core > self.0.threshold;
        self.0.manager.set_load(LoadStatus {
            load1,
            cores: cores as u32,
            per_core,
            threshold: self.0.threshold,
            holding: over,
        });
        *self.0.holding.lock().expect("load lock") = over;
        let note = self.should_note(over, now);
        let (started, ended) = self.track_hold(over, now);
        let mut top = String::new();
        if note || started {
            let this = self.clone();
            top = tokio::task::spawn_blocking(move || this.0.source.top_consumers())
                .await
                .unwrap_or_default();
        }
        if started {
            self.0
                .manager
                .emit_system_event(
                    event_kind::LOAD_HOLD_STARTED,
                    serde_json::json!({
                        "load1": load1, "cores": cores, "per_core": per_core,
                        "threshold": self.0.threshold, "consumers": top,
                    }),
                )
                .await;
        }
        if let Some(held) = ended {
            self.0
                .manager
                .emit_system_event(
                    event_kind::LOAD_HOLD_ENDED,
                    serde_json::json!({ "held_secs": held.as_secs() }),
                )
                .await;
        }
        self.escalate_if_long(now).await;
        if !note {
            return;
        }
        let mut body = format!(
            "Machine load is high: {load1:.1} on {cores} cores ({per_core:.1} per core, \
             threshold {:.1}). New agent spawns are refused until it falls (they are \
             not queued); retry then. Don't add work: wait.",
            self.0.threshold
        );
        if !top.is_empty() {
            body.push_str(&format!(" Top consumers: {top}."));
        }
        if let Some(line) = bridle_process_line(&top) {
            body.push_str(&line);
        }
        let _ = self
            .0
            .manager
            .send(
                "system".to_string(),
                ToTarget::External(crate::wake::ORCHESTRATOR.to_string()),
                MessageKind::Note,
                body,
                When::Now,
                None,
            )
            .await;
    }
}

impl LoadWatch {
    /// Updates the hold history; returns (a hold started, the length of one that ended).
    fn track_hold(&self, over: bool, now: SystemTime) -> (bool, Option<Duration>) {
        let mut h = self.0.hold.lock().expect("load hold lock");
        h.done
            .retain(|(_, e)| now.duration_since(*e).unwrap_or_default() < ESCALATE_WINDOW);
        match (over, h.started) {
            (true, None) => {
                h.started = Some(now);
                (true, None)
            }
            (false, Some(s)) => {
                h.started = None;
                h.done.push((s, now));
                (false, Some(now.duration_since(s).unwrap_or_default()))
            }
            _ => (false, None),
        }
    }

    /// One message to the orchestrator, and a note for the human's morning list, when spawns
    /// have been held over an hour of the last two (at most once an hour). A refused spawn of
    /// a critical task is not a trigger: spawn requests carry no task.
    async fn escalate_if_long(&self, now: SystemTime) {
        let held = {
            let mut h = self.0.hold.lock().expect("load hold lock");
            let held = h.held_within(now);
            let recent = h
                .last_escalation
                .is_some_and(|t| now.duration_since(t).unwrap_or_default() < ESCALATE_EVERY);
            if held <= ESCALATE_HELD || recent {
                return;
            }
            h.last_escalation = Some(now);
            held
        };
        let body = format!(
            "Spawns have been held {} minutes of the last {}: investigate the cause now",
            held.as_secs() / 60,
            ESCALATE_WINDOW.as_secs() / 60
        );
        let manager = &self.0.manager;
        let to_orch = ToTarget::External(crate::wake::ORCHESTRATOR.to_string());
        for to in [to_orch, ToTarget::Human] {
            let _ = manager
                .send(
                    "system".to_string(),
                    to,
                    MessageKind::Note,
                    body.clone(),
                    When::Now,
                    None,
                )
                .await;
        }
    }

    /// Whether this tick sends a note: over the threshold, armed, and no daemon on the machine
    /// has sent one inside the gap. A skip because of the stamp still uses up the arming: the
    /// other daemon's note covers this crossing.
    fn should_note(&self, over: bool, now: SystemTime) -> bool {
        let mut rate = self.0.rate.lock().expect("load rate lock");
        if !over {
            let since = *rate.below_since.get_or_insert(now);
            if now.duration_since(since).unwrap_or_default() >= self.0.quiet_below {
                rate.armed = true;
            }
            return false;
        }
        rate.below_since = None;
        if !rate.armed {
            return false;
        }
        rate.armed = false;
        if let Some(last) = read_stamp(&self.0.stamp)
            && now
                .duration_since(last)
                .is_ok_and(|age| age < self.0.note_gap)
        {
            return false;
        }
        write_stamp(&self.0.stamp, now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::events::Emitter;
    use crate::paths::Workspace;
    use crate::store::{ListMessages, Store};
    use std::collections::VecDeque;

    /// Hands out the scripted loads one per tick.
    struct Fake(Mutex<VecDeque<f64>>, &'static str);

    impl LoadSource for Fake {
        fn sample(&self) -> Option<(f64, usize)> {
            Some((self.0.lock().unwrap().pop_front()?, 4))
        }
        fn top_consumers(&self) -> String {
            self.1.to_string()
        }
    }

    const MIN: Duration = Duration::from_secs(60);

    fn at(mins: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_000_000) + MIN * mins as u32
    }

    async fn fixture(loads: &[f64]) -> (LoadWatch, AgentManager, Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("bridle.db")).await.unwrap();
        let manager = AgentManager::new(
            store.clone(),
            Workspace::new(dir.path().join("repo"), None),
            Config::default(),
            "claude".to_string(),
            "http://127.0.0.1:0".to_string(),
            "test".to_string(),
            Emitter::new(store.clone()),
            Default::default(),
        );
        let fake = Fake(Mutex::new(loads.iter().copied().collect()), "rustc 300%");
        let watch = watch_in(dir.path(), fake, &manager);
        (watch, manager, store, dir)
    }

    /// Quiet period 10 minutes, gap 30; `home` holds the machine stamp.
    fn watch_in(home: &std::path::Path, fake: Fake, manager: &AgentManager) -> LoadWatch {
        LoadWatch::new(
            2.0,
            10 * MIN,
            30 * MIN,
            home.to_path_buf(),
            Box::new(fake),
            manager.clone(),
        )
    }

    async fn orchestrator_messages(store: &Store) -> Vec<String> {
        store
            .list_messages(ListMessages {
                to: Some(crate::wake::ORCHESTRATOR.to_string()),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.body)
            .collect()
    }

    #[tokio::test]
    async fn holds_over_threshold_releases_when_it_falls_and_tells_once_per_crossing() {
        // 4 cores, threshold 2.0 per core: 8.0 is the line (over means above).
        let (watch, manager, store, _dir) = fixture(&[1.0, 12.0, 20.0, 4.0, 1.0, 16.0]).await;
        watch.tick_at(at(0)).await;
        assert!(!manager.load_status().unwrap().holding);
        assert!(manager.refuse_if_load_held().is_ok());

        watch.tick_at(at(1)).await;
        let l = manager.load_status().unwrap();
        assert!(l.holding && l.per_core == 3.0 && l.cores == 4);
        assert!(manager.refuse_if_load_held().is_err());
        watch.tick_at(at(2)).await; // still over: no second message
        let msgs = orchestrator_messages(&store).await;
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("12.0 on 4 cores") && msgs[0].contains("rustc 300%"));

        watch.tick_at(at(3)).await; // falls: spawns resume
        assert!(manager.refuse_if_load_held().is_ok());
        watch.tick_at(at(45)).await; // quiet long enough
        watch.tick_at(at(46)).await; // a new crossing, past the gap: a second message
        assert_eq!(orchestrator_messages(&store).await.len(), 2);
    }

    #[tokio::test]
    async fn flapping_sends_one_note_until_quiet_below_or_gap() {
        // Over, under, over, under, over inside 5 minutes: one note.
        let (watch, _m, store, _dir) = fixture(&[12.0, 1.0, 12.0, 1.0, 12.0, 1.0, 1.0, 12.0]).await;
        for m in 0..5 {
            watch.tick_at(at(m)).await;
        }
        assert_eq!(orchestrator_messages(&store).await.len(), 1);
        // Under for 10 minutes re-arms; the stamp is 35 minutes old by the next crossing.
        watch.tick_at(at(10)).await;
        watch.tick_at(at(21)).await;
        watch.tick_at(at(35)).await; // over, armed, but the gap (30m from 0) has passed
        assert_eq!(orchestrator_messages(&store).await.len(), 2);
    }

    #[tokio::test]
    async fn crossing_inside_the_gap_is_held_back_even_when_quiet() {
        // Quiet for 10 minutes by minute 11, but the last note was at minute 0: gap not over.
        let (watch, _m, store, _dir) = fixture(&[12.0, 1.0, 1.0, 12.0]).await;
        watch.tick_at(at(0)).await;
        watch.tick_at(at(1)).await;
        watch.tick_at(at(12)).await;
        watch.tick_at(at(13)).await;
        assert_eq!(orchestrator_messages(&store).await.len(), 1);
    }

    #[tokio::test]
    async fn second_daemon_on_the_machine_skips_inside_the_gap() {
        let (first, manager, store, dir) = fixture(&[12.0, 12.0]).await;
        let second = watch_in(
            dir.path(),
            Fake(Mutex::new(VecDeque::from([12.0, 12.0])), "rustc 300%"),
            &manager,
        );
        first.tick_at(at(0)).await;
        second.tick_at(at(1)).await; // same home, inside the gap: skipped
        assert_eq!(orchestrator_messages(&store).await.len(), 1);

        // Re-armed after a quiet stretch and past the gap, the second may send.
        let late = watch_in(
            dir.path(),
            Fake(Mutex::new(VecDeque::from([12.0])), "rustc 300%"),
            &manager,
        );
        late.tick_at(at(31)).await;
        assert_eq!(orchestrator_messages(&store).await.len(), 2);
    }

    #[test]
    fn missing_or_garbled_stamp_means_send() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(STAMP_FILE);
        assert!(read_stamp(&p).is_none());
        std::fs::write(&p, "junk").unwrap();
        assert!(read_stamp(&p).is_none());
        write_stamp(&p, at(0));
        assert_eq!(read_stamp(&p), Some(at(0)));
    }

    #[test]
    fn parses_linux_and_macos_forms() {
        assert_eq!(parse_load1("0.52 0.58 0.59 1/467 8223\n"), Some(0.52));
        assert_eq!(parse_load1("{ 3.17 2.90 2.85 }\n"), Some(3.17));
        assert_eq!(parse_load1("junk"), None);
    }

    #[test]
    fn top_consumers_fold_by_name() {
        let ps = " 80.0 /usr/sbin/syspolicyd\n 30.0 /x/rustc\n 40.0 /y/rustc\n  1.0 /bin/zsh\n";
        assert_eq!(top_from_ps(ps, 2), "syspolicyd 80%, rustc 70%");
    }

    #[tokio::test]
    async fn note_does_not_claim_a_resume_and_names_bridle_processes() {
        let (watch, _m, store, dir) = fixture(&[12.0]).await;
        watch.tick_at(at(0)).await;
        let plain = orchestrator_messages(&store).await.remove(0);
        assert!(!plain.contains("resumes them itself"));
        assert!(plain.contains("refused until it falls") && plain.contains("retry then"));
        assert!(!plain.contains("bridle process"));

        let (w2, _m2, store2, _d2) = fixture(&[]).await;
        let w2 = watch_in(
            dir.path().join("other").as_path(),
            Fake(Mutex::new([12.0].into()), "rustc 300%, ps 90%, zsh 1%"),
            &w2.0.manager,
        );
        w2.tick_at(at(0)).await;
        let msg = orchestrator_messages(&store2).await.remove(0);
        assert!(msg.contains("A bridle process is a top consumer: ps 90%. Find the cause now"));
    }

    #[tokio::test]
    async fn hold_events_and_escalation_after_an_hour_held() {
        let (watch, _m, store, _dir) = fixture(&[12.0; 70]).await;
        for m in 0..=61 {
            watch.tick_at(at(m)).await;
        }
        // 61 minutes held: just over the hour. One escalation to orchestrator and human.
        let msgs = orchestrator_messages(&store).await;
        let esc: Vec<_> = msgs
            .iter()
            .filter(|b| b.starts_with("Spawns have been held"))
            .collect();
        assert_eq!(esc.len(), 1, "{msgs:?}");
        assert!(esc[0].contains("61 minutes of the last 120"));
        watch.tick_at(at(100)).await; // within the hour of the last escalation: no second
        assert_eq!(
            orchestrator_messages(&store)
                .await
                .iter()
                .filter(|b| b.starts_with("Spawns have been held"))
                .count(),
            1
        );
        let kinds: Vec<_> = store
            .list_events(Default::default())
            .await
            .unwrap()
            .into_iter()
            .map(|e| e.kind)
            .collect();
        assert_eq!(
            kinds.iter().filter(|k| *k == "load.hold.started").count(),
            1
        );
    }

    #[tokio::test]
    async fn hold_ended_event_carries_the_held_seconds() {
        let (watch, _m, store, _dir) = fixture(&[12.0, 12.0, 1.0]).await;
        for m in 0..3 {
            watch.tick_at(at(m * 5)).await;
        }
        let evs = store.list_events(Default::default()).await.unwrap();
        let ended = evs.iter().find(|e| e.kind == "load.hold.ended").unwrap();
        assert_eq!(ended.data["held_secs"], 600);
        let started = evs.iter().find(|e| e.kind == "load.hold.started").unwrap();
        assert_eq!(started.data["consumers"], "rustc 300%");
    }
}
