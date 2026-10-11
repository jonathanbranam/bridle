//! The orchestrator's wake conditions and the `wait-for-wake` long poll
//! (docs/design/agent-host/orchestrator-supervision.md, section 5), moved here from
//! `scripts/orchestrator-watch.sh`.
//!
//! Most conditions are facts in the event log (an exit, a message, a failed CI run, a budget
//! hold), so the persisted cursor is "events up to here have been delivered": a daemon restart
//! re-derives whatever was queued and undelivered. The rest are states (usage) kept in
//! memory; each fires once per crossing. `main` moving is not a wake: it needs no decision.

use std::collections::HashSet;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use bridle_api::types::{AgentState, Event, EventQuery, MessageKind, WakeReason, event_kind};
use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::sync::{Mutex, Notify};

use crate::store::Store;

/// The wake cursor: the seq of the last event scanned when wakes were last delivered.
const CURSOR_KEY: &str = "orchestrator_wake_cursor";
/// The principal the orchestrator's CLI calls act as, and the recipient of its messages.
pub const ORCHESTRATOR: &str = "external:orchestrator";
const FIVE_HOUR_WAKE: f64 = 0.93;
const SEVEN_DAY_WAKE: f64 = 0.85;
/// A poll nothing wakes is answered empty after this long.
pub const POLL_TIMEOUT: Duration = Duration::from_secs(25 * 60);
/// The longest any wake request may ask to wait: 1 h 55 min, under Claude Code's 2-hour kill of
/// background tasks. Both wake routes clamp a caller's `timeout_secs` to it.
pub const MAX_WAKE_TIMEOUT: Duration = Duration::from_secs(6900);

/// A wake request's wait: `requested` seconds, or `default` when absent, never over
/// [`MAX_WAKE_TIMEOUT`]. Shared by both wake routes.
pub fn clamp_timeout(requested: Option<u64>, default: Duration) -> Duration {
    requested
        .map_or(default, Duration::from_secs)
        .min(MAX_WAKE_TIMEOUT)
}

/// Who is waiting: a `wait-for-wake` request open now, or one that closed within the grace.
pub struct Waiters {
    started: DateTime<Utc>,
    state: StdMutex<WaiterState>,
    /// Why the daemon is stopping, set by the first planned path to ask; `None` means a signal.
    stop_reason: StdMutex<Option<String>>,
    /// `Some(reason)` once the stop is announced: every open waiter answers with it.
    stop_tx: tokio::sync::watch::Sender<Option<String>>,
}

#[derive(Default)]
struct WaiterState {
    open: u32,
    /// `bridle agent wake` requests open now; not part of the orchestrator's presence check.
    principal_open: Vec<PrincipalWait>,
    /// Messages a `bridle agent wake` printed but whose session hasn't run a wake or inbox
    /// since: (target, message id, session). In memory only: after a restart they are simply
    /// unread, and so offered again.
    unacked: Vec<(String, String, Option<String>)>,
    next_wait_id: u64,
    last_closed: Option<DateTime<Utc>>,
    /// When a poll last answered with wakes (not an empty timeout); in memory only.
    last_delivered: Option<DateTime<Utc>>,
}

/// One open `bridle agent wake`: who it waits as, which session started it, and the way to end it.
struct PrincipalWait {
    id: u64,
    target: String,
    session: Option<String>,
    end: tokio::sync::oneshot::Sender<()>,
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

/// Held for as long as a `bridle agent wake` request is open.
pub struct PrincipalGuard(Arc<Waiters>, u64);

impl Drop for PrincipalGuard {
    fn drop(&mut self) {
        let id = self.1;
        self.0
            .state
            .lock()
            .expect("waiters lock")
            .principal_open
            .retain(|w| w.id != id);
    }
}

impl Waiters {
    pub fn new(started: DateTime<Utc>) -> Arc<Self> {
        Arc::new(Waiters {
            started,
            state: Default::default(),
            stop_reason: Default::default(),
            stop_tx: tokio::sync::watch::channel(None).0,
        })
    }

    /// Registers a `bridle agent wake` for `target`. A wait from the same `session` that is
    /// still open is ended (its receiver resolves): replacement is per session, not per identity,
    /// because identities are shared; a wait with no session replaces nothing. The receiver
    /// resolves when this wait is itself ended, by a newer wait or by [`Waiters::end_principal_waits`].
    pub fn principal_opened(
        self: &Arc<Self>,
        target: &str,
        session: Option<&str>,
    ) -> (PrincipalGuard, tokio::sync::oneshot::Receiver<()>) {
        let mut st = self.state.lock().expect("waiters lock");
        if let Some(s) = session {
            let (old, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut st.principal_open)
                .into_iter()
                .partition(|w| w.session.as_deref() == Some(s));
            st.principal_open = keep;
            for w in old {
                let _ = w.end.send(());
            }
        }
        st.next_wait_id += 1;
        let id = st.next_wait_id;
        let (end, rx) = tokio::sync::oneshot::channel();
        st.principal_open.push(PrincipalWait {
            id,
            target: target.to_string(),
            session: session.map(str::to_string),
            end,
        });
        (PrincipalGuard(self.clone(), id), rx)
    }

    /// Ends open `bridle agent wake` waits: those of `session` when given, else those waiting as
    /// `target`. Returns how many it ended.
    pub fn end_principal_waits(&self, target: &str, session: Option<&str>) -> usize {
        let mut st = self.state.lock().expect("waiters lock");
        let (hit, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut st.principal_open)
            .into_iter()
            .partition(|w| match session {
                Some(s) => w.session.as_deref() == Some(s),
                None => w.target == target,
            });
        st.principal_open = keep;
        let n = hit.len();
        for w in hit {
            let _ = w.end.send(());
        }
        n
    }

    /// Notes that `ids` were handed to a wait of `session` as `target` and await its next call.
    pub fn handed_over(&self, target: &str, session: Option<&str>, ids: &[String]) {
        let mut st = self.state.lock().expect("waiters lock");
        for id in ids {
            st.unacked
                .retain(|(t, i, s)| !(t == target && i == id && s.as_deref() == session));
            st.unacked
                .push((target.to_string(), id.clone(), session.map(str::to_string)));
        }
    }

    /// Takes the messages `session` was handed as `target` and hasn't acknowledged: its next
    /// wake is the acknowledgement. Other sessions' hand-overs stay, so those are offered again.
    pub fn take_unacked(&self, target: &str, session: Option<&str>) -> Vec<String> {
        let mut st = self.state.lock().expect("waiters lock");
        let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut st.unacked)
            .into_iter()
            .partition(|(t, _, s)| t == target && s.as_deref() == session);
        st.unacked = rest;
        mine.into_iter().map(|(_, id, _)| id).collect()
    }

    /// Forgets `ids`: they were read some other way (the inbox), nothing is left to acknowledge.
    pub fn forget_unacked(&self, ids: &[String]) {
        self.state
            .lock()
            .expect("waiters lock")
            .unacked
            .retain(|(_, i, _)| !ids.contains(i));
    }

    /// Records why the daemon is about to stop; the first caller wins.
    pub fn set_stop_reason(&self, reason: String) {
        self.stop_reason
            .lock()
            .expect("stop reason lock")
            .get_or_insert(reason);
    }

    /// Ends every open waiter with the stop reason (`default` when no planned path gave one).
    /// Returns the reason and how many waiters were open; the count is taken before they are
    /// told, so none has left yet.
    pub fn announce_stop(&self, default: &str) -> (String, u32) {
        let reason = self
            .stop_reason
            .lock()
            .expect("stop reason lock")
            .get_or_insert_with(|| default.to_string())
            .clone();
        let st = self.state.lock().expect("waiters lock");
        let n = st.open + st.principal_open.len() as u32;
        drop(st);
        self.stop_tx.send_replace(Some(reason.clone()));
        (reason, n)
    }

    /// Resolves with the reason once the stop is announced.
    pub async fn stopping(&self) -> String {
        let mut rx = self.stop_tx.subscribe();
        // The guard `wait_for` returns isn't Send, so it is dropped before any later await.
        let reason = rx.wait_for(|v| v.is_some()).await.ok().map(|v| v.clone());
        match reason {
            Some(r) => r.unwrap_or_default(),
            None => std::future::pending().await,
        }
    }

    pub fn opened(self: &Arc<Self>) -> WaiterGuard {
        self.state.lock().expect("waiters lock").open += 1;
        WaiterGuard(self.clone())
    }

    /// A poll answered with wakes just now.
    pub fn delivered(&self, at: DateTime<Utc>) {
        self.state.lock().expect("waiters lock").last_delivered = Some(at);
    }

    /// Whether a request is open now, and when wakes were last delivered.
    pub fn snapshot(&self) -> (bool, Option<DateTime<Utc>>) {
        let st = self.state.lock().expect("waiters lock");
        (st.open > 0, st.last_delivered)
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
    /// Wakes queued since the last tick that haven't been sent home yet.
    unforwarded: Vec<WakeReason>,
    usage_fired: HashSet<String>,
}

pub struct Wakes {
    store: Store,
    state: Mutex<State>,
    notify: Notify,
    /// Where wakes are forwarded as system messages when the orchestrator's home is another
    /// daemon; set once the daemon has built its outbox.
    outbox: std::sync::OnceLock<crate::outbox::Outbox>,
}

impl Wakes {
    pub fn new(store: Store) -> Arc<Self> {
        Arc::new(Wakes {
            store,
            state: Default::default(),
            notify: Notify::new(),
            outbox: Default::default(),
        })
    }

    pub fn set_outbox(&self, outbox: crate::outbox::Outbox) {
        let _ = self.outbox.set(outbox);
    }

    /// Sends a wake to every home daemon of a visiting orchestrator as a `system` message, so the
    /// single waiter there wakes for it (3haz). One outbox row per wake and home. This daemon's
    /// own waiter is still woken by the wake itself: that path stays until the message path is
    /// verified. Messages and questions are messages already, and a stop is this daemon's own.
    async fn forward_home(&self, wakes: &[WakeReason]) {
        let Some(outbox) = self.outbox.get() else {
            return;
        };
        let news: Vec<&WakeReason> = wakes
            .iter()
            .filter(|w| {
                !matches!(
                    w.reason.as_str(),
                    "message" | "question" | bridle_api::types::DAEMON_STOPPING_WAKE
                )
            })
            .collect();
        if news.is_empty() {
            return;
        }
        let homes = match self.store.visitor_homes_named("orchestrator").await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(error = %e, "looking up the orchestrator's home failed");
                return;
            }
        };
        let machine = outbox.machine_name();
        let at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        for home in homes {
            if outbox.check_destination(&home).is_err() {
                continue;
            }
            for w in &news {
                let row = crate::store::OutboxRow {
                    id: String::new(),
                    project: home.clone(),
                    from: format!("system@{machine}"),
                    to: ORCHESTRATOR.to_string(),
                    kind: MessageKind::System,
                    body: format!("[{}] {} (as of {at})", w.reason, w.text),
                    reply_to: None,
                    when: bridle_api::types::When::Now,
                    attempts: 0,
                    last_error: None,
                };
                if let Err(e) = self.store.outbox_enqueue(row).await {
                    tracing::warn!(error = %e, "queueing a wake for the orchestrator's home failed");
                }
            }
            let outbox = outbox.clone();
            tokio::spawn(async move { outbox.flush(&home).await });
        }
    }

    /// Looks for new wake conditions and queues them.
    pub async fn tick(&self, _now: DateTime<Utc>) {
        let mut st = self.state.lock().await;
        if let Err(e) = self.scan(&mut st).await {
            tracing::warn!(error = %e, "orchestrator wake scan failed; retrying next tick");
        }
        if !st.pending.is_empty() {
            self.notify.notify_one();
        }
        let fresh = std::mem::take(&mut st.unforwarded);
        drop(st);
        self.forward_home(&fresh).await;
    }

    async fn scan(&self, st: &mut State) -> Result<(), crate::store::StoreError> {
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
                    message: None,
                    to: None,
                })
                .await?;
            let Some(last) = events.last().map(|e| e.seq) else {
                break;
            };
            for ev in &events {
                if let Some(w) = self.wake_for_event(ev).await {
                    st.unforwarded.push(w.clone());
                    st.pending.push(w);
                }
            }
            since = last;
            st.scanned = Some(last);
        }

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
                let wake = WakeReason {
                    reason: "usage".to_string(),
                    text: format!("{} usage at {:.0}%", rl.window, util * 100.0),
                    detail: serde_json::to_value(&rl).unwrap_or(Value::Null),
                };
                st.unforwarded.push(wake.clone());
                st.pending.push(wake);
            }
        }
        Ok(())
    }

    /// Queues a wake the supervisor raised (a context or uptime note); it is delivered like the
    /// rest, and never lost to a waiter that isn't there.
    pub async fn push(&self, wake: WakeReason) {
        self.forward_home(std::slice::from_ref(&wake)).await;
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
    use serde_json::json;

    use super::*;
    use crate::store::{NewMessage, RecipientKind};

    #[test]
    fn wake_timeouts_are_honoured_up_to_the_cap_and_clamped_above() {
        let min = |m: u64| Duration::from_secs(m * 60);
        assert_eq!(clamp_timeout(Some(90 * 60), POLL_TIMEOUT), min(90));
        assert_eq!(clamp_timeout(Some(6900), POLL_TIMEOUT), min(115));
        assert_eq!(clamp_timeout(Some(7200), POLL_TIMEOUT), min(115));
        assert_eq!(clamp_timeout(Some(u64::MAX), POLL_TIMEOUT), min(115));
        // Absent: the orchestrator's 25 minutes stay its default; an agent's default is the cap.
        assert_eq!(clamp_timeout(None, POLL_TIMEOUT), min(25));
        assert_eq!(clamp_timeout(None, MAX_WAKE_TIMEOUT), min(115));
    }

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
