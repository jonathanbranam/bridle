//! Tells the manager the queue changed (f5ww). A burst of edits settles into one
//! message: each change restarts the wait (trailing edge).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bridle_api::types::{MessageKind, When};

use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget};

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
                this.send().await;
            }
        });
    }

    /// To the running manager, else to the orchestrator (who starts one when
    /// there's work, w2hj).
    async fn send(&self) {
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
                    MESSAGE.to_string(),
                    When::Idle,
                    None,
                )
                .await;
        }
    }
}
