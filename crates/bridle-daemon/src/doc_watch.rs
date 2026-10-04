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
use chrono::{DateTime, Duration, Utc};

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

/// The agent's name for a document: `doc-<file stem>`.
pub fn agent_name(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let slug: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("doc-{}", slug.trim_matches('-'))
}

/// Comment threads whose last reply is the human's, each as its quoted block.
pub fn pending_threads(text: &str) -> Vec<String> {
    let mut threads: Vec<Vec<&str>> = Vec::new();
    for line in text.lines() {
        if line.starts_with("> [!comment]") {
            threads.push(vec![line]);
        } else if line.starts_with('>')
            && let Some(t) = threads.last_mut()
            && t.last().is_some_and(|l| l.starts_with('>'))
        {
            t.push(line);
        }
    }
    threads
        .into_iter()
        .filter(|t| last_author_is_human(t))
        .map(|t| t.join("\n"))
        .collect()
}

/// An author with "agent" in the name is the document agent; anyone else is the human.
fn last_author_is_human(lines: &[&str]) -> bool {
    let mut human = true;
    for l in lines {
        let who = if let Some(h) = l.strip_prefix("> [!comment]") {
            h.trim().split(',').next()
        } else if let Some(r) = l.strip_prefix("> **") {
            r.split(',').next()
        } else {
            continue;
        };
        human = !who.unwrap_or("").to_lowercase().contains("agent");
    }
    human
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
        let seen = pending_threads(text).join("\n\n");
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
}

impl DocWatcher {
    pub fn new(store: Store, manager: AgentManager, repo: PathBuf, cfg: ReviewConfig) -> Self {
        Self {
            store,
            manager,
            repo,
            cfg,
            core: Default::default(),
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
            if self.deliver(&d).await {
                self.core.lock().expect("doc watch lock").delivered(&d.path);
            }
        }
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
        assert_eq!(pending_threads(HUMAN).len(), 1);
        assert!(pending_threads(ANSWERED).is_empty());
        let again = format!("{ANSWERED}>\n> **human, 14:20:** and?\n");
        assert_eq!(pending_threads(&again).len(), 1);
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
            |n| n == "doc-c",
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
            "doc-my-ticket-x8jt"
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
}
