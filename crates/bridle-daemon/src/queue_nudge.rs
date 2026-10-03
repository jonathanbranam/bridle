//! Tells the manager the queue changed (f5ww). A burst of edits settles into one
//! message: each change restarts the wait (trailing edge).

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::types::{MessageKind, When};

use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget};
use crate::tasks::TaskManager;

/// How long the queue must stay unchanged before the nudge goes out.
pub const DEBOUNCE: Duration = Duration::from_secs(30);

/// The text carries no diff and never touches work in flight: the manager reads
/// `bridle queue` for the current state.
pub const MESSAGE: &str = "queue updated: re-read `bridle queue` before you next start something. \
    This doesn't change work already in flight (don't stop, re-plan or re-assign running workers).";

#[derive(Clone)]
pub struct QueueNudge {
    store: Store,
    manager: AgentManager,
    debounce: Duration,
    generation: Arc<AtomicU64>,
}

impl QueueNudge {
    pub fn new(store: Store, manager: AgentManager, debounce: Duration) -> Self {
        Self {
            store,
            manager,
            debounce,
            generation: Arc::default(),
        }
    }

    /// Called after a `queue.changed` by `actor` (a principal id). Returns at once;
    /// the wait happens in a spawned task.
    pub async fn changed(&self, actor: &str) {
        let name = actor.strip_prefix("agent:").unwrap_or(actor);
        if let Ok(Some(a)) = self.store.get_agent(name).await
            && a.role == "manager"
        {
            return;
        }
        let mine = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let this = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(this.debounce).await;
            if this.generation.load(Ordering::SeqCst) == mine {
                this.send(MESSAGE).await;
            }
        });
    }

    /// To the running manager, else to the orchestrator (who starts one when
    /// there's work, w2hj).
    async fn send(&self, text: &str) {
        let Ok(agents) = self.store.list_agents(false).await else {
            return;
        };
        let mut to: Vec<ToTarget> = agents
            .iter()
            .filter(|a| a.role == "manager" && a.state.is_running())
            .map(|m| ToTarget::Agent(m.id.clone()))
            .collect();
        if to.is_empty() {
            to.push(ToTarget::External(crate::wake::ORCHESTRATOR.to_string()));
        }
        for t in to {
            let _ = self
                .manager
                .send(
                    "system".to_string(),
                    t,
                    MessageKind::Note,
                    text.to_string(),
                    When::Idle,
                    None,
                )
                .await;
        }
    }
}

/// Tells the manager when a queued task's settle period ends (ny9u follow-up):
/// nothing else wakes an idle manager then. Compares "settling" at this tick with
/// the last one, in memory only; a restart can miss one note, which is fine.
#[derive(Clone)]
pub struct SettleWake {
    nudge: QueueNudge,
    tasks: TaskManager,
    settling: Arc<Mutex<HashSet<String>>>,
}

impl SettleWake {
    pub fn new(nudge: QueueNudge, tasks: TaskManager) -> Self {
        Self {
            nudge,
            tasks,
            settling: Arc::default(),
        }
    }

    /// One tick. Never fails: a send error is the nudge's to swallow.
    pub async fn tick(&self) {
        let now = self.tasks.list_tasks();
        let settled = self.scan(&now);
        for id in settled {
            self.nudge.send(&format!("{id} is now startable")).await;
        }
    }

    /// Ids that left the settling set by settling alone; stores the new set. Tasks
    /// blocked by deps or questions are never in the set, so they never note.
    fn scan(&self, tasks: &[bridle_api::types::Task]) -> Vec<String> {
        let now = chrono::Utc::now();
        let mut prev = self.settling.lock().expect("settling lock");
        let mut settling = HashSet::new();
        let mut settled = Vec::new();
        for t in tasks.iter().filter(|t| self.tasks.is_ready_unsettled(t)) {
            if self.tasks.settle_until(t, now).is_some() {
                settling.insert(t.id.clone());
            } else if prev.contains(&t.id) {
                settled.push(t.id.clone());
            }
        }
        *prev = settling;
        settled
    }
}
