//! Keeps `open` tasks from sitting unplanned (xz4f). A task readied by anyone but the PM
//! tells the PM (else the orchestrator) it is open, one message per settled burst; one left
//! open past `[tasks] open_stale` goes back to `pending` with a comment saying why. A prepared
//! version 2 ticket instead stays open and the PM and orchestrator are told again (h5kz).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::types::{MessageKind, PrincipalId, Ticket, TicketState, When, event_kind};

use crate::events::Emitter;
use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget};
use crate::tasks::TicketManager;

/// How long readies must stop arriving before the message goes out (as queue_nudge).
pub const DEBOUNCE: Duration = Duration::from_secs(30);

#[derive(Default)]
struct Pending {
    ids: Vec<String>,
    generation: u64,
}

#[derive(Clone)]
pub struct OpenWatch {
    store: Store,
    manager: AgentManager,
    tasks: TicketManager,
    emitter: Emitter,
    debounce: Duration,
    stale_after: Duration,
    pending: Arc<Mutex<Pending>>,
    /// Who readied each task, to tell them if it goes stale. In memory only: after a restart
    /// the task's creator is told instead.
    readied_by: Arc<Mutex<HashMap<String, PrincipalId>>>,
    /// When a prepared version 2 ticket was last announced again by `tick`, so one open past
    /// `open_stale` is told once per period, not once per tick. In memory only.
    reannounced: Arc<Mutex<HashMap<String, std::time::Instant>>>,
}

impl OpenWatch {
    pub fn new(
        store: Store,
        manager: AgentManager,
        tasks: TicketManager,
        emitter: Emitter,
        debounce: Duration,
        stale_after: Duration,
    ) -> Self {
        Self {
            store,
            manager,
            tasks,
            emitter,
            debounce,
            stale_after,
            pending: Arc::default(),
            readied_by: Arc::default(),
            reannounced: Arc::default(),
        }
    }

    /// What follows any ready, whoever did it: tell the PM it is open (`readied`) and note the
    /// filing for the manager. The API's ready and the daemon's own both end here, so they
    /// cannot drift apart (h5kz).
    pub async fn ready_effects(&self, task: &Ticket, actor: &PrincipalId) {
        self.readied(&task.id, actor).await;
        let open = self
            .tasks
            .list_tasks()
            .iter()
            .filter(|t| t.state == TicketState::Open)
            .count();
        self.manager
            .note_task_filed(&task.id, &task.title, open)
            .await;
    }

    /// A version 2 ticket the daemon opened because it is prepared: the state-change event the
    /// API's ready emits, then the same effects.
    pub async fn auto_readied(&self, task: &Ticket) {
        let bridle = "bridle".to_string();
        let _ = self
            .emitter
            .emit(
                event_kind::TASK_STATE,
                bridle.clone(),
                None,
                serde_json::json!({
                    "task": task.id, "from": TicketState::Pending, "to": task.state,
                }),
            )
            .await;
        self.ready_effects(task, &bridle).await;
    }

    /// Called after `actor` readied `task_id`. Returns at once; the wait happens in a spawned task.
    pub async fn readied(&self, task_id: &str, actor: &PrincipalId) {
        self.readied_by
            .lock()
            .expect("readied_by lock")
            .insert(task_id.to_string(), actor.clone());
        let name = actor.strip_prefix("agent:").unwrap_or(actor);
        if let Ok(Some(a)) = self.store.get_agent(name).await
            && a.role == "project-manager"
        {
            return;
        }
        let mine = {
            let mut p = self.pending.lock().expect("pending lock");
            p.ids.push(task_id.to_string());
            p.generation += 1;
            p.generation
        };
        let this = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(this.debounce).await;
            let ids = {
                let mut p = this.pending.lock().expect("pending lock");
                if p.generation != mine {
                    return;
                }
                std::mem::take(&mut p.ids)
            };
            this.announce(ids).await;
        });
    }

    async fn announce(&self, ids: Vec<String>) {
        let lines: Vec<String> = ids
            .iter()
            .filter_map(|id| self.tasks.get_task(id))
            .filter(|t| t.state == TicketState::Open)
            .map(|t| format!("- {} ({})", t.id, t.title))
            .collect();
        if lines.is_empty() {
            return;
        }
        let text = format!(
            "{} open and not yet planned: triage and plan {}.\n{}",
            if lines.len() == 1 {
                "a task is"
            } else {
                "tasks are"
            },
            if lines.len() == 1 { "it" } else { "them" },
            lines.join("\n")
        );
        let Ok(agents) = self.store.list_agents(false).await else {
            return;
        };
        let mut to: Vec<ToTarget> = agents
            .iter()
            .filter(|a| a.role == "project-manager" && a.state.is_running())
            .map(|a| ToTarget::Agent(a.id.clone()))
            .collect();
        if to.is_empty() {
            to.push(ToTarget::External(crate::wake::ORCHESTRATOR.to_string()));
        }
        for t in to {
            self.send(t, &text).await;
        }
    }

    async fn send(&self, to: ToTarget, text: &str) {
        let _ = self
            .manager
            .send(
                "system".to_string(),
                to,
                MessageKind::Note,
                text.to_string(),
                When::Idle,
                None,
            )
            .await;
    }

    /// One tick, on the settle-wake loop. Never fails.
    pub async fn tick(&self) {
        if self.stale_after.is_zero() {
            return;
        }
        let Ok(after) = chrono::Duration::from_std(self.stale_after) else {
            return;
        };
        let now = chrono::Utc::now();
        let stale: Vec<_> = self
            .tasks
            .list_tasks()
            .into_iter()
            .filter(|t| t.state == TicketState::Open && now - t.updated_at >= after)
            .collect();
        for t in stale {
            // A prepared version 2 ticket has nothing to re-prepare: pending would only
            // be undone by the next link sync, so say it is still open instead.
            if self.tasks.is_prepared_v2(&t.id) {
                self.reannounce(&t.id, &t.title).await;
                continue;
            }
            let hours = self.stale_after.as_secs_f64() / 3600.0;
            let reason = format!(
                "open {hours}h, never planned: back to pending. Ready it again once someone will plan it."
            );
            let system = "system".to_string();
            match self.tasks.unready_stale(&t.id, &system, &reason).await {
                Ok(Some(task)) => {
                    let _ = self
                        .emitter
                        .emit(
                            event_kind::TASK_STATE,
                            system,
                            None,
                            serde_json::json!({
                                "task": task.id, "from": TicketState::Open, "to": task.state,
                            }),
                        )
                        .await;
                    self.tell_stale(&task.id, &task.title, &reason).await;
                }
                Ok(None) => {}
                Err(e) => tracing::warn!(task = %t.id, "returning a stale open task failed: {e}"),
            }
        }
    }

    async fn reannounce(&self, id: &str, title: &str) {
        {
            let mut seen = self.reannounced.lock().expect("reannounced lock");
            if seen
                .get(id)
                .is_some_and(|at| at.elapsed() < self.stale_after)
            {
                return;
            }
            seen.insert(id.to_string(), std::time::Instant::now());
        }
        let hours = self.stale_after.as_secs_f64() / 3600.0;
        self.announce(vec![id.to_string()]).await;
        let text = format!(
            "{id} ({title}): open {hours}h, prepared but never planned; it stays open (nothing to re-prepare). Plan it or hand it to the PM."
        );
        self.send(
            ToTarget::External(crate::wake::ORCHESTRATOR.to_string()),
            &text,
        )
        .await;
    }

    async fn tell_stale(&self, id: &str, title: &str, reason: &str) {
        let readier = self
            .readied_by
            .lock()
            .expect("readied_by lock")
            .remove(id)
            .or_else(|| self.tasks.get_task(id).map(|t| t.created_by));
        let text = format!("{id} ({title}): {reason}");
        let orchestrator = ToTarget::External(crate::wake::ORCHESTRATOR.to_string());
        let mut to = vec![orchestrator];
        if let Some(r) = readier {
            let target = match r.as_str() {
                "human" => Some(ToTarget::Human),
                r if r.starts_with("external:") => Some(ToTarget::External(r.to_string())),
                r => {
                    let name = r.strip_prefix("agent:").unwrap_or(r);
                    self.store
                        .get_agent(name)
                        .await
                        .ok()
                        .flatten()
                        .map(|a| ToTarget::Agent(a.id))
                }
            };
            if let Some(t) = target
                && !to.contains(&t)
            {
                to.push(t);
            }
        }
        for t in to {
            self.send(t, &text).await;
        }
    }
}
