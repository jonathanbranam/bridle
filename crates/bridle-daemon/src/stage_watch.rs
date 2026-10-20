//! Spawns a `designer` for each unblocked stage of a version 2 ticket, and tells the product
//! manager when one is not done in time (br-ewhy, epic br-tj7q; docs/design/stages.md).
//!
//! The ticket files are the truth: a stage is wanted when the ticket's row is `pending`, nothing
//! claims it (no `claimed_by` line, no row claim) and `stages::states` has an unblocked stage.
//! The writers it started are tracked in memory, one per ticket; after a restart a running
//! `designer-<id>-<stage>` agent is taken back by its name. A stage whose writer ended without
//! a successful `stage-done` (which lands its branch on main, br-upxq), or is still not written
//! `[stages] timeout` after the writer started, sends one message and is not retried; `ticket restage` (or any later edit of the ticket file) lets the
//! next tick start a writer again.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use bridle_api::types::{AgentState, MessageKind, SpawnRequest, TicketState, When};
use chrono::{DateTime, Utc};

use crate::stages::{self, State};
use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget, system_principal};
use crate::tasks::TicketManager;

/// Who hears of a stage that did not get done.
pub const PRODUCT_MANAGER: &str = "external:advisor/product-manager";

const ROLE: &str = "designer";

struct Writer {
    agent: String,
    stage: String,
    started: DateTime<Utc>,
    /// When the product manager was told; the stage is not retried until the file changes.
    alerted: Option<SystemTime>,
    /// The stage was seen written: if it is later not (`ticket changes`), the writer is a leftover,
    /// not a failure.
    done_seen: bool,
}

/// A writer holds the ticket only while its turn runs: an idle one has ended its turn and, with
/// its stage written, is finished (br-43ug).
fn holds(state: Option<AgentState>) -> bool {
    matches!(state, Some(AgentState::Starting | AgentState::Working))
}

enum Verdict {
    /// Still running: nothing new is spawned.
    Hold,
    /// The writer is finished or was overtaken: forget it.
    Drop,
    /// Ended without writing its stage.
    Failed,
    /// Nothing to say yet; the timeout decides.
    Wait,
}

fn verdict(done: bool, done_seen: bool, state: Option<AgentState>) -> Verdict {
    match (done, done_seen, holds(state)) {
        (true, _, true) => Verdict::Hold,
        (true, _, false) => Verdict::Drop,
        // `ticket changes` un-wrote the stage the writer had finished.
        (false, true, _) => Verdict::Drop,
        (false, false, false) => Verdict::Failed,
        (false, false, true) => Verdict::Wait,
    }
}

/// One version 2 ticket as the watcher reads it.
struct Prep {
    id: String,
    file: PathBuf,
    stages: Vec<stages::Stage>,
    marks: stages::Marks,
    claimed: bool,
}

#[derive(Clone)]
pub struct StageWatch {
    store: Store,
    manager: AgentManager,
    tasks: TicketManager,
    tickets: PathBuf,
    timeout: Duration,
    writers: Arc<Mutex<HashMap<String, Writer>>>,
}

fn front(text: &str) -> Vec<&str> {
    text.lines().skip(1).take_while(|l| *l != "---").collect()
}

/// The version 2 tickets under `<tickets>/open` that still have stages to prepare.
fn read_preps(tickets: &Path) -> Vec<Prep> {
    let Ok(rd) = std::fs::read_dir(tickets.join("open")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let file = entry.path();
        if file.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        if !text.starts_with("---\n") {
            continue;
        }
        let lines = front(&text);
        if !lines.contains(&"version: 2") {
            continue;
        }
        let id = lines
            .iter()
            .find_map(|l| l.strip_prefix("id:"))
            .map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
            .unwrap_or_default();
        let Ok(Some(stages)) = stages::parse(&lines) else {
            continue;
        };
        let claimed = lines.iter().any(|l| {
            l.strip_prefix("claimed_by:")
                .is_some_and(|v| !v.trim().is_empty())
        });
        if !id.is_empty() {
            out.push(Prep {
                id,
                marks: stages::marks(&text),
                file,
                stages,
                claimed,
            });
        }
    }
    out
}

/// For a version 2 ticket `id` under `<tickets>/open` that is not prepared, what is written and
/// what is not, for the refusal text; `None` for a version 1 ticket, an unknown one or a prepared
/// one (br-fyww).
pub fn unprepared(tickets: &Path, id: &str) -> Option<String> {
    let p = read_preps(tickets).into_iter().find(|p| p.id == id)?;
    if stages::prepared(&p.stages, &p.marks) {
        return None;
    }
    let (done, todo): (Vec<_>, Vec<_>) = stages::states(&p.stages, &p.marks)
        .into_iter()
        .partition(|(_, s)| matches!(s, State::Written | State::Signed));
    let names = |v: Vec<(String, State)>| {
        let v: Vec<_> = v.into_iter().map(|(n, _)| n).collect();
        if v.is_empty() {
            "none".to_string()
        } else {
            v.join(", ")
        }
    };
    Some(format!("{} written; {} not", names(done), names(todo)))
}

/// The stage `ticket next` would write: the first unblocked one, in schema order.
fn next_stage(p: &Prep) -> Option<String> {
    stages::states(&p.stages, &p.marks)
        .into_iter()
        .find(|(_, s)| *s == State::Unblocked)
        .map(|(n, _)| n)
}

fn is_done(p: &Prep, stage: &str) -> bool {
    stages::states(&p.stages, &p.marks).iter().any(|(n, s)| {
        n == stage
            && matches!(
                s,
                State::Written | State::WaitingForSignature | State::Signed
            )
    })
}

fn tail(id: &str) -> &str {
    id.rsplit('-').next().unwrap_or(id)
}

fn writer_prompt(id: &str, stage: &str) -> String {
    format!(
        "Write the {stage} stage of ticket {id}. Run `bridle ticket next {id} --json --comments` \
         first: it gives the rule, the template, the output path and the files you build on. \
         If it lists comments, the human added these comments; please address them. \
         Write the stage, build nothing, and commit \
         the ticket file and its stage folder only. Commit on your branch first, then run \
         `bridle ticket stage-done {id} {stage}`: the daemon checks and merges your branch to main \
         and, if it refuses, says why on the command (fix the branch and run it again). \
         Nobody else lands it, so you need not tell a manager. Then run \
         `bridle ticket next {id} --json --comments` again: if another stage is unblocked, write \
         it the same way, fully, before the next (one after another; order does not matter). \
         Write each stage from its own `depends_on` inputs, not from a sibling stage you wrote \
         in this session: do not let one shape the other. End when `ticket next` has no stage \
         left to write."
    )
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

impl StageWatch {
    pub fn new(
        store: Store,
        manager: AgentManager,
        tasks: TicketManager,
        tickets: PathBuf,
        timeout: Duration,
    ) -> Self {
        Self {
            store,
            manager,
            tasks,
            tickets,
            timeout,
            writers: Arc::default(),
        }
    }

    /// One tick, on the settle-wake loop. Never fails; a zero timeout turns the watcher off.
    pub async fn tick(&self) {
        if self.timeout.is_zero() {
            return;
        }
        let dir = self.tickets.clone();
        let Ok(preps) = tokio::task::spawn_blocking(move || read_preps(&dir)).await else {
            return;
        };
        let Ok(agents) = self.store.list_agents(false).await else {
            return;
        };
        let mut running = agents
            .iter()
            .filter(|a| a.role == ROLE && a.state.is_running())
            .count();
        let cap = self.manager.effective_max_workers() as usize;
        for p in preps {
            let Some(row) = self.tasks.get_task(&p.id) else {
                continue;
            };
            if row.state != TicketState::Pending || stages::prepared(&p.stages, &p.marks) {
                self.writers.lock().expect("writers lock").remove(&p.id);
                continue;
            }
            let prefix = format!("{ROLE}-{}-", tail(&p.id));
            if !self
                .writers
                .lock()
                .expect("writers lock")
                .contains_key(&p.id)
                && let Some(a) = agents.iter().find(|a| {
                    a.role == ROLE
                        && holds(Some(a.state))
                        && a.name.starts_with(&prefix)
                        && !is_done(&p, &a.name[prefix.len()..])
                })
            {
                let stage = a.name[prefix.len()..].to_string();
                self.writers.lock().expect("writers lock").insert(
                    p.id.clone(),
                    Writer {
                        agent: a.name.clone(),
                        stage,
                        started: a.created_at,
                        alerted: None,
                        done_seen: false,
                    },
                );
            }
            if self.check_writer(&p).await {
                continue;
            }
            if p.claimed || row.claimed_by.is_some() || running >= cap {
                continue;
            }
            let Some(stage) = next_stage(&p) else {
                continue;
            };
            if self.start(&p, &stage).await {
                running += 1;
            }
        }
    }

    /// Looks at the ticket's writer, if it has one. True while one holds the ticket (running,
    /// waiting to land, or alerted and not yet restaged): nothing new is spawned.
    async fn check_writer(&self, p: &Prep) -> bool {
        let (agent, stage, started, alerted, done_seen) = {
            let w = self.writers.lock().expect("writers lock");
            let Some(w) = w.get(&p.id) else {
                return false;
            };
            (
                w.agent.clone(),
                w.stage.clone(),
                w.started,
                w.alerted,
                w.done_seen,
            )
        };
        let done = is_done(p, &stage);
        let state = self
            .store
            .get_agent(&agent)
            .await
            .ok()
            .flatten()
            .map(|a| a.state);
        if done && let Some(w) = self.writers.lock().expect("writers lock").get_mut(&p.id) {
            w.done_seen = true;
        }
        // The writer goes on to the ticket's other unblocked stages (br-t6ev): while its turn
        // runs it still holds the ticket, so a second writer is not spawned beside it.
        let v = verdict(done, done_seen, state);
        if matches!(v, Verdict::Hold) {
            return true;
        }
        if matches!(v, Verdict::Drop) {
            self.writers.lock().expect("writers lock").remove(&p.id);
            return false;
        }
        if let Some(at) = alerted {
            if mtime(&p.file).is_some_and(|m| m > at) {
                self.writers.lock().expect("writers lock").remove(&p.id);
                return false;
            }
            return true;
        }
        let ended = matches!(v, Verdict::Failed);
        let reason = if Utc::now() - started
            >= chrono::Duration::from_std(self.timeout).unwrap_or_default()
        {
            Some("timed out")
        } else if ended {
            Some("the writer ended without a successful stage-done")
        } else {
            None
        };
        if let Some(reason) = reason {
            let text = format!("{}: {stage} not done by {agent}, {reason}", p.id);
            if let Some(w) = self.writers.lock().expect("writers lock").get_mut(&p.id) {
                w.alerted = Some(SystemTime::now());
            }
            let bridle = "bridle".to_string();
            let _ = self.tasks.note_task(&p.id, &bridle, &text).await;
            let _ = self
                .manager
                .send(
                    "system".to_string(),
                    ToTarget::External(PRODUCT_MANAGER.to_string()),
                    MessageKind::Note,
                    text,
                    When::Idle,
                    None,
                )
                .await;
        }
        true
    }

    async fn start(&self, p: &Prep, stage: &str) -> bool {
        let name = format!("{ROLE}-{}-{stage}", tail(&p.id));
        // A redo after `changes` finds the first writer's name taken: take a generated one.
        let name = match self.store.get_agent(&name).await {
            Ok(None) => Some(name),
            _ => None,
        };
        let req = SpawnRequest {
            role: ROLE.to_string(),
            name,
            prompt: Some(writer_prompt(&p.id, stage)),
            ticket: Some(p.id.clone()),
            ..Default::default()
        };
        match self.manager.spawn(req, &system_principal()).await {
            Ok(agent) => {
                self.writers.lock().expect("writers lock").insert(
                    p.id.clone(),
                    Writer {
                        agent: agent.name,
                        stage: stage.to_string(),
                        started: Utc::now(),
                        alerted: None,
                        done_seen: false,
                    },
                );
                true
            }
            // Held by the budget, the settle period or a drain: the next tick tries again.
            Err(e) => {
                tracing::debug!(ticket = %p.id, stage, error = %e, "stage writer not spawned");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, text: &str) {
        std::fs::create_dir_all(dir.join("open")).expect("mkdir");
        std::fs::write(dir.join("open").join(name), text).expect("write");
    }

    const V2: &str = "---\nid: br-aaaa\nversion: 2\nstages:\n  proposal: {review: human}\n  design: {requires: [proposal]}\n---\nbody\n";

    #[test]
    fn reads_only_version_2_and_picks_the_first_unblocked_stage() {
        let tmp = tempfile::tempdir().expect("tmp");
        write(tmp.path(), "a-aaaa.md", V2);
        write(tmp.path(), "b-bbbb.md", "---\nid: br-bbbb\n---\nold\n");
        let preps = read_preps(tmp.path());
        assert_eq!(preps.len(), 1);
        assert_eq!(next_stage(&preps[0]).as_deref(), Some("proposal"));
        assert!(!preps[0].claimed);
        assert_eq!(tail("br-aaaa"), "aaaa");
    }

    #[test]
    fn an_idle_writer_does_not_hold_the_ticket() {
        use AgentState::*;
        assert!(holds(Some(Starting)) && holds(Some(Working)));
        assert!(!holds(Some(Idle)) && !holds(None) && !holds(Some(Stopped)));
        // Finished stage, idle writer: dropped, so a redo can start.
        assert!(matches!(verdict(true, true, Some(Idle)), Verdict::Drop));
        assert!(matches!(verdict(true, true, Some(Working)), Verdict::Hold));
    }

    #[test]
    fn a_stage_sent_back_by_changes_is_not_a_failure() {
        use AgentState::*;
        // Written once, now un-written: the writer is dropped whether it was stopped or idle.
        assert!(matches!(verdict(false, true, None), Verdict::Drop));
        assert!(matches!(verdict(false, true, Some(Idle)), Verdict::Drop));
        // Never written and ended: still a failure.
        assert!(matches!(verdict(false, false, Some(Idle)), Verdict::Failed));
        assert!(matches!(
            verdict(false, false, Some(Working)),
            Verdict::Wait
        ));
    }

    #[test]
    fn re_adoption_skips_a_written_stage() {
        let tmp = tempfile::tempdir().expect("tmp");
        write(
            tmp.path(),
            "a-aaaa.md",
            &V2.replace("version: 2", "version: 2\nwritten: [proposal]"),
        );
        let p = &read_preps(tmp.path())[0];
        assert!(is_done(p, "proposal"));
        assert!(!is_done(p, "design"));
    }

    #[test]
    fn a_claim_or_a_signature_wait_means_no_new_stage() {
        let tmp = tempfile::tempdir().expect("tmp");
        write(
            tmp.path(),
            "a-aaaa.md",
            &V2.replace(
                "version: 2",
                "version: 2\nclaimed_by: human\nwritten: [proposal]",
            ),
        );
        let p = &read_preps(tmp.path())[0];
        assert!(p.claimed);
        // The proposal waits for the human's signature, so design is still blocked.
        assert_eq!(next_stage(p), None);
        assert!(is_done(p, "proposal"));
    }
}
