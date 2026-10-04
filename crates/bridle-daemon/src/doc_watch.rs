//! Notices new comments in documents under review and wakes that document's agent
//! (x8jt slice 2). The format is in `workflow/base/roles/document-reviewer.md`.
//!
//! A document is "under review" when its repo-relative path is a line in
//! `.bridle/review-documents.txt` (`bridle review add`). A thread is *pending* when its last
//! reply is the human's. Pending text that has been still for the quiet period goes to the
//! document's agent as one batch; the agent's own replies end the pending state, so its edits
//! don't wake it again, and a restart doesn't resend answered threads.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bridle_api::types::{AgentState, MessageKind, SpawnRequest, When, Workdir};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use sha2::Digest;

use crate::config::ReviewConfig;
use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget, system_principal};

pub const REGISTRY: &str = ".bridle/review-documents.txt";
const ROLE: &str = "document-reviewer";

/// The registered paths (blank lines and `#` comments skipped).
pub fn read_registry(repo: &Path) -> Vec<String> {
    std::fs::read_to_string(repo.join(REGISTRY))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Adds (`on`) or removes a path; true when the file changed.
pub fn set_registered(repo: &Path, path: &str, on: bool) -> std::io::Result<bool> {
    let mut paths = read_registry(repo);
    let had = paths.iter().any(|p| p == path);
    if had == on {
        return Ok(false);
    }
    if on {
        paths.push(path.to_string());
    } else {
        paths.retain(|p| p != path);
    }
    let file = repo.join(REGISTRY);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        file,
        paths.iter().map(|p| format!("{p}\n")).collect::<String>(),
    )?;
    Ok(true)
}

/// Agent names are at most 40 characters (`worktree.rs`).
const NAME_MAX: usize = 40;
/// The ticket ID alphabet (`bridle ticket new`).
const ID_ALPHABET: &str = "abcdefghjkmnpqrstuvwxyz23456789";

/// The agent's name for a document. A ticket (stem ending `-<id>`) is `doc-<id>`: short and
/// stable. Any other file is `doc-<slug>-<hash>`, the slug cut to fit and the hash of the path
/// keeping two long names apart.
pub fn agent_name(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let slug: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-');
    if let Some((_, id)) = slug.rsplit_once('-')
        && id.len() == 4
        && id.chars().all(|c| ID_ALPHABET.contains(c))
    {
        return format!("doc-{id}");
    }
    let hash: String = sha2::Sha256::digest(path.as_bytes())[..3]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let room = NAME_MAX - "doc-".len() - "-".len() - hash.len();
    let cut = &slug[..slug.len().min(room)];
    format!("doc-{}-{hash}", cut.trim_end_matches('-'))
}

/// What bridle appends to a thread's newest human entry when it sends the thread to the agent:
/// ` · sent 2026-10-04 21:14` (US Eastern). Plain text in the document; a restart doesn't resend.
const SENT: &str = " · sent ";
const STAMP_LEN: usize = "2026-10-04 21:14".len();

/// The line without its sent mark.
fn strip_mark(line: &str) -> &str {
    match line.rfind(SENT) {
        Some(i) if line.len() - i - SENT.len() == STAMP_LEN => &line[..i],
        _ => line,
    }
}

/// One comment thread: the line numbers of its quoted block, and which one is its newest entry
/// (the `[!comment]` header or the last `> **author, time:**` reply).
struct Thread {
    lines: Vec<usize>,
    newest: usize,
}

fn scan(all: &[&str]) -> Vec<Thread> {
    let mut threads: Vec<Thread> = Vec::new();
    for (i, line) in all.iter().enumerate() {
        if line.starts_with("> [!comment]") {
            threads.push(Thread {
                lines: vec![i],
                newest: i,
            });
        } else if line.starts_with('>')
            && let Some(t) = threads.last_mut()
        {
            t.lines.push(i);
            if line.starts_with("> **") {
                t.newest = i;
            }
        }
    }
    threads
}

impl Thread {
    /// The quoted block, without sent marks.
    fn block(&self, all: &[&str]) -> String {
        self.lines
            .iter()
            .map(|&i| strip_mark(all[i]))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Comment threads whose last reply is the human's, each as its quoted block. Threads whose
/// newest entry already carries a sent mark are left out unless `resend`.
pub fn pending_threads(text: &str, resend: bool) -> Vec<String> {
    let all: Vec<&str> = text.split('\n').collect();
    scan(&all)
        .iter()
        .filter(|t| is_human(all[t.newest]))
        .filter(|t| resend || strip_mark(all[t.newest]).len() == all[t.newest].len())
        .map(|t| t.block(&all))
        .collect()
}

/// The text with a sent mark on the newest entry of each thread whose block is in `sent`
/// (a mark already there is replaced).
fn mark_sent(text: &str, sent: &[String], stamp: &str) -> String {
    let all: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = all.iter().map(|l| l.to_string()).collect();
    for t in scan(&all) {
        if is_human(all[t.newest]) && sent.contains(&t.block(&all)) {
            out[t.newest] = format!("{}{SENT}{stamp}", strip_mark(all[t.newest]));
        }
    }
    out.join("\n")
}

/// An author with "agent" in the name is the document agent; anyone else is the human.
fn is_human(entry: &str) -> bool {
    let who = if let Some(h) = entry.strip_prefix("> [!comment]") {
        h.trim().split(',').next()
    } else if let Some(r) = entry.strip_prefix("> **") {
        r.split(',').next()
    } else {
        None
    };
    !who.unwrap_or("").to_lowercase().contains("agent")
}

/// UTC offset of US Eastern at `at`: -4h in daylight time, else -5h (the US rule since 2007).
fn eastern_offset(at: DateTime<Utc>) -> Duration {
    let year = at.year();
    let transition = |month, nth, hour| {
        NaiveDate::from_weekday_of_month_opt(year, month, Weekday::Sun, nth)
            .and_then(|d| d.and_hms_opt(hour, 0, 0))
            .map(|d| d.and_utc())
    };
    // 2:00 EST = 07:00 UTC in March; 2:00 EDT = 06:00 UTC in November.
    match (transition(3, 2, 7), transition(11, 1, 6)) {
        (Some(start), Some(end)) if at >= start && at < end => Duration::hours(-4),
        _ => Duration::hours(-5),
    }
}

/// `YYYY-MM-DD HH:MM` in US Eastern, the human's time (rule human-timezone).
fn stamp(now: DateTime<Utc>) -> String {
    (now + eastern_offset(now))
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

#[derive(Default)]
struct Doc {
    /// Pending threads at the last look; `changed_at` is when this last differed.
    seen: String,
    changed_at: Option<DateTime<Utc>>,
    /// What the agent was last given.
    delivered: String,
}

/// A document's batch, ready to go.
#[derive(Debug, PartialEq)]
pub struct Due {
    pub path: String,
    pub batch: String,
}

/// The debounce state, with time passed in so tests need no clock.
#[derive(Default)]
pub struct Core {
    docs: HashMap<String, Doc>,
}

impl Core {
    pub fn observe(&mut self, path: &str, text: &str, now: DateTime<Utc>) {
        let seen = pending_threads(text, false).join("\n\n");
        let d = self.docs.entry(path.to_string()).or_default();
        if d.seen != seen {
            d.seen = seen;
            d.changed_at = Some(now);
        }
    }

    /// Documents whose pending threads have been still for `quiet`, oldest first.
    pub fn due(&self, now: DateTime<Utc>, quiet: Duration) -> Vec<Due> {
        let mut v: Vec<(DateTime<Utc>, Due)> = self
            .docs
            .iter()
            .filter(|(_, d)| !d.seen.is_empty() && d.seen != d.delivered)
            .filter_map(|(p, d)| {
                let at = d.changed_at?;
                (now - at >= quiet).then(|| {
                    (
                        at,
                        Due {
                            path: p.clone(),
                            batch: d.seen.clone(),
                        },
                    )
                })
            })
            .collect();
        v.sort_by_key(|(at, _)| *at);
        v.into_iter().map(|(_, d)| d).collect()
    }

    pub fn delivered(&mut self, path: &str) {
        if let Some(d) = self.docs.get_mut(path) {
            d.delivered = d.seen.clone();
        }
    }
}

/// Which due documents go now: one whose agent is already running always does; one that needs
/// a start takes a slot, and waits (stays due) when the cap is full.
pub fn admit(
    due: Vec<Due>,
    is_running: impl Fn(&str) -> bool,
    mut running: usize,
    cap: usize,
) -> Vec<Due> {
    let mut out = Vec::new();
    for d in due {
        if is_running(&agent_name(&d.path)) {
            out.push(d);
        } else if running < cap {
            running += 1;
            out.push(d);
        }
    }
    out
}

/// An idle document agent past its expiry stops (resumed when its next batch comes).
pub fn expired(now: DateTime<Utc>, last_active: DateTime<Utc>, idle: Duration) -> bool {
    now - last_active >= idle
}

fn prompt(repo: &Path, path: &str, batch: &str) -> String {
    format!(
        "New comments to answer in {} (the document under review). Read it, answer each thread \
         below as your role says, and commit.\n\n{batch}",
        repo.join(path).display()
    )
}

#[derive(Clone)]
pub struct DocWatcher {
    store: Store,
    manager: AgentManager,
    repo: PathBuf,
    cfg: ReviewConfig,
    core: std::sync::Arc<std::sync::Mutex<Core>>,
    /// Held while a batch is read, delivered and marked, so a tick and a review-now never send
    /// the same thread twice.
    sending: std::sync::Arc<tokio::sync::Mutex<()>>,
}

/// What `review now` did.
#[derive(Debug, PartialEq, Eq)]
pub struct Reviewed {
    pub agent: String,
    /// Threads sent; 0 when none were pending (or unsent).
    pub threads: usize,
}

impl DocWatcher {
    pub fn new(store: Store, manager: AgentManager, repo: PathBuf, cfg: ReviewConfig) -> Self {
        Self {
            store,
            manager,
            repo,
            cfg,
            core: Default::default(),
            sending: Default::default(),
        }
    }

    pub async fn tick(&self) {
        self.tick_at(Utc::now()).await;
    }

    pub async fn tick_at(&self, now: DateTime<Utc>) {
        let Ok(agents) = self.store.list_agents(false).await else {
            return;
        };
        let docs: Vec<&_> = agents.iter().filter(|a| a.role == ROLE).collect();
        let idle = Duration::from_std(self.cfg.idle).unwrap_or(Duration::hours(4));
        for a in docs.iter().filter(|a| a.state == AgentState::Idle) {
            let last = a.last_event_at.unwrap_or(a.updated_at);
            if expired(now, last, idle) {
                let _ = self.manager.stop(&a.id, false, &system_principal()).await;
            }
        }

        let due = {
            let mut core = self.core.lock().expect("doc watch lock");
            for p in read_registry(&self.repo) {
                if let Ok(text) = std::fs::read_to_string(self.repo.join(&p)) {
                    core.observe(&p, &text, now);
                }
            }
            core.due(
                now,
                Duration::from_std(self.cfg.quiet).unwrap_or(Duration::minutes(7)),
            )
        };
        let running = docs.iter().filter(|a| a.state.is_running()).count();
        let go = admit(
            due,
            |n| docs.iter().any(|a| a.name == n && a.state.is_running()),
            running,
            self.cfg.max_agents as usize,
        );
        for d in go {
            // `Ok(0)`: the text changed since it was observed; the next tick looks again.
            if let Ok(n) = self.send(&d.path, false, Some(&d.batch), now).await
                && n > 0
            {
                self.core.lock().expect("doc watch lock").delivered(&d.path);
            }
        }
    }

    /// `bridle review add`. With `only_if_pending`, a document with no pending thread is left
    /// as it is. Adding one already under review changes nothing. Returns whether the document
    /// is under review afterwards.
    pub fn add(&self, path: &str, only_if_pending: bool) -> Result<bool, String> {
        if Path::new(path)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(format!("{path} is not a path inside the repo"));
        }
        let file = self.repo.join(path);
        let text = std::fs::read_to_string(&file)
            .map_err(|e| format!("{path} is not a readable file under the repo: {e}"))?;
        if only_if_pending && pending_threads(&text, false).is_empty() {
            return Ok(read_registry(&self.repo).iter().any(|p| p == path));
        }
        set_registered(&self.repo, path, true)
            .map_err(|e| format!("writing the review list: {e}"))?;
        Ok(true)
    }

    /// `bridle review now`: sends the document's pending threads at once, skipping the quiet
    /// period (and the agent cap: the human asked). Threads already marked sent stay out unless
    /// `resend`.
    pub async fn review_now(&self, path: &str, resend: bool) -> Result<Reviewed, String> {
        if !read_registry(&self.repo).iter().any(|p| p == path) {
            return Err(format!(
                "{path} is not under review (bridle review add {path})"
            ));
        }
        let threads = self.send(path, resend, None, Utc::now()).await?;
        Ok(Reviewed {
            agent: agent_name(path),
            threads,
        })
    }

    /// Reads the document, delivers its pending threads (all of them, with `resend`) and marks
    /// them sent. With `expect`, sends only if the pending text is still exactly that. Returns
    /// the number of threads delivered.
    async fn send(
        &self,
        path: &str,
        resend: bool,
        expect: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<usize, String> {
        let _guard = self.sending.lock().await;
        let file = self.repo.join(path);
        let text = std::fs::read_to_string(&file).map_err(|e| format!("{path}: {e}"))?;
        let threads = pending_threads(&text, resend);
        let batch = threads.join("\n\n");
        if threads.is_empty() || expect.is_some_and(|e| e != batch) {
            return Ok(0);
        }
        if !self
            .deliver(&Due {
                path: path.to_string(),
                batch,
            })
            .await
        {
            return Err("the document's agent could not be started; see the daemon log".into());
        }
        // Read again: the agent may already have edited the file.
        let latest = std::fs::read_to_string(&file).map_err(|e| format!("{path}: {e}"))?;
        let marked = mark_sent(&latest, &threads, &stamp(now));
        std::fs::write(&file, marked).map_err(|e| format!("marking {path}: {e}"))?;
        Ok(threads.len())
    }

    /// Start, resume or message the document's agent. False when it didn't go.
    async fn deliver(&self, d: &Due) -> bool {
        let name = agent_name(&d.path);
        let text = prompt(&self.repo, &d.path, &d.batch);
        let p = system_principal();
        let agent = self.store.get_agent(&name).await.ok().flatten();
        let r = match agent {
            None => self
                .manager
                .spawn(
                    SpawnRequest {
                        role: ROLE.to_string(),
                        name: Some(name),
                        prompt: Some(text),
                        workdir: Some(Workdir::Repo),
                        ..Default::default()
                    },
                    &p,
                )
                .await
                .map(|_| ()),
            Some(a) => {
                let resumed = if a.state.is_running() {
                    Ok(())
                } else {
                    self.manager.resume(&a.id, false, &p).await.map(|_| ())
                };
                match resumed {
                    Ok(()) => self
                        .manager
                        .send(
                            "system".to_string(),
                            ToTarget::Agent(a.id),
                            MessageKind::Note,
                            text,
                            When::Idle,
                            None,
                        )
                        .await
                        .map(|_| ()),
                    Err(e) => Err(e),
                }
            }
        };
        if let Err(e) = &r {
            tracing::warn!(path = %d.path, error = %e, "document agent not started; will retry");
        }
        r.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HUMAN: &str = "Line.\n\n> [!comment] human, 2026-10-03 14:05, on \"Line\"\n> Why?\n";
    const ANSWERED: &str = "Line.\n\n> [!comment] human, 2026-10-03 14:05, on \"Line\"\n> Why?\n>\n> **docs agent, 14:06:** @human Because.\n";

    fn t(min: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_800_000_000, 0).unwrap() + Duration::minutes(min)
    }

    #[test]
    fn answered_threads_are_not_pending() {
        assert_eq!(pending_threads(HUMAN, false).len(), 1);
        assert!(pending_threads(ANSWERED, false).is_empty());
        let again = format!("{ANSWERED}>\n> **human, 14:20:** and?\n");
        assert_eq!(pending_threads(&again, false).len(), 1);
    }

    #[test]
    fn debounce_restarts_on_each_change() {
        let q = Duration::minutes(7);
        let mut c = Core::default();
        c.observe("a.md", HUMAN, t(0));
        assert!(c.due(t(6), q).is_empty());
        let more = format!("{HUMAN}>\n> **human, 14:09:** more\n");
        c.observe("a.md", &more, t(5));
        assert!(c.due(t(11), q).is_empty());
        let due = c.due(t(12), q);
        assert_eq!(due.len(), 1);
        assert!(due[0].batch.contains("more"));
    }

    #[test]
    fn batch_has_every_pending_thread_and_sends_once() {
        let two = format!("{HUMAN}\nOther.\n\n> [!comment] human, 14:07, on \"Other\"\n> Hm?\n");
        let mut c = Core::default();
        c.observe("a.md", &two, t(0));
        let due = c.due(t(7), Duration::minutes(7));
        assert_eq!(due[0].batch.matches("[!comment]").count(), 2);
        c.delivered("a.md");
        assert!(c.due(t(30), Duration::minutes(7)).is_empty());
        // The agent answers; no new batch, and a later human reply is one.
        c.observe("a.md", ANSWERED, t(31));
        assert!(c.due(t(60), Duration::minutes(7)).is_empty());
    }

    #[test]
    fn cap_holds_back_starts_not_running_agents() {
        let due = |p: &str| Due {
            path: p.to_string(),
            batch: "x".into(),
        };
        let go = admit(
            vec![due("a.md"), due("b.md"), due("c.md")],
            |n| n == agent_name("c.md"),
            2,
            3,
        );
        let paths: Vec<_> = go.iter().map(|d| d.path.as_str()).collect();
        assert_eq!(paths, ["a.md", "c.md"]);
    }

    #[test]
    fn idle_agents_expire() {
        let idle = Duration::hours(4);
        assert!(!expired(t(239), t(0), idle));
        assert!(expired(t(240), t(0), idle));
    }

    #[test]
    fn names_and_registry() {
        assert_eq!(
            agent_name("docs/tickets/open/My Ticket_x8jt.md"),
            "doc-x8jt"
        );
        let real = agent_name(
            "docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md",
        );
        assert_eq!(real, "doc-3haz");
        assert!(crate::worktree::validate_agent_name(&real).is_ok());
        let (a, b) = (
            agent_name("docs/a-very-long-document-name-that-keeps-going-and-going-one.md"),
            agent_name("docs/a-very-long-document-name-that-keeps-going-and-going-two.md"),
        );
        assert!(a.len() <= 40 && b.len() <= 40 && a != b);
        assert!(crate::worktree::validate_agent_name(&a).is_ok());
        let short = agent_name("notes/Plan.md");
        assert!(
            short.starts_with("doc-plan-") && crate::worktree::validate_agent_name(&short).is_ok()
        );
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bridle")).unwrap();
        std::fs::write(dir.path().join(REGISTRY), "# x\na.md\n\n b.md \n").unwrap();
        assert_eq!(read_registry(dir.path()), ["a.md", "b.md"]);
        assert!(set_registered(dir.path(), "c.md", true).unwrap());
        assert!(!set_registered(dir.path(), "c.md", true).unwrap());
        assert!(set_registered(dir.path(), "a.md", false).unwrap());
        assert_eq!(read_registry(dir.path()), ["b.md", "c.md"]);
    }

    #[test]
    fn sent_mark_goes_on_the_newest_human_entry_once() {
        let text =
            format!("{HUMAN}>\n> **docs agent, 14:06:** Because.\n>\n> **human, 14:20:** and?\n");
        let sent = pending_threads(&text, false);
        assert_eq!(sent.len(), 1);
        let marked = mark_sent(&text, &sent, "2026-10-04 21:14");
        assert!(marked.contains("> **human, 14:20:** and? · sent 2026-10-04 21:14\n"));
        assert_eq!(marked.matches(" · sent ").count(), 1);
        // Marked threads are left out, are still the human's (so the watcher's pending state
        // is unchanged by the mark), and come back with resend, without the old mark.
        assert!(pending_threads(&marked, false).is_empty());
        let again = pending_threads(&marked, true);
        assert_eq!(again, sent);
        let remarked = mark_sent(&marked, &again, "2026-10-05 08:00");
        assert!(remarked.contains("and? · sent 2026-10-05 08:00\n"));
        assert_eq!(remarked.matches(" · sent ").count(), 1);
        // A new human reply is unsent again.
        let reply = format!("{marked}>\n> **human, 14:30:** hello?\n");
        assert_eq!(pending_threads(&reply, false).len(), 1);
        // A mark on a header works too.
        let m = mark_sent(HUMAN, &pending_threads(HUMAN, false), "2026-10-04 21:14");
        assert!(m.contains("on \"Line\" · sent 2026-10-04 21:14\n"));
        assert!(pending_threads(&m, false).is_empty());
    }

    #[test]
    fn mark_does_not_retrigger_the_watcher() {
        let mut c = Core::default();
        c.observe("a.md", HUMAN, t(0));
        assert_eq!(c.due(t(7), Duration::minutes(7)).len(), 1);
        c.delivered("a.md");
        let marked = mark_sent(HUMAN, &pending_threads(HUMAN, false), "2026-10-04 21:14");
        c.observe("a.md", &marked, t(8));
        assert!(c.due(t(60), Duration::minutes(7)).is_empty());
    }

    #[test]
    fn stamp_is_eastern() {
        // 2026-10-04 01:14 UTC is 21:14 the evening before in EDT; January is EST.
        let edt = DateTime::parse_from_rfc3339("2026-10-05T01:14:00Z")
            .unwrap()
            .to_utc();
        assert_eq!(stamp(edt), "2026-10-04 21:14");
        let est = DateTime::parse_from_rfc3339("2026-01-05T01:14:00Z")
            .unwrap()
            .to_utc();
        assert_eq!(stamp(est), "2026-01-04 20:14");
    }
}
