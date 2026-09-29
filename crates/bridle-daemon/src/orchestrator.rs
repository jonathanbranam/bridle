//! The orchestrator supervisor: keeps the human's interactive `claude` running in its tmux
//! pane by relaunching `scripts/claude-orchestrator` when it dies, with crash-loop backoff
//! (docs/design/agent-host/orchestrator-supervision.md, sections 3 and 4; spike 07).
//!
//! The pane is the human's: the supervisor types into it only when the recorded process is
//! dead and the pane's foreground command is a shell on two checks 5 s apart. tmux, process
//! liveness and the incident sink sit behind traits so tests drive a fake clock and fakes.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use bridle_api::types::{MessageKind, When, event_kind};
use chrono::{DateTime, Utc};
use serde_json::json;
use tokio::sync::Mutex;

use crate::config::OrchestratorConfig;
use crate::events::Emitter;
use crate::supervisor::{AgentManager, ToTarget};

pub const TICK_INTERVAL: Duration = Duration::from_secs(10);
/// The tag the human sets on the orchestrator's pane: `tmux set -p @bridle orchestrator`.
const PANE_TAG: &str = "orchestrator";
const SHELLS: &[&str] = &["zsh", "bash", "sh", "fish"];
/// The pane must show a shell on two checks at least this far apart.
const SHELL_RECHECK: Duration = Duration::from_secs(5);
/// A launch is confirmed when the pid file changes to a live pid within this long.
const CONFIRM_WITHIN: Duration = Duration::from_secs(60);

pub trait Tmux: Send + Sync {
    /// `(pane id, @bridle tag)` for every pane; the tag is empty when unset.
    async fn panes(&self) -> Result<Vec<(String, String)>, String>;
    async fn pane_command(&self, pane: &str) -> Result<String, String>;
    /// Types `text` literally, then a separate Enter (spike 07 #7).
    async fn type_line(&self, pane: &str, text: &str) -> Result<(), String>;
}

pub trait Procs: Send + Sync {
    /// The pid exists and its start time matches: a reused pid is not the session.
    async fn is_alive(&self, pid: i32, start: &str) -> bool;
}

/// The interim "record an incident" (section 8): one function nc7r replaces.
pub trait Incidents: Send + Sync {
    async fn record(&self, text: &str);
}

/// `<pid> <process start time> <launch epoch>`; the start time has spaces (`ps lstart`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PidRecord {
    pid: i32,
    start: String,
    launched: i64,
}

fn parse_pid_file(text: &str) -> Option<PidRecord> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    Some(PidRecord {
        pid: parts[0].parse().ok()?,
        launched: parts[parts.len() - 1].parse().ok()?,
        start: parts[1..parts.len() - 1].join(" "),
    })
}

#[derive(Default)]
struct State {
    warned_no_file: bool,
    /// Relaunches since the session was last stable.
    attempts: u32,
    last_attempt: Option<DateTime<Utc>>,
    /// The pid file as it was when we typed, and when: confirmation is it changing to a live pid.
    pending: Option<DateTime<Utc>>,
    given_up: bool,
    /// When the pane first showed a shell for the current relaunch.
    shell_seen: Option<DateTime<Utc>>,
    /// The death of this session has been reported.
    dead_noted: bool,
    /// The last incident text, so a condition that persists across ticks is reported once.
    last_incident: Option<String>,
}

pub struct Supervisor<T, P, I> {
    home: PathBuf,
    launcher: String,
    backoff: Vec<Duration>,
    stable_after: Duration,
    tmux: T,
    procs: P,
    incidents: I,
    state: Mutex<State>,
}

impl<T: Tmux, P: Procs, I: Incidents> Supervisor<T, P, I> {
    /// `launcher` is the absolute path to type into the pane.
    pub fn new(
        home: PathBuf,
        launcher: String,
        cfg: &OrchestratorConfig,
        tmux: T,
        procs: P,
        incidents: I,
    ) -> Self {
        Supervisor {
            home,
            launcher,
            backoff: cfg.relaunch_backoff.clone(),
            stable_after: cfg.stable_after,
            tmux,
            procs,
            incidents,
            state: Mutex::new(State::default()),
        }
    }

    pub async fn tick(&self, now: DateTime<Utc>) {
        let mut st = self.state.lock().await;
        let Some(rec) = std::fs::read_to_string(self.home.join("orchestrator.pid"))
            .ok()
            .and_then(|t| parse_pid_file(&t))
        else {
            // Never launch what the supervisor hasn't seen run.
            if !std::mem::replace(&mut st.warned_no_file, true) {
                tracing::info!("no orchestrator.pid: the orchestrator supervisor does nothing");
            }
            return;
        };
        st.warned_no_file = false;

        if self.procs.is_alive(rec.pid, &rec.start).await {
            let up = now.timestamp() - rec.launched;
            if up >= self.stable_after.as_secs() as i64 {
                st.attempts = 0;
            }
            st.given_up = false;
            st.pending = None;
            st.shell_seen = None;
            st.dead_noted = false;
            st.last_incident = None;
            return;
        }

        if !std::mem::replace(&mut st.dead_noted, true) {
            let text = format!(
                "The orchestrator session (pid {}) is not running; found dead at {}. {}",
                rec.pid,
                now.format("%H:%M:%S UTC"),
                self.transcript_activity()
            );
            self.report(&mut st, text).await;
        }
        if let Some(at) = st.pending {
            if now - at < chrono_dur(CONFIRM_WITHIN) {
                return;
            }
            // Never confirmed (the trust prompt, a broken launcher): a failed relaunch.
            st.pending = None;
        }
        if st.given_up {
            return;
        }
        if st.attempts as usize > self.backoff.len() {
            st.given_up = true;
            let text = format!(
                "The orchestrator is not staying up: {} relaunches, last attempt {}. Not trying again until it is running.",
                st.attempts,
                st.last_attempt
                    .map(|t| t.format("%H:%M:%S UTC").to_string())
                    .unwrap_or_default()
            );
            self.report(&mut st, text).await;
            return;
        }
        // The first relaunch goes at once, then `backoff[0]`, `backoff[1]`, ...
        if let (Some(last), Some(wait)) = (
            st.last_attempt,
            st.attempts.checked_sub(1).map(|i| self.backoff[i as usize]),
        ) && now - last < chrono_dur(wait)
        {
            return;
        }

        let pane = match self.tmux.panes().await {
            Err(e) => {
                return self
                    .report(
                        &mut st,
                        format!("The orchestrator is down and tmux is unavailable: {e}"),
                    )
                    .await;
            }
            Ok(panes) => panes.into_iter().find(|(_, tag)| tag == PANE_TAG),
        };
        let Some((pane, _)) = pane else {
            let text = "The orchestrator is down and no pane is tagged @bridle=orchestrator (`tmux set -p @bridle orchestrator` in its pane).".to_string();
            return self.report(&mut st, text).await;
        };
        match self.tmux.pane_command(&pane).await {
            Err(e) => {
                return self
                    .report(
                        &mut st,
                        format!("The orchestrator is down and tmux failed: {e}"),
                    )
                    .await;
            }
            Ok(cmd) if !SHELLS.contains(&cmd.trim()) => {
                st.shell_seen = None;
                let text = format!(
                    "The orchestrator is down but pane {pane} is running {:?}, not a shell; not typing into it.",
                    cmd.trim()
                );
                return self.report(&mut st, text).await;
            }
            Ok(_) => {}
        }
        match st.shell_seen {
            None => {
                st.shell_seen = Some(now);
                return;
            }
            Some(t) if now - t < chrono_dur(SHELL_RECHECK) => return,
            Some(_) => {}
        }

        st.attempts += 1;
        st.last_attempt = Some(now);
        st.shell_seen = None;
        tracing::info!(pane = %pane, attempt = st.attempts, "relaunching the orchestrator");
        match self.tmux.type_line(&pane, &self.launcher).await {
            Ok(()) => st.pending = Some(now),
            Err(e) => {
                self.report(&mut st, format!("Relaunching the orchestrator failed: {e}"))
                    .await
            }
        }
    }

    async fn report(&self, st: &mut State, text: String) {
        if st.last_incident.as_deref() == Some(&text) {
            return;
        }
        self.incidents.record(&text).await;
        st.last_incident = Some(text);
    }

    /// When the transcript was last written, from `orchestrator.session` (`<id> <path>`).
    fn transcript_activity(&self) -> String {
        let path = std::fs::read_to_string(self.home.join("orchestrator.session"))
            .ok()
            .and_then(|t| t.split_once(' ').map(|(_, p)| p.trim().to_string()))
            .filter(|p| !p.is_empty());
        let mtime = path
            .as_deref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok());
        match mtime {
            Some(t) => format!(
                "Its transcript was last written {}.",
                DateTime::<Utc>::from(t).format("%H:%M:%S UTC")
            ),
            None => "Its transcript's last activity is unknown.".to_string(),
        }
    }
}

fn chrono_dur(d: Duration) -> chrono::Duration {
    chrono::Duration::from_std(d).unwrap_or(chrono::Duration::MAX)
}

/// The launcher as typed into the shell: the path itself unless it needs quoting.
pub fn shell_word(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "_./-+@:".contains(c))
    {
        s.into_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

pub struct RealTmux;

impl RealTmux {
    async fn run(args: &[&str]) -> Result<String, String> {
        let out = tokio::process::Command::new("tmux")
            .args(args)
            .output()
            .await
            .map_err(|e| format!("running tmux: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "tmux {} exited {}: {}",
                args[0],
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

impl Tmux for RealTmux {
    async fn panes(&self) -> Result<Vec<(String, String)>, String> {
        let out = Self::run(&["list-panes", "-a", "-F", "#{pane_id} #{@bridle}"]).await?;
        Ok(out
            .lines()
            .map(|l| match l.split_once(' ') {
                Some((id, tag)) => (id.to_string(), tag.trim().to_string()),
                None => (l.to_string(), String::new()),
            })
            .collect())
    }

    async fn pane_command(&self, pane: &str) -> Result<String, String> {
        Self::run(&[
            "display-message",
            "-p",
            "-t",
            pane,
            "#{pane_current_command}",
        ])
        .await
        .map(|s| s.trim().to_string())
    }

    async fn type_line(&self, pane: &str, text: &str) -> Result<(), String> {
        Self::run(&["send-keys", "-t", pane, "-l", text]).await?;
        Self::run(&["send-keys", "-t", pane, "Enter"]).await?;
        Ok(())
    }
}

pub struct RealProcs;

impl Procs for RealProcs {
    async fn is_alive(&self, pid: i32, start: &str) -> bool {
        let start = start.to_string();
        tokio::task::spawn_blocking(move || crate::containment::is_same_process(pid, &start))
            .await
            .unwrap_or(false)
    }
}

/// A `system` note to the human plus an `orchestrator.incident` event.
pub struct RealIncidents {
    pub manager: AgentManager,
    pub emitter: Emitter,
}

impl Incidents for RealIncidents {
    async fn record(&self, text: &str) {
        tracing::warn!(incident = %text, "orchestrator incident");
        let _ = self
            .emitter
            .emit(
                event_kind::ORCHESTRATOR_INCIDENT,
                "system".to_string(),
                None,
                json!({ "text": text }),
            )
            .await;
        let _ = self
            .manager
            .send(
                "system".to_string(),
                ToTarget::Human,
                MessageKind::Note,
                text.to_string(),
                When::Now,
                None,
            )
            .await;
    }
}

pub type RealSupervisor = Supervisor<RealTmux, RealProcs, RealIncidents>;

pub fn real(
    home: PathBuf,
    launcher: String,
    cfg: &OrchestratorConfig,
    manager: AgentManager,
    emitter: Emitter,
) -> Arc<RealSupervisor> {
    Arc::new(Supervisor::new(
        home,
        launcher,
        cfg,
        RealTmux,
        RealProcs,
        RealIncidents { manager, emitter },
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex as StdMutex;

    use super::*;

    #[derive(Default)]
    struct FakeTmux {
        panes: StdMutex<Vec<(String, String)>>,
        command: StdMutex<String>,
        typed: StdMutex<Vec<(String, String)>>,
    }
    impl Tmux for Arc<FakeTmux> {
        async fn panes(&self) -> Result<Vec<(String, String)>, String> {
            Ok(self.panes.lock().unwrap().clone())
        }
        async fn pane_command(&self, _pane: &str) -> Result<String, String> {
            Ok(self.command.lock().unwrap().clone())
        }
        async fn type_line(&self, pane: &str, text: &str) -> Result<(), String> {
            self.typed.lock().unwrap().push((pane.into(), text.into()));
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeProcs(StdMutex<HashSet<i32>>);
    impl Procs for Arc<FakeProcs> {
        async fn is_alive(&self, pid: i32, _start: &str) -> bool {
            self.0.lock().unwrap().contains(&pid)
        }
    }

    #[derive(Default)]
    struct FakeIncidents(StdMutex<Vec<String>>);
    impl Incidents for Arc<FakeIncidents> {
        async fn record(&self, text: &str) {
            self.0.lock().unwrap().push(text.into());
        }
    }

    struct Rig {
        dir: tempfile::TempDir,
        tmux: Arc<FakeTmux>,
        procs: Arc<FakeProcs>,
        incidents: Arc<FakeIncidents>,
        sup: Supervisor<Arc<FakeTmux>, Arc<FakeProcs>, Arc<FakeIncidents>>,
        t0: DateTime<Utc>,
    }

    impl Rig {
        fn new() -> Rig {
            let dir = tempfile::tempdir().unwrap();
            let tmux = Arc::new(FakeTmux::default());
            *tmux.panes.lock().unwrap() = vec![
                ("%1".into(), String::new()),
                ("%2".into(), "orchestrator".into()),
            ];
            *tmux.command.lock().unwrap() = "zsh".into();
            let procs = Arc::new(FakeProcs::default());
            let incidents = Arc::new(FakeIncidents::default());
            let cfg = OrchestratorConfig {
                relaunch_backoff: vec![Duration::from_secs(30), Duration::from_secs(120)],
                stable_after: Duration::from_secs(600),
                ..Default::default()
            };
            let sup = Supervisor::new(
                dir.path().to_path_buf(),
                "/x/launch".into(),
                &cfg,
                tmux.clone(),
                procs.clone(),
                incidents.clone(),
            );
            let t0 = DateTime::from_timestamp(1_800_000_000, 0).unwrap();
            Rig {
                dir,
                tmux,
                procs,
                incidents,
                sup,
                t0,
            }
        }

        fn write_pid(&self, pid: i32, launched_at: i64) {
            std::fs::write(
                self.dir.path().join("orchestrator.pid"),
                format!("{pid} Mon Sep 29 10:00:00 2026 {launched_at}\n"),
            )
            .unwrap();
        }

        fn typed(&self) -> usize {
            self.tmux.typed.lock().unwrap().len()
        }

        fn incidents(&self) -> Vec<String> {
            self.incidents.0.lock().unwrap().clone()
        }

        /// A tick `secs` after t0.
        async fn tick(&self, secs: i64) {
            self.sup
                .tick(self.t0 + chrono::Duration::seconds(secs))
                .await;
        }

        /// Ticks at `secs` and `secs + 10` (the two shell checks), so a due relaunch types.
        async fn relaunch(&self, secs: i64) {
            self.tick(secs).await;
            self.tick(secs + 10).await;
        }
    }

    #[test]
    fn pid_file_with_a_spaced_start_time() {
        let r = parse_pid_file("123 Mon Sep 29 10:00:00 2026 1800000000\n").unwrap();
        assert_eq!(r.pid, 123);
        assert_eq!(r.start, "Mon Sep 29 10:00:00 2026");
        assert_eq!(r.launched, 1_800_000_000);
        assert!(parse_pid_file("123 x").is_none());
        assert!(parse_pid_file("abc def 5").is_none());
    }

    #[test]
    fn shell_word_quotes_only_when_needed() {
        assert_eq!(shell_word(Path::new("/a/b-c/launch")), "/a/b-c/launch");
        assert_eq!(shell_word(Path::new("/a b/it's")), "'/a b/it'\\''s'");
    }

    #[tokio::test]
    async fn no_pid_file_does_nothing() {
        let r = Rig::new();
        r.relaunch(0).await;
        assert_eq!(r.typed(), 0);
        assert!(r.incidents().is_empty());
    }

    #[tokio::test]
    async fn alive_session_is_left_alone() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp());
        r.procs.0.lock().unwrap().insert(10);
        r.relaunch(0).await;
        assert_eq!(r.typed(), 0);
        assert!(r.incidents().is_empty());
    }

    #[tokio::test]
    async fn dead_session_relaunches_after_two_shell_checks() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        r.tick(0).await;
        assert_eq!(r.typed(), 0, "one shell check is not enough");
        r.tick(3).await;
        assert_eq!(r.typed(), 0, "the second check must be 5 s later");
        r.tick(10).await;
        assert_eq!(
            *r.tmux.typed.lock().unwrap(),
            vec![("%2".to_string(), "/x/launch".to_string())]
        );
        assert!(r.incidents()[0].contains("not running"));
    }

    #[tokio::test]
    async fn backoff_sequence_then_give_up() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        // Relaunch 1 at once (types at t=10 after the second check).
        r.relaunch(0).await;
        assert_eq!(r.typed(), 1);
        // Unconfirmed: nothing more inside the 60 s window or the 30 s backoff.
        r.tick(50).await;
        assert_eq!(r.typed(), 1);
        // Relaunch 2 waits backoff[0] = 30 s after the previous attempt (t=10).
        r.relaunch(70).await;
        assert_eq!(r.typed(), 2, "attempt at t=80");
        // Relaunch 3 waits backoff[1] = 120 s after t=80.
        r.relaunch(150).await;
        assert_eq!(r.typed(), 2, "too soon");
        r.relaunch(210).await;
        assert_eq!(r.typed(), 3, "attempt at t=220");
        // All failed: give up with one incident, then stay quiet.
        r.tick(300).await;
        r.tick(10_000).await;
        r.relaunch(20_000).await;
        assert_eq!(r.typed(), 3);
        let gave_up: Vec<_> = r
            .incidents()
            .into_iter()
            .filter(|t| t.contains("not staying up"))
            .collect();
        assert_eq!(gave_up.len(), 1);
        assert!(gave_up[0].contains("3 relaunches"));

        // A live pid (the human launched it) clears the give-up.
        r.write_pid(11, r.t0.timestamp() + 30_000);
        r.procs.0.lock().unwrap().insert(11);
        r.tick(30_001).await;
        r.tick(30_700).await;
        r.procs.0.lock().unwrap().clear();
        r.relaunch(31_000).await;
        assert_eq!(r.typed(), 4);
    }

    #[tokio::test]
    async fn stable_session_resets_the_count() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        r.relaunch(0).await;
        assert_eq!(r.typed(), 1);
        // The relaunched session (pid 11, launched at t=15) comes up and stays up.
        r.write_pid(11, r.t0.timestamp() + 15);
        r.procs.0.lock().unwrap().insert(11);
        r.tick(20).await;
        r.tick(15 + 601).await;
        // It dies: the next relaunch is the first again, at once, not after backoff[0].
        r.procs.0.lock().unwrap().clear();
        r.relaunch(700).await;
        assert_eq!(r.typed(), 2);
    }

    #[tokio::test]
    async fn short_lived_session_keeps_the_count() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        r.relaunch(0).await; // attempt 1 at t=10
        r.write_pid(11, r.t0.timestamp() + 15);
        r.procs.0.lock().unwrap().insert(11);
        r.tick(20).await;
        r.procs.0.lock().unwrap().clear();
        // Died young: attempt 2 still waits backoff[0] after t=10.
        r.tick(25).await;
        assert_eq!(r.typed(), 1);
        r.relaunch(45).await;
        assert_eq!(r.typed(), 2);
    }

    #[tokio::test]
    async fn non_shell_foreground_is_never_typed_into() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        *r.tmux.command.lock().unwrap() = "vim".into();
        r.relaunch(0).await;
        r.relaunch(20).await;
        assert_eq!(r.typed(), 0);
        let inc = r.incidents();
        assert_eq!(inc.iter().filter(|t| t.contains("not a shell")).count(), 1);
        // The human leaves vim: back at a shell, it relaunches.
        *r.tmux.command.lock().unwrap() = "bash".into();
        r.relaunch(40).await;
        assert_eq!(r.typed(), 1);
    }

    #[tokio::test]
    async fn no_tagged_pane_reports_then_retries_once_tagged() {
        let r = Rig::new();
        r.write_pid(10, r.t0.timestamp() - 1000);
        *r.tmux.panes.lock().unwrap() = vec![("%1".into(), String::new())];
        r.relaunch(0).await;
        r.relaunch(20).await;
        assert_eq!(r.typed(), 0);
        let inc = r.incidents();
        assert_eq!(
            inc.iter()
                .filter(|t| t.contains("no pane is tagged"))
                .count(),
            1
        );
        *r.tmux.panes.lock().unwrap() = vec![("%1".into(), "orchestrator".into())];
        r.relaunch(40).await;
        assert_eq!(r.typed(), 1);
    }
}
