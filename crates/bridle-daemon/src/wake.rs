//! The orchestrator's wake conditions and the `wait-for-wake` long poll
//! (docs/design/agent-host/orchestrator-supervision.md, section 5), moved here from
//! `scripts/orchestrator-watch.sh`.
//!
//! Most conditions are facts in the event log (an exit, a message, a failed CI run, a budget
//! hold), so the persisted cursor is "events up to here have been delivered": a daemon restart
//! re-derives whatever was queued and undelivered. The rest are states (idle, usage) kept in
//! memory; each fires once per crossing. `main` moving is not a wake: it needs no decision.

use std::collections::HashSet;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use bridle_api::types::{AgentState, Event, EventQuery, MessageKind, WakeReason, event_kind};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use tokio::sync::{Mutex, Notify};

use crate::store::Store;

/// The wake cursor: the seq of the last event scanned when wakes were last delivered.
const CURSOR_KEY: &str = "orchestrator_wake_cursor";
/// The principal the orchestrator's CLI calls act as, and the recipient of its messages.
pub const ORCHESTRATOR: &str = "external:orchestrator";
const ALL_IDLE_AFTER: i64 = 15 * 60;
const FIVE_HOUR_WAKE: f64 = 0.93;
const SEVEN_DAY_WAKE: f64 = 0.85;
/// A poll nothing wakes is answered empty after this long.
pub const POLL_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Who is waiting: a `wait-for-wake` request open now, or one that closed within the grace.
pub struct Waiters {
    started: DateTime<Utc>,
    state: StdMutex<WaiterState>,
}

#[derive(Default)]
struct WaiterState {
    open: u32,
    last_closed: Option<DateTime<Utc>>,
}

/// Held for as long as a wake request is open; dropping it (also when the client hangs up)
/// starts the grace.
pub struct WaiterGuard(Arc<Waiters>);

impl Drop for WaiterGuard {
    fn drop(&mut self) {
        let mut st = self.0.state.lock().expect("waiters lock");
        st.open -= 1;
        st.last_closed = Some(Utc::now());
    }
}

impl Waiters {
    pub fn new(started: DateTime<Utc>) -> Arc<Self> {
        Arc::new(Waiters {
            started,
            state: Default::default(),
        })
    }

    pub fn opened(self: &Arc<Self>) -> WaiterGuard {
        self.state.lock().expect("waiters lock").open += 1;
        WaiterGuard(self.clone())
    }

    /// Since when nobody has been waiting, if that is longer than `grace`. `floor` is when
    /// there could first have been a waiter (the session's launch): a session that only just
    /// started hasn't had time to start its command.
    pub fn absent_since(
        &self,
        now: DateTime<Utc>,
        grace: Duration,
        floor: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        let st = self.state.lock().expect("waiters lock");
        if st.open > 0 {
            return None;
        }
        let since = st.last_closed.unwrap_or(self.started).max(floor);
        (now - since > chrono::Duration::from_std(grace).unwrap_or(chrono::Duration::MAX))
            .then_some(since)
    }
}

#[derive(Default)]
struct State {
    /// Events up to here have been looked at; `None` until the first tick.
    scanned: Option<i64>,
    pending: Vec<WakeReason>,
    idle_since: Option<DateTime<Utc>>,
    idle_fired: bool,
    usage_fired: HashSet<String>,
}

pub struct Wakes {
    store: Store,
    state: Mutex<State>,
    notify: Notify,
}

impl Wakes {
    pub fn new(store: Store) -> Arc<Self> {
        Arc::new(Wakes {
            store,
            state: Default::default(),
            notify: Notify::new(),
        })
    }

    /// Looks for new wake conditions and queues them.
    pub async fn tick(&self, now: DateTime<Utc>) {
        let mut st = self.state.lock().await;
        if let Err(e) = self.scan(&mut st, now).await {
            tracing::warn!(error = %e, "orchestrator wake scan failed; retrying next tick");
        }
        if !st.pending.is_empty() {
            self.notify.notify_one();
        }
    }

    async fn scan(
        &self,
        st: &mut State,
        now: DateTime<Utc>,
    ) -> Result<(), crate::store::StoreError> {
        let mut since = match st.scanned {
            Some(s) => s,
            None => {
                // First ever run: start at the tail rather than replaying the history.
                let cursor = match self.store.get_meta(CURSOR_KEY).await? {
                    Some(v) => v.parse().unwrap_or(0),
                    None => {
                        let tail = self.tail().await?;
                        self.store.swap_meta(CURSOR_KEY, &tail.to_string()).await?;
                        tail
                    }
                };
                st.scanned = Some(cursor);
                cursor
            }
        };
        loop {
            let events = self
                .store
                .list_events(EventQuery {
                    since: Some(since),
                    agent: None,
                    kind: None,
                    limit: Some(500),
                })
                .await?;
            let Some(last) = events.last().map(|e| e.seq) else {
                break;
            };
            for ev in &events {
                if let Some(w) = self.wake_for_event(ev).await {
                    st.pending.push(w);
                }
            }
            since = last;
            st.scanned = Some(last);
        }

        self.check_idle(st, now).await?;
        self.check_usage(st).await?;
        Ok(())
    }

    async fn tail(&self) -> Result<i64, crate::store::StoreError> {
        let recent = self
            .store
            .list_events(EventQuery {
                limit: Some(1),
                ..Default::default()
            })
            .await?;
        Ok(recent.last().map_or(0, |e| e.seq))
    }

    async fn wake_for_event(&self, ev: &Event) -> Option<WakeReason> {
        let d = &ev.data;
        let agent = ev.agent.as_deref().unwrap_or("?");
        let (reason, text) = match ev.kind.as_str() {
            event_kind::AGENT_EXITED => {
                let why = d["reason"].as_str().unwrap_or("");
                if why == "stdin_closed" || why == "budget_paused" {
                    return None;
                }
                ("agent_exited", format!("agent {agent} exited ({why})"))
            }
            event_kind::AGENT_STATE if d["to"] == AgentState::Crashed.as_str() => {
                ("agent_crashed", format!("agent {agent} crashed"))
            }
            event_kind::AGENT_STALLED => ("agent_stalled", format!("agent {agent} stalled")),
            event_kind::CI_COMPLETED if d["conclusion"] == "failure" => (
                "ci_failed",
                format!(
                    "CI failed on {}: {}",
                    d["sha"].as_str().unwrap_or("?"),
                    d["url"].as_str().unwrap_or("no url")
                ),
            ),
            event_kind::BUDGET_STATE if d["to"] != "normal" => (
                "budget_hold",
                format!(
                    "budget governor: {} -> {}",
                    d["from"].as_str().unwrap_or("?"),
                    d["to"].as_str().unwrap_or("?")
                ),
            ),
            event_kind::TASK_CREATED if d["kind"] == "incident" => {
                let task_id = d["task"].as_str().unwrap_or("?");
                ("incident_created", format!("incident {task_id} created"))
            }
            event_kind::MESSAGE_SENT => return self.wake_for_message(ev).await,
            _ => return None,
        };
        Some(WakeReason {
            reason: reason.to_string(),
            text,
            detail: serde_json::to_value(ev).unwrap_or(Value::Null),
        })
    }

    /// A message to the orchestrator, or a question to the human, by the event's seq: no
    /// dependence on anyone marking anything read.
    async fn wake_for_message(&self, ev: &Event) -> Option<WakeReason> {
        let to = ev.data["to"].as_str()?;
        if to != ORCHESTRATOR && to != "human" {
            return None;
        }
        let msg = self
            .store
            .get_message(ev.data["message"].as_str()?)
            .await
            .ok()??;
        let (reason, text) = if to == ORCHESTRATOR {
            ("message", format!("message {} from {}", msg.id, msg.from))
        } else if msg.kind == MessageKind::Question {
            (
                "question",
                format!("question {} to the human from {}", msg.id, msg.from),
            )
        } else {
            return None;
        };
        Some(WakeReason {
            reason: reason.to_string(),
            text,
            detail: serde_json::to_value(&msg).unwrap_or(Value::Null),
        })
    }

    async fn check_idle(
        &self,
        st: &mut State,
        now: DateTime<Utc>,
    ) -> Result<(), crate::store::StoreError> {
        let busy = self
            .store
            .list_agents(false)
            .await?
            .iter()
            .any(|a| matches!(a.state, AgentState::Working | AgentState::Starting));
        if busy {
            st.idle_since = None;
            st.idle_fired = false;
            return Ok(());
        }
        let since = *st.idle_since.get_or_insert(now);
        if !st.idle_fired && (now - since).num_seconds() >= ALL_IDLE_AFTER {
            st.idle_fired = true;
            st.pending.push(WakeReason {
                reason: "all_idle".to_string(),
                text: "all agents have been idle for 15 minutes".to_string(),
                detail: json!({ "since": since }),
            });
        }
        Ok(())
    }

    async fn check_usage(&self, st: &mut State) -> Result<(), crate::store::StoreError> {
        for rl in self.store.rate_limits().await? {
            let limit = match rl.window.as_str() {
                "five_hour" => FIVE_HOUR_WAKE,
                "seven_day" => SEVEN_DAY_WAKE,
                _ => continue,
            };
            let util = rl.utilization.unwrap_or(0.0);
            if util < limit {
                st.usage_fired.remove(&rl.window);
            } else if st.usage_fired.insert(rl.window.clone()) {
                st.pending.push(WakeReason {
                    reason: "usage".to_string(),
                    text: format!("{} usage at {:.0}%", rl.window, util * 100.0),
                    detail: serde_json::to_value(&rl).unwrap_or(Value::Null),
                });
            }
        }
        Ok(())
    }

    /// Queues a wake the supervisor raised (a context or uptime note); it is delivered like the
    /// rest, and never lost to a waiter that isn't there.
    pub async fn push(&self, wake: WakeReason) {
        self.state.lock().await.pending.push(wake);
        self.notify.notify_one();
    }

    /// Takes every queued wake and marks them delivered.
    pub async fn take(&self) -> Vec<WakeReason> {
        let mut st = self.state.lock().await;
        if st.pending.is_empty() {
            return Vec::new();
        }
        if let Some(seq) = st.scanned
            && let Err(e) = self.store.swap_meta(CURSOR_KEY, &seq.to_string()).await
        {
            tracing::warn!(error = %e, "saving the orchestrator wake cursor failed");
        }
        std::mem::take(&mut st.pending)
    }

    /// The long poll: the queued wakes, at once if there are any, else as soon as one comes up;
    /// empty after `timeout`.
    pub async fn wait(&self, timeout: Duration) -> Vec<WakeReason> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let wakes = self.take().await;
            if !wakes.is_empty() {
                return wakes;
            }
            tokio::select! {
                _ = self.notify.notified() => {}
                _ = tokio::time::sleep_until(deadline) => return Vec::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bridle_api::types::{RateLimit, When};

    use super::*;
    use crate::store::{NewMessage, RecipientKind};

    struct Rig {
        store: Store,
        wakes: Arc<Wakes>,
        repo: tempfile::TempDir,
        _db: tempfile::TempDir,
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git")
            .success();
        assert!(ok, "git {args:?}");
    }

    async fn rig() -> Rig {
        let db = tempfile::tempdir().expect("tempdir");
        let store = Store::open(db.path().join("bridle.db"))
            .await
            .expect("store");
        let repo = tempfile::tempdir().expect("tempdir");
        git(repo.path(), &["init", "-q", "-b", "main"]);
        git(repo.path(), &["commit", "-q", "--allow-empty", "-m", "one"]);
        let wakes = Wakes::new(store.clone());
        // The first tick sets the cursor at the tail.
        wakes.tick(Utc::now()).await;
        Rig {
            store,
            wakes,
            repo,
            _db: db,
        }
    }

    impl Rig {
        async fn event(&self, kind: &str, agent: Option<&str>, data: Value) {
            self.store
                .append_event(kind, "system".into(), agent.map(String::from), data)
                .await
                .expect("event");
        }

        async fn message(&self, to: &str, kind: MessageKind) {
            let m = self
                .store
                .insert_message(NewMessage {
                    from: "human".into(),
                    to: to.into(),
                    to_kind: if to == "human" {
                        RecipientKind::Human
                    } else {
                        RecipientKind::External
                    },
                    kind,
                    body: "hi".into(),
                    reply_to: None,
                    when: When::Now,
                    state: bridle_api::types::MessageState::Pending,
                })
                .await
                .expect("message");
            self.event(
                event_kind::MESSAGE_SENT,
                None,
                json!({"message": m.id, "to": to}),
            )
            .await;
        }

        async fn reasons(&self, now: DateTime<Utc>) -> Vec<String> {
            self.wakes.tick(now).await;
            self.wakes
                .take()
                .await
                .into_iter()
                .map(|w| w.reason)
                .collect()
        }
    }

    #[tokio::test]
    async fn agent_conditions_fire_once() {
        let r = rig().await;
        r.event(
            event_kind::AGENT_EXITED,
            Some("a1"),
            json!({"reason": "exit"}),
        )
        .await;
        r.event(
            event_kind::AGENT_EXITED,
            Some("a2"),
            json!({"reason": "stdin_closed"}),
        )
        .await;
        r.event(
            event_kind::AGENT_STATE,
            Some("a3"),
            json!({"from": "working", "to": "crashed"}),
        )
        .await;
        r.event(
            event_kind::AGENT_STATE,
            Some("a3"),
            json!({"from": "idle", "to": "working"}),
        )
        .await;
        r.event(event_kind::AGENT_STALLED, Some("a4"), json!({}))
            .await;
        assert_eq!(
            r.reasons(Utc::now()).await,
            ["agent_exited", "agent_crashed", "agent_stalled"]
        );
        assert!(r.reasons(Utc::now()).await.is_empty());
    }

    #[tokio::test]
    async fn messages_questions_ci_and_holds() {
        let r = rig().await;
        r.message(ORCHESTRATOR, MessageKind::Note).await;
        r.message("human", MessageKind::Question).await;
        r.message("human", MessageKind::Note).await;
        r.event(
            event_kind::CI_COMPLETED,
            None,
            json!({"sha": "abc", "conclusion": "success"}),
        )
        .await;
        r.event(
            event_kind::CI_COMPLETED,
            None,
            json!({"sha": "def", "conclusion": "failure"}),
        )
        .await;
        r.event(
            event_kind::BUDGET_STATE,
            None,
            json!({"from": "normal", "to": "holding"}),
        )
        .await;
        r.event(
            event_kind::BUDGET_STATE,
            None,
            json!({"from": "holding", "to": "normal"}),
        )
        .await;
        assert_eq!(
            r.reasons(Utc::now()).await,
            ["message", "question", "ci_failed", "budget_hold"]
        );
        assert!(r.reasons(Utc::now()).await.is_empty());
    }

    #[tokio::test]
    async fn queued_wakes_survive_and_return_at_once() {
        let r = rig().await;
        r.event(event_kind::AGENT_STALLED, Some("a"), json!({}))
            .await;
        r.wakes.tick(Utc::now()).await;
        // Nobody is waiting yet; the poll that arrives later returns at once.
        let got = tokio::time::timeout(Duration::from_secs(1), r.wakes.wait(POLL_TIMEOUT))
            .await
            .expect("returned at once");
        assert_eq!(got.len(), 1);
        // And an empty poll times out empty.
        assert!(r.wakes.wait(Duration::from_millis(20)).await.is_empty());
    }

    #[tokio::test]
    async fn a_waiting_poll_is_woken_by_the_next_tick() {
        let r = rig().await;
        let w = r.wakes.clone();
        let poll = tokio::spawn(async move { w.wait(POLL_TIMEOUT).await });
        tokio::time::sleep(Duration::from_millis(20)).await;
        r.event(event_kind::AGENT_STALLED, Some("a"), json!({}))
            .await;
        r.wakes.tick(Utc::now()).await;
        let got = tokio::time::timeout(Duration::from_secs(1), poll)
            .await
            .expect("woken")
            .expect("join");
        assert_eq!(got.len(), 1);
    }

    #[tokio::test]
    async fn the_cursor_is_stored_only_on_delivery() {
        let r = rig().await;
        r.event(event_kind::AGENT_STALLED, Some("a"), json!({}))
            .await;
        r.wakes.tick(Utc::now()).await;
        // A restart before delivery re-derives the wake from the events.
        let again = Wakes::new(r.store.clone());
        again.tick(Utc::now()).await;
        assert_eq!(again.take().await.len(), 1);
        // After delivery a restart doesn't repeat it.
        let after = Wakes::new(r.store.clone());
        after.tick(Utc::now()).await;
        assert!(after.take().await.is_empty());
    }

    #[tokio::test]
    async fn all_idle_fires_once_after_15_minutes() {
        let r = rig().await;
        let t0 = Utc::now();
        assert!(r.reasons(t0).await.is_empty());
        assert!(
            r.reasons(t0 + chrono::Duration::minutes(14))
                .await
                .is_empty()
        );
        assert_eq!(
            r.reasons(t0 + chrono::Duration::minutes(16)).await,
            ["all_idle"]
        );
        assert!(
            r.reasons(t0 + chrono::Duration::minutes(40))
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn usage_fires_once_per_crossing() {
        let r = rig().await;
        let set = |u: f64| RateLimit {
            window: "five_hour".into(),
            status: None,
            utilization: Some(u),
            resets_at: None,
            observed_at: Utc::now(),
        };
        r.store.upsert_rate_limit(set(0.92)).await.expect("rl");
        assert!(r.reasons(Utc::now()).await.is_empty());
        r.store.upsert_rate_limit(set(0.94)).await.expect("rl");
        assert_eq!(r.reasons(Utc::now()).await, ["usage"]);
        r.store.upsert_rate_limit(set(0.96)).await.expect("rl");
        assert!(r.reasons(Utc::now()).await.is_empty());
        r.store.upsert_rate_limit(set(0.1)).await.expect("rl");
        r.reasons(Utc::now()).await;
        r.store.upsert_rate_limit(set(0.95)).await.expect("rl");
        assert_eq!(r.reasons(Utc::now()).await, ["usage"]);
    }

    #[tokio::test]
    async fn main_moving_alone_does_not_wake() {
        let r = rig().await;
        assert!(r.reasons(Utc::now()).await.is_empty());
        git(
            r.repo.path(),
            &["commit", "-q", "--allow-empty", "-m", "two"],
        );
        assert!(r.reasons(Utc::now()).await.is_empty());
    }

    #[tokio::test]
    async fn incident_created_wakes_but_feature_task_does_not() {
        let r = rig().await;
        // Feature task creation should not wake
        r.event(
            event_kind::TASK_CREATED,
            None,
            json!({"task": "br-1234", "kind": "feature", "state": "open"}),
        )
        .await;
        assert!(r.reasons(Utc::now()).await.is_empty());

        // Incident task creation should wake
        r.event(
            event_kind::TASK_CREATED,
            None,
            json!({"task": "br-5678", "kind": "incident", "state": "open"}),
        )
        .await;
        assert_eq!(r.reasons(Utc::now()).await, ["incident_created"]);
    }

    #[test]
    fn waiter_grace() {
        let t0 = Utc::now();
        let grace = Duration::from_secs(120);
        let w = Waiters::new(t0);
        let later = |s: i64| t0 + chrono::Duration::seconds(s);
        // Never waited: measured from the start, or the session's launch if later.
        assert_eq!(w.absent_since(later(60), grace, t0), None);
        assert_eq!(w.absent_since(later(200), grace, t0), Some(t0));
        assert_eq!(w.absent_since(later(200), grace, later(150)), None);
        // An open request is a waiter however long it lasts; closing it starts the grace.
        let g = w.opened();
        assert_eq!(w.absent_since(later(10_000), grace, t0), None);
        drop(g);
        assert_eq!(w.absent_since(Utc::now(), grace, t0), None);
        let gone = Utc::now() + chrono::Duration::seconds(300);
        assert!(w.absent_since(gone, grace, t0).is_some());
    }
}
