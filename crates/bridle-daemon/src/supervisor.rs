//! The agent supervisor: spawns and drives headless `claude` processes,
//! delivers messages to them, and stops/resumes/removes them. See
//! docs/design/agent-host/agents.md and messages.md.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use bridle_api::types::{
    Agent, AgentState, ExitInfo, Message, MessageKind, MessageState, PrincipalId, PrincipalKind,
    SpawnRequest, Workdir, event_kind,
};
use bridle_claude::command::{ClaudeCommand, Session};
use bridle_claude::events::{ContentBlock, EventKind as ClaudeEventKind};
use bridle_claude::process::{AgentHandle, ExitOutcome};
use bridle_claude::transcript::Transcript;
use chrono::Utc;
use serde_json::{Value, json};
use tokio::sync::{Mutex as AsyncMutex, watch};
use uuid::Uuid;

use crate::config::{Config, Role};
use crate::containment::{self, Tracker};
use crate::events::Emitter;
use crate::paths::{Workspace, write_secret_file};
use crate::store::{NewAgent, NewMessage, Principal, Store, StoreError};
use crate::worktree::{self, WorktreeError};

const TRUNCATE_TEXT: usize = 2048;
const TRUNCATE_SUMMARY: usize = 120;
const INTERRUPT_TIMEOUT: Duration = Duration::from_secs(10);
const TERMINATE_GRACE: Duration = Duration::from_secs(3);
const SWEEP_GRACE: Duration = Duration::from_secs(2);
/// How long `spawn` waits for the process's first `system/init` (or its
/// exit) before answering anyway (docs/questions/open/v1-follow-ups-from-the-build-9c6e.md).
const SPAWN_READY_TIMEOUT: Duration = Duration::from_secs(8);
/// `result.subtype` when `--max-budget-usd` is spent (docs/spikes/02-budget-cap-findings.md).
const BUDGET_EXHAUSTED_SUBTYPE: &str = "error_max_budget_usd";
/// How long the turn-end `get_context_usage` probe waits before falling back
/// to `result.usage` (kc4v).
const CONTEXT_USAGE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("agent not running: {0}")]
    AgentNotRunning(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<StoreError> for SupervisorError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound(m) => SupervisorError::NotFound(m),
            StoreError::Conflict(m) => SupervisorError::Conflict(m),
            other => SupervisorError::Internal(other.to_string()),
        }
    }
}

impl From<WorktreeError> for SupervisorError {
    fn from(e: WorktreeError) -> Self {
        match e {
            WorktreeError::BranchExists { branch } => {
                SupervisorError::Conflict(format!("branch {branch:?} already exists"))
            }
            WorktreeError::InvalidName(n) => {
                SupervisorError::BadRequest(format!("invalid agent name {n:?}"))
            }
            other => SupervisorError::Internal(other.to_string()),
        }
    }
}

impl From<std::io::Error> for SupervisorError {
    fn from(e: std::io::Error) -> Self {
        SupervisorError::Internal(e.to_string())
    }
}

// ---------- runtime bookkeeping for a live agent process ----------

struct RuntimeState {
    tracker: Tracker,
    /// FIFO of (message id, exact text written to stdin), oldest first.
    fifo: VecDeque<(String, String)>,
    /// The session's cumulative cost as of the last `result`, per
    /// docs/design/agent-host/agents.md's cost-delta rule.
    last_cumulative: f64,
    /// The last turn number started (equals `agent.turns` until a turn is
    /// in flight, then `agent.turns + 1`).
    turn_n: u32,
    stall_notified: bool,
    /// This runtime's session id, set once at construction (htp6b): lets
    /// `mark_context_renew` tell a live crossing from a stale one — a
    /// `tick_context_check` snapshot naming an *old* session's tokens will
    /// carry that old session's id, which can't match here once `renew`
    /// has swapped in a new runtime for a new session, however this lookup
    /// races that swap.
    session_id: String,
    /// Set once this crossing of the `[context] wind_down_at` threshold has
    /// sent its "Context handoff:" notice (htp6b, mirrors
    /// `wind_down_pending`): the agent gets `context_renew_deadline` to
    /// finish a handoff turn on its own (checked at turn-end, alongside
    /// `wind_down_pending`) before `expire_context_renews` renews it
    /// regardless. `register_and_start` gives every fresh runtime —
    /// including the one `renew` itself creates — a clean `false`, so the
    /// next crossing (on a fresh session, after context_tokens resets)
    /// notifies again.
    context_renew_pending: bool,
    context_renew_deadline: Option<std::time::Instant>,
    current_state: AgentState,
    /// Whether any stdout line at all has been seen (system/init counts).
    saw_any_line: bool,
    /// Whether this process's Claude Code version has been checked; init
    /// repeats every turn but the version can't change within a process.
    version_checked: bool,
    /// Last time `touch_agent` actually wrote to the store, to throttle it
    /// to roughly once a second under a chatty agent.
    last_touch: std::time::Instant,
    /// Set once the governor's wind-down notice has been sent
    /// (usage-and-budget.md, The wind-down); `wind_down_deadline` is when
    /// the grace period runs out (or, for an escalation straight to
    /// `paused`, "now").
    wind_down_pending: bool,
    wind_down_deadline: Option<std::time::Instant>,
}

struct AgentRuntime {
    handle: AgentHandle,
    stop_requested: AtomicBool,
    /// Set when claude reported its `--max-budget-usd` spent; the agent is
    /// then stopped, and a resume grants a fresh allowance.
    budget_exhausted: AtomicBool,
    /// Set when the account-wide budget governor is stopping this agent
    /// (idle at once, or a notified working agent whose turn ended or grace
    /// expired); gives `agent.exited` the `budget_paused` reason, distinct
    /// from the per-agent `budget_exhausted` spend cap.
    budget_paused: AtomicBool,
    /// Set by [`AgentManager::stop_all`] before it calls `stop()`, so
    /// `classify_exit` can tell a shutdown-triggered stop from an ordinary
    /// `bridle stop`, and give it a reason that `resume_on_restart` treats
    /// like `lost` after the next start.
    shutdown_requested: AtomicBool,
    /// CAS-claimed the first time something actually calls `renew` for a
    /// pending context handoff (htp6b), so only one of the turn-end hook
    /// and `expire_context_renews`'s periodic sweep spawns it — plain
    /// `context_renew_pending` alone isn't enough: it stays true, read by
    /// both, until the fresh runtime `renew` creates replaces this one, so
    /// either the turn that caused the crossing and the handoff turn it
    /// triggers can both see it true and both spawn a renew, or two sweep
    /// ticks can both see the same deadline as due before the first renew
    /// finishes.
    context_renew_claimed: AtomicBool,
    state: AsyncMutex<RuntimeState>,
    exited: watch::Receiver<bool>,
    task: AsyncMutex<Option<tokio::task::JoinHandle<()>>>,
}

// ---------- the manager ----------

struct Inner {
    store: Store,
    workspace: Workspace,
    config: Config,
    claude_program: String,
    url: String,
    project: String,
    emitter: Emitter,
    runtimes: std::sync::Mutex<HashMap<String, Arc<AgentRuntime>>>,
    governor: crate::governor::GovernorHandle,
    /// `bridle budget max-workers`: a live cap replacing
    /// `[budget] max_workers` until cleared or the daemon restarts.
    /// usage-and-budget.md, Max-workers override.
    max_workers_override: std::sync::Mutex<Option<u32>>,
    /// Tasks filed since the manager was last told; see `note_task_filed`.
    task_wake: std::sync::Mutex<TaskWake>,
}

/// Coalescing state for the "new task filed" message to the manager.
#[derive(Default)]
struct TaskWake {
    last_sent: Option<std::time::Instant>,
    pending: Vec<(String, String)>,
    open_count: usize,
    flush_scheduled: bool,
}

/// At most one "task filed" message per manager per this long.
const TASK_WAKE_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);

#[derive(Clone)]
pub struct AgentManager(Arc<Inner>);

/// The role `max_workers` counts; the manager, PM and orchestrator don't.
pub(crate) const WORKER_ROLE: &str = "worker";

pub fn system_principal() -> Principal {
    Principal {
        id: "system".to_string(),
        kind: PrincipalKind::System,
    }
}

/// Wins the race to actually renew `rt`'s agent for its pending context
/// handoff (htp6b): `true` only for the first caller, so the turn-end hook
/// and `expire_context_renews`'s sweep — which can both see
/// `context_renew_pending` true for the same still-live runtime — don't
/// each spawn their own `renew`.
fn claim_context_renew(rt: &AgentRuntime) -> bool {
    !rt.context_renew_claimed.swap(true, Ordering::SeqCst)
}

/// The agent-row fields `register_and_start` needs to build a fresh
/// runtime, grouped so the function stays under clippy's argument limit.
struct StartingAgent<'a> {
    agent_id: &'a str,
    session_id: &'a str,
    turns_so_far: u32,
    cost_so_far: f64,
}

impl AgentManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: Store,
        workspace: Workspace,
        config: Config,
        claude_program: String,
        url: String,
        project: String,
        emitter: Emitter,
        governor: crate::governor::GovernorHandle,
    ) -> Self {
        AgentManager(Arc::new(Inner {
            store,
            workspace,
            config,
            claude_program,
            url,
            project,
            emitter,
            runtimes: std::sync::Mutex::new(HashMap::new()),
            governor,
            max_workers_override: std::sync::Mutex::new(None),
            task_wake: std::sync::Mutex::new(TaskWake::default()),
        }))
    }

    /// A project with no product manager has nobody to triage a new `open`
    /// task, so the running manager is told (at most once a minute, listing
    /// every task filed since). Does nothing when a PM is running or no
    /// manager is.
    pub async fn note_task_filed(&self, id: &str, title: &str, open_count: usize) {
        let Ok(agents) = self.0.store.list_agents(false).await else {
            return;
        };
        let running = |role: &str| {
            agents
                .iter()
                .any(|a| a.role == role && a.state.is_running())
        };
        if running("product-manager") || !running("manager") {
            return;
        }
        let delay = {
            let mut w = self.0.task_wake.lock().expect("task_wake mutex poisoned");
            w.pending.push((id.to_string(), title.to_string()));
            w.open_count = open_count;
            match w.last_sent.map(|t| t.elapsed()) {
                Some(e) if e < TASK_WAKE_WINDOW => {
                    if w.flush_scheduled {
                        return;
                    }
                    w.flush_scheduled = true;
                    TASK_WAKE_WINDOW - e
                }
                _ => std::time::Duration::ZERO,
            }
        };
        if delay.is_zero() {
            self.flush_task_wake().await;
        } else {
            let this = self.clone();
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                this.flush_task_wake().await;
            });
        }
    }

    async fn flush_task_wake(&self) {
        let (pending, open_count) = {
            let mut w = self.0.task_wake.lock().expect("task_wake mutex poisoned");
            w.flush_scheduled = false;
            w.last_sent = Some(std::time::Instant::now());
            (std::mem::take(&mut w.pending), w.open_count)
        };
        if pending.is_empty() {
            return;
        }
        let Ok(agents) = self.0.store.list_agents(false).await else {
            return;
        };
        let filed: Vec<String> = pending
            .iter()
            .map(|(id, title)| format!("task {id} filed: {title}"))
            .collect();
        let body = format!(
            "{}; open tasks: {open_count}. Plan it or queue it.",
            filed.join("; ")
        );
        for m in agents
            .iter()
            .filter(|a| a.role == "manager" && a.state.is_running())
        {
            let _ = self
                .send(
                    "system".to_string(),
                    ToTarget::Agent(m.id.clone()),
                    MessageKind::Note,
                    body.clone(),
                    bridle_api::types::When::Idle,
                    None,
                )
                .await;
        }
    }

    /// Any one live agent's control handle, for the governor's `get_usage`
    /// probe (usage-and-budget.md, Seeing the windows): cheaper than a
    /// dedicated probe process when an agent is already running.
    pub fn any_running_handle(&self) -> Option<bridle_claude::process::AgentHandle> {
        self.0
            .runtimes
            .lock()
            .expect("runtimes mutex poisoned")
            .values()
            .next()
            .map(|rt| rt.handle.clone())
    }

    /// The budget state that applies to a spawn/resume/message pinned to
    /// `model` (the worse of the default-scoped state and that model's own
    /// weekly window).
    fn budget_block_for_model(&self, model: &str) -> crate::governor::WindowBlock {
        self.0
            .governor
            .lock()
            .expect("governor mutex poisoned")
            .for_model(model)
    }

    /// Picks the model for a spawn/resume that didn't pin one explicitly:
    /// the first entry in the role's `[models]` list whose window is
    /// `Normal`, stepping down as a window gets tight (usage-and-budget.md,
    /// Model choice). Falls back to the list's first (strongest) entry when
    /// every candidate is blocked, so the caller's own budget check still
    /// refuses on that model's window rather than inventing a new failure
    /// mode — stepping down delays a wind-down, it never replaces one.
    fn choose_model(&self, role_name: &str, role: &Role) -> String {
        let candidates = self.0.config.models.candidates(role_name, role);
        let chosen = {
            let governor = self.0.governor.lock().expect("governor mutex poisoned");
            candidates
                .iter()
                .copied()
                .find(|m| governor.for_model(m).state == bridle_api::types::GovernorState::Normal)
        };
        chosen
            .or_else(|| candidates.first().copied())
            .map(str::to_string)
            .unwrap_or_else(|| role.model.clone())
    }

    fn refuse_if_holding(&self, model: &str) -> Result<(), SupervisorError> {
        let block = self.budget_block_for_model(model);
        if block.state == bridle_api::types::GovernorState::Normal {
            return Ok(());
        }
        let window = block.window.as_deref().unwrap_or("budget");
        let resets = block
            .resets_at
            .map(|r| r.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string());
        Err(SupervisorError::Conflict(format!(
            "budget governor is {} ({window}, resets {resets}); pass --ignore-budget once that's built, or wait",
            block.state
        )))
    }

    fn get_runtime(&self, id: &str) -> Option<Arc<AgentRuntime>> {
        self.0
            .runtimes
            .lock()
            .expect("runtimes mutex poisoned")
            .get(id)
            .cloned()
    }

    pub fn set_max_workers_override(&self, cap: Option<u32>) {
        *self
            .0
            .max_workers_override
            .lock()
            .expect("max_workers_override mutex poisoned") = cap;
    }

    pub fn max_workers_override(&self) -> Option<u32> {
        *self
            .0
            .max_workers_override
            .lock()
            .expect("max_workers_override mutex poisoned")
    }

    /// The cap in force: the live override, else the configured value.
    pub fn effective_max_workers(&self) -> u32 {
        self.max_workers_override()
            .unwrap_or(self.0.config.budget.max_workers)
    }

    /// Running agents in the `worker` role: what `max_workers` limits, both
    /// at spawn and in the governor's `maybe_resume`.
    pub async fn running_worker_count(&self) -> usize {
        let mut n = 0;
        for id in self.running_ids() {
            if let Ok(Some(a)) = self.0.store.get_agent(&id).await
                && a.role == WORKER_ROLE
            {
                n += 1;
            }
        }
        n
    }

    pub fn running_ids(&self) -> Vec<String> {
        self.0
            .runtimes
            .lock()
            .expect("runtimes mutex poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// A snapshot of every live runtime, for the background containment
    /// tracker loop.
    fn runtimes_snapshot(&self) -> Vec<(String, Arc<AgentRuntime>)> {
        self.0
            .runtimes
            .lock()
            .expect("runtimes mutex poisoned")
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Updates every live agent's containment tracker from one process
    /// snapshot (agents.md, Containment; called every `tracker_interval`).
    pub async fn tick_tracker(&self) {
        let snap = match tokio::task::spawn_blocking(containment::snapshot).await {
            Ok(Ok(s)) => s,
            _ => return,
        };
        for (_, rt) in self.runtimes_snapshot() {
            let mut st = rt.state.lock().await;
            st.tracker.update(&snap);
        }
    }

    /// Emits `agent.stalled` for any `working` agent silent past
    /// `config.stall_after`, once per silent stretch (agents.md, States).
    pub async fn tick_stall_check(&self) {
        let Ok(agents) = self.0.store.list_agents(false).await else {
            return;
        };
        let stall_after = chrono::Duration::from_std(self.0.config.stall_after)
            .unwrap_or_else(|_| chrono::Duration::zero());
        for a in agents {
            if a.state != AgentState::Working {
                continue;
            }
            let Some(last) = a.last_event_at.or(a.turn_started_at) else {
                continue;
            };
            if Utc::now() - last < stall_after {
                continue;
            }
            let Some(rt) = self.get_runtime(&a.id) else {
                continue;
            };
            let mut st = rt.state.lock().await;
            if st.stall_notified {
                continue;
            }
            st.stall_notified = true;
            drop(st);
            let _ = self
                .0
                .emitter
                .emit(
                    event_kind::AGENT_STALLED,
                    "system".to_string(),
                    Some(a.id),
                    json!({}),
                )
                .await;
        }
    }

    /// Sends `Context handoff:` to any agent whose `context_tokens` crosses
    /// its role's `[context] wind_down_at` threshold, once per crossing
    /// (htp6b: govern context size the way the budget governor governs
    /// usage — see `mark_context_renew`/`expire_context_renews`, mirroring
    /// `mark_wind_down`/`expire_wind_downs`). The renew itself is deferred:
    /// a handoff needs a whole turn (commit WIP, write a note) to run
    /// safely, not `stop_grace`'s 30s meant for an idle process noticing
    /// stdin EOF, so it fires either when that turn ends on its own (the
    /// `context_renew_pending` check at turn-end, alongside
    /// `wind_down_pending`) or, failing that, once `context.wind_down_grace`
    /// runs out.
    pub async fn tick_context_check(&self) {
        let Ok(agents) = self.0.store.list_agents(false).await else {
            return;
        };
        for a in agents {
            if !matches!(a.state, AgentState::Idle | AgentState::Working) {
                continue;
            }
            let Some(tokens) = a.context_tokens else {
                continue;
            };
            let threshold = self.0.config.context.wind_down_at.get(&a.role);
            if (tokens as f64) < threshold {
                continue;
            }
            let deadline = std::time::Instant::now() + self.0.config.context.wind_down_grace;
            if self
                .mark_context_renew(&a.id, &a.session_id, deadline)
                .await
                != Some(true)
            {
                continue;
            }

            let body = format!(
                "Context handoff: your context is at {tokens} tokens, past this role's \
                 {threshold:.0}-token wind-down threshold. Commit your work in progress \
                 to your branch, then send a short handoff note on where you are and \
                 what's next — you'll be renewed with a fresh context once this turn \
                 ends (or in {grace_secs}s regardless).",
                grace_secs = self.0.config.context.wind_down_grace.as_secs(),
            );
            let _ = self
                .send(
                    "system".to_string(),
                    ToTarget::Agent(a.id.clone()),
                    MessageKind::Note,
                    body,
                    bridle_api::types::When::Now,
                    None,
                )
                .await;
        }
        self.expire_context_renews().await;
    }

    /// Marks `id` as owing a context renew once its handoff turn ends or
    /// its grace deadline passes, whichever comes first (mirrors
    /// `mark_wind_down`). Returns `Some(true)` the first time for a
    /// crossing that's still live, `None` if it no longer is.
    ///
    /// `expected_session_id` guards against `tick_context_check`'s
    /// once-per-tick agent list going stale mid-iteration (30+ agents,
    /// each an await point): if an *earlier* agent's crossing already
    /// renewed a *different* agent by the time this one's turn comes up,
    /// that's unrelated and harmless, but if renew has *already happened
    /// for this same agent* — e.g. its handoff turn finished and renewed
    /// while this stale-snapshot iteration was still working through
    /// other agents — `get_runtime` below now returns the fresh runtime
    /// `renew` swapped in, whose `context_renew_pending` reads `false` and
    /// would otherwise look exactly like a genuine new crossing. Session
    /// id is checked *inside* the lock this function already takes, not
    /// via a separate fresh read beforehand (which would just narrow the
    /// race, not close it): the moment renew's `register_and_start`
    /// installs a new runtime with a new session id, any concurrent
    /// caller here — no matter how stale its snapshot — either observes
    /// that new runtime (id mismatch, bail out) or the old one it
    /// replaced (id matches, genuinely still current); there's no
    /// in-between state to race.
    async fn mark_context_renew(
        &self,
        id: &str,
        expected_session_id: &str,
        deadline: std::time::Instant,
    ) -> Option<bool> {
        let rt = self.get_runtime(id)?;
        let mut st = rt.state.lock().await;
        if st.session_id != expected_session_id {
            return None;
        }
        let first = !st.context_renew_pending;
        st.context_renew_pending = true;
        if first {
            st.context_renew_deadline = Some(deadline);
        }
        Some(first)
    }

    /// Renews every context-renew-pending agent whose grace deadline has
    /// passed (mirrors `expire_wind_downs`): the fallback for an agent
    /// that's idle when notified, or whose handoff turn runs past grace.
    async fn expire_context_renews(&self) {
        let now = std::time::Instant::now();
        let mut due = Vec::new();
        for (id, rt) in self.runtimes_snapshot() {
            let is_due = {
                let st = rt.state.lock().await;
                st.context_renew_pending && st.context_renew_deadline.is_some_and(|d| now >= d)
            };
            if is_due && claim_context_renew(&rt) {
                due.push(id);
            }
        }
        for id in due {
            let _ = self.renew(&id, &system_principal()).await;
        }
    }

    /// Stops every running agent concurrently, capped at `cap` in total
    /// (the shutdown sequence in agents.md, Stopping).
    pub async fn stop_all(&self, cap: Duration) {
        let ids = self.running_ids();
        let system = system_principal();
        let futs = ids.into_iter().map(|id| {
            let this = self.clone();
            let system = system.clone();
            async move {
                if let Some(rt) = this.get_runtime(&id) {
                    rt.shutdown_requested.store(true, Ordering::SeqCst);
                }
                let _ = this.stop(&id, false, &system).await;
            }
        });
        let _ = tokio::time::timeout(cap, futures::future::join_all(futs)).await;
    }

    // ---------- spawn ----------

    /// The config's view of a component list: unknown ids rejected, repeats dropped.
    pub fn normalize_components(&self, ids: &[String]) -> Result<Vec<String>, SupervisorError> {
        self.0
            .config
            .normalize_components(ids)
            .map_err(SupervisorError::BadRequest)
    }

    pub fn components_match(&self, named: &[String], filter: &str) -> bool {
        self.0.config.components_match(named, filter)
    }

    pub fn has_components(&self) -> bool {
        !self.0.config.components.is_empty()
    }

    pub async fn spawn(
        &self,
        req: SpawnRequest,
        principal: &Principal,
    ) -> Result<Agent, SupervisorError> {
        let role =
            self.0.config.roles.get(&req.role).cloned().ok_or_else(|| {
                SupervisorError::BadRequest(format!("unknown role {:?}", req.role))
            })?;
        let model = match &req.model {
            Some(m) => m.clone(),
            None => self.choose_model(&req.role, &role),
        };
        if !req.ignore_budget {
            self.refuse_if_holding(&model)?;
        }
        if req.role == WORKER_ROLE {
            let cap = self.effective_max_workers() as usize;
            let running = self.running_worker_count().await;
            if running >= cap {
                return Err(SupervisorError::Conflict(format!(
                    "{running} workers running, at max_workers {cap}; wait for one to finish or raise it with `bridle budget max-workers`"
                )));
            }
        }

        let name = match req.name {
            Some(n) => {
                worktree::validate_agent_name(&n)?;
                if self.0.store.get_agent(&n).await?.is_some() {
                    return Err(SupervisorError::Conflict(format!(
                        "agent name {n:?} already exists"
                    )));
                }
                n
            }
            None => self.generate_name(&req.role).await?,
        };

        let workdir = req.workdir.unwrap_or(match role.workdir {
            crate::config::Workdir::Worktree => Workdir::Worktree { base: None },
            crate::config::Workdir::Repo => Workdir::Repo,
        });

        let mut created_worktree: Option<(std::path::PathBuf, String)> = None;
        let (workdir_kind, cwd, worktree_path, branch) = match workdir {
            Workdir::Worktree { base } => {
                let path = self.0.workspace.worktree(&name);
                let branch = format!("bridle/{name}");
                let base_ref = base.unwrap_or_else(|| role.base.clone());
                worktree::add(&self.0.workspace.repo, &path, &branch, &base_ref).await?;
                if self.0.config.warm_target {
                    worktree::warm_target(&self.0.workspace.repo, &path).await;
                }
                created_worktree = Some((path.clone(), branch.clone()));
                worktree::copy_files(&self.0.workspace.repo, &path, &self.0.config.copy);
                if let Some(cmd) = &self.0.config.setup
                    && let Err(e) =
                        worktree::run_setup(&path, cmd, self.0.config.setup_timeout).await
                {
                    // Same cleanup as the later spawn-failure paths.
                    let _ = worktree::remove(&self.0.workspace.repo, &path, true).await;
                    let _ = worktree::delete_branch(&self.0.workspace.repo, &branch, true).await;
                    return Err(e.into());
                }
                ("worktree", path.clone(), Some(path), Some(branch))
            }
            Workdir::Repo => ("repo", self.0.workspace.repo.clone(), None, None),
            Workdir::Path { path } => {
                let path = std::path::PathBuf::from(path);
                if !path.is_dir() {
                    return Err(SupervisorError::BadRequest(format!(
                        "workdir path {} does not exist",
                        path.display()
                    )));
                }
                // One agent per branch: a directory inside another agent's
                // worktree is that agent's branch, running or stopped.
                let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
                for other in self.0.store.list_agents(true).await? {
                    let Some(wt) = other.worktree.as_deref() else {
                        continue;
                    };
                    let wt = std::path::Path::new(wt);
                    let wt = wt.canonicalize().unwrap_or_else(|_| wt.to_path_buf());
                    if canon.starts_with(&wt) {
                        let branch = other.branch.as_deref().unwrap_or("its branch");
                        return Err(SupervisorError::Conflict(format!(
                            "{} is agent {:?}'s worktree (branch {branch}); one agent per branch: `bridle renew {}` to continue it, or `bridle remove {}` first",
                            path.display(),
                            other.name,
                            other.name,
                            other.name
                        )));
                    }
                }
                ("path", path, None, None)
            }
        };

        let cleanup_worktree = |created: &Option<(std::path::PathBuf, String)>| {
            let workspace = self.0.workspace.clone();
            let created = created.clone();
            async move {
                if let Some((path, branch)) = created {
                    let _ = worktree::remove(&workspace.repo, &path, true).await;
                    let _ = worktree::delete_branch(&workspace.repo, &branch, true).await;
                }
            }
        };

        let session_id = Uuid::new_v4();
        let new_agent = NewAgent {
            name: name.clone(),
            role: req.role.clone(),
            model: model.clone(),
            session_id: session_id.to_string(),
            workdir_kind: workdir_kind.to_string(),
            cwd: cwd.to_string_lossy().into_owned(),
            worktree: worktree_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            branch: branch.clone(),
            created_by: principal.id.clone(),
            extra_allowed_tools: req.extra_allowed_tools.clone(),
            extra_env: req.extra_env.clone(),
            components: req.components.clone(),
        };
        let agent = match self.0.store.insert_agent(new_agent).await {
            Ok(a) => a,
            Err(e) => {
                cleanup_worktree(&created_worktree).await;
                return Err(e.into());
            }
        };

        let token = match self
            .0
            .store
            .create_agent_token(&agent.id, &agent.name)
            .await
        {
            Ok(t) => t,
            Err(e) => {
                cleanup_worktree(&created_worktree).await;
                let _ = self.0.store.delete_agent(&agent.id).await;
                return Err(e.into());
            }
        };
        if let Err(e) =
            std::fs::create_dir_all(self.0.workspace.agent_dir(&agent.id)).and_then(|_| {
                write_secret_file(&self.0.workspace.agent_dir(&agent.id).join("token"), &token)
            })
        {
            cleanup_worktree(&created_worktree).await;
            let _ = self
                .0
                .store
                .revoke_principal(&format!("agent:{}", agent.name))
                .await;
            let _ = self.0.store.delete_agent(&agent.id).await;
            return Err(e.into());
        }

        let prompt_text = crate::config::render_system_prompt(
            &req.role,
            &role,
            &self.0.workspace.repo,
            &self.0.config.branches,
            &self.0.config.commands,
            &agent.name,
            &cwd,
            branch.as_deref(),
        );
        let system_prompt_path = self.0.workspace.system_prompt(&agent.id);
        if let Err(e) = std::fs::write(&system_prompt_path, &prompt_text) {
            cleanup_worktree(&created_worktree).await;
            let _ = self
                .0
                .store
                .revoke_principal(&format!("agent:{}", agent.name))
                .await;
            let _ = self.0.store.delete_agent(&agent.id).await;
            return Err(e.into());
        }

        let mut cmd = ClaudeCommand::new(cwd.clone(), Session::New(session_id));
        cmd.program = self.0.claude_program.clone();
        cmd.model = Some(model.clone());
        cmd.effort = role.effort.clone();
        cmd.append_system_prompt_file = Some(system_prompt_path);
        cmd.permission_mode = Some(role.permission_mode.clone());
        cmd.allowed_tools = role.effective_allowed_tools();
        for tool in &req.extra_allowed_tools {
            if !cmd.allowed_tools.contains(tool) {
                cmd.allowed_tools.push(tool.clone());
            }
        }
        cmd.disallowed_tools = role.disallowed_tools.clone();
        cmd.stop_check = role.stop_check;
        cmd.name = Some(agent.name.clone());
        cmd.max_budget_usd = role.max_budget_usd;
        cmd.env = agent_env(
            &self.0.workspace,
            &self.0.url,
            &self.0.project,
            &agent.id,
            &agent.name,
            &token,
            &agent.components,
        );
        if !req.extra_env.is_empty() {
            tracing::info!(
                agent = %agent.id,
                keys = ?req.extra_env.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
                "spawn: extra env vars for this spawn only"
            );
            cmd.env.extend(req.extra_env.iter().cloned());
        }

        let transcript = match Transcript::open(
            &self.0.workspace.transcript(&agent.id),
            std::time::Instant::now(),
        ) {
            Ok(t) => t,
            Err(e) => {
                cleanup_worktree(&created_worktree).await;
                let _ = self
                    .0
                    .store
                    .revoke_principal(&format!("agent:{}", agent.name))
                    .await;
                let _ = self.0.store.delete_agent(&agent.id).await;
                return Err(e.into());
            }
        };

        let spawned = match bridle_claude::process::spawn(&cmd, transcript).await {
            Ok(s) => s,
            Err(e) => {
                cleanup_worktree(&created_worktree).await;
                let _ = self
                    .0
                    .store
                    .revoke_principal(&format!("agent:{}", agent.name))
                    .await;
                let _ = self.0.store.delete_agent(&agent.id).await;
                return Err(SupervisorError::Internal(format!("spawning claude: {e}")));
            }
        };

        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let runtime = self
            .register_and_start(
                StartingAgent {
                    agent_id: &agent.id,
                    session_id: &agent.session_id,
                    turns_so_far: agent.turns,
                    cost_so_far: agent.cost_usd_total,
                },
                spawned,
                principal,
                Some(ready_tx),
            )
            .await?;

        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_SPAWNED,
                principal.id.clone(),
                Some(agent.id.clone()),
                json!({"role": req.role, "model": model, "cwd": agent.cwd, "branch": branch}),
            )
            .await;

        // The agent's name, cwd and branch are always in its system prompt
        // now (render_system_prompt), so the first message no longer needs
        // to carry them: it's just the task, if there is one.
        let first_message = req.prompt.or(role.start_prompt.clone());
        let sent_first_message = first_message.is_some();
        if let Some(prompt) = first_message {
            self.send(
                principal.id.clone(),
                ToTarget::Agent(agent.id.clone()),
                MessageKind::Note,
                prompt,
                bridle_api::types::When::Now,
                None,
            )
            .await?;
        }

        // Wait for the process to prove it's actually up before answering:
        // its first `system/init` (a turn-start marker, re-emitted before
        // every turn, not a startup handshake — spike 01, S2) or its exit,
        // whichever comes first. Only worth doing when a first message was
        // just sent: that's what starts the turn where a bad model name or
        // expired auth would actually fail. An idle spawn (no prompt, no
        // `start_prompt`) starts no turn, so there is nothing to wait for
        // yet, and waiting out SPAWN_READY_TIMEOUT on every such spawn would
        // only add latency. If neither init nor an exit arrives within the
        // timeout, the agent is returned as-is (typically still `working`);
        // spawn's contract is "the daemon tried, here's the agent's current
        // state," not "the agent is ready" (docs/design/agent-host/agents.md,
        // Spawning).
        if sent_first_message {
            let mut exited = runtime.exited.clone();
            tokio::select! {
                _ = ready_rx => {}
                _ = exited.wait_for(|v| *v) => {}
                _ = tokio::time::sleep(SPAWN_READY_TIMEOUT) => {
                    tracing::warn!(agent = %agent.id, "spawn: timed out waiting for claude readiness");
                }
            }
        }

        self.0
            .store
            .get_agent(&agent.id)
            .await?
            .ok_or_else(|| SupervisorError::Internal("agent vanished after spawn".to_string()))
    }

    async fn generate_name(&self, role: &str) -> Result<String, SupervisorError> {
        for n in 1u32.. {
            let candidate = format!("{role}-{n}");
            if self.0.store.get_agent(&candidate).await?.is_none() {
                return Ok(candidate);
            }
        }
        unreachable!("u32 exhausted generating an agent name")
    }

    /// Records pid/start time, brings the agent to `idle`, builds the
    /// runtime and starts its event-processing task. `ready_tx`, if given,
    /// is fired the first time the process's `system/init` is seen (used by
    /// `spawn` to wait for readiness); resume passes `None`, since a resumed
    /// agent's caller doesn't wait on it.
    async fn register_and_start(
        &self,
        start: StartingAgent<'_>,
        spawned: bridle_claude::process::Spawned,
        _principal: &Principal,
        ready_tx: Option<tokio::sync::oneshot::Sender<()>>,
    ) -> Result<Arc<AgentRuntime>, SupervisorError> {
        let StartingAgent {
            agent_id,
            session_id,
            turns_so_far,
            cost_so_far,
        } = start;
        let pid = spawned.handle.pid();
        let start = tokio::task::spawn_blocking(move || containment::start_time(pid))
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        self.0
            .store
            .set_agent_process(agent_id, Some(pid), Some(start.clone()))
            .await?;
        self.0
            .store
            .set_agent_state(agent_id, AgentState::Idle)
            .await?;

        let (exited_tx, exited_rx) = watch::channel(false);
        let runtime = Arc::new(AgentRuntime {
            handle: spawned.handle,
            stop_requested: AtomicBool::new(false),
            budget_exhausted: AtomicBool::new(false),
            budget_paused: AtomicBool::new(false),
            shutdown_requested: AtomicBool::new(false),
            context_renew_claimed: AtomicBool::new(false),
            state: AsyncMutex::new(RuntimeState {
                tracker: Tracker::new(pid, start),
                fifo: VecDeque::new(),
                last_cumulative: cost_so_far,
                turn_n: turns_so_far,
                stall_notified: false,
                session_id: session_id.to_string(),
                context_renew_pending: false,
                context_renew_deadline: None,
                current_state: AgentState::Idle,
                saw_any_line: false,
                version_checked: false,
                last_touch: std::time::Instant::now(),
                wind_down_pending: false,
                wind_down_deadline: None,
            }),
            exited: exited_rx,
            task: AsyncMutex::new(None),
        });
        self.0
            .runtimes
            .lock()
            .expect("runtimes mutex poisoned")
            .insert(agent_id.to_string(), runtime.clone());

        let manager = self.clone();
        let id = agent_id.to_string();
        let task = tokio::spawn(manager.run_event_task(
            id,
            runtime.clone(),
            spawned.events,
            spawned.exit,
            exited_tx,
            ready_tx,
        ));
        *runtime.task.lock().await = Some(task);
        Ok(runtime)
    }

    // ---------- the per-agent event-processing task ----------

    async fn run_event_task(
        self,
        id: String,
        runtime: Arc<AgentRuntime>,
        mut events: tokio::sync::mpsc::UnboundedReceiver<bridle_claude::events::Event>,
        exit: tokio::sync::oneshot::Receiver<ExitOutcome>,
        exited_tx: watch::Sender<bool>,
        mut ready_tx: Option<tokio::sync::oneshot::Sender<()>>,
    ) {
        while let Some(ev) = events.recv().await {
            let is_init = matches!(ev.kind, ClaudeEventKind::SystemInit(_));
            self.handle_claude_event(&id, &runtime, ev).await;
            // Fired after handle_claude_event, not before: by the time
            // `spawn` sees this, the store already reflects `working` and
            // `turn.started` has already been emitted, not just enqueued.
            if is_init && let Some(tx) = ready_tx.take() {
                let _ = tx.send(());
            }
        }
        let outcome = exit.await.unwrap_or(ExitOutcome {
            code: None,
            signal: None,
            stderr_tail: Vec::new(),
        });
        self.finish_agent(&id, &runtime, outcome).await;
        let _ = exited_tx.send(true);
        self.0
            .runtimes
            .lock()
            .expect("runtimes mutex poisoned")
            .remove(&id);
    }

    async fn handle_claude_event(
        &self,
        id: &str,
        runtime: &Arc<AgentRuntime>,
        ev: bridle_claude::events::Event,
    ) {
        let should_touch = {
            let mut st = runtime.state.lock().await;
            st.saw_any_line = true;
            st.stall_notified = false;
            let due = st.last_touch.elapsed() >= Duration::from_secs(1);
            if due {
                st.last_touch = std::time::Instant::now();
            }
            due
        };
        if should_touch {
            self.touch_now(id).await;
        }

        match ev.kind {
            ClaudeEventKind::SystemInit(init) => {
                let (n, check_version) = {
                    let mut st = runtime.state.lock().await;
                    st.turn_n += 1;
                    let check = !st.version_checked;
                    st.version_checked = true;
                    (st.turn_n, check)
                };
                if check_version && let Some(version) = init.claude_code_version.as_deref() {
                    self.note_claude_version(id, version).await;
                }
                let _ = self.0.store.add_turn_start(id, n, Utc::now()).await;
                let _ = self.0.store.mark_session_started(id).await;
                self.transition_state(id, runtime, AgentState::Working)
                    .await;
                let _ = self
                    .0
                    .emitter
                    .emit(
                        event_kind::TURN_STARTED,
                        "system".to_string(),
                        Some(id.to_string()),
                        json!({"n": n}),
                    )
                    .await;
            }
            ClaudeEventKind::System { .. } => {}
            ClaudeEventKind::Assistant(msg) => {
                for block in msg.blocks() {
                    match block {
                        ContentBlock::Text { text } => {
                            let _ = self
                                .0
                                .emitter
                                .emit(
                                    event_kind::AGENT_TEXT,
                                    "system".to_string(),
                                    Some(id.to_string()),
                                    json!({"text": truncate_chars(&text, TRUNCATE_TEXT)}),
                                )
                                .await;
                        }
                        ContentBlock::ToolUse { name, input, .. } => {
                            let _ = self
                                .0
                                .emitter
                                .emit(
                                    event_kind::TOOL_USE,
                                    "system".to_string(),
                                    Some(id.to_string()),
                                    json!({"name": name, "input_summary": input_summary(&input)}),
                                )
                                .await;
                        }
                        _ => {}
                    }
                }
            }
            ClaudeEventKind::User(msg) => {
                if msg.is_replay() {
                    let Some(text) = msg.replay_text() else {
                        return;
                    };
                    let matched = {
                        let mut st = runtime.state.lock().await;
                        let pos = st.fifo.iter().position(|(_, t)| t == text);
                        pos.map(|i| st.fifo.remove(i).expect("position just found"))
                    };
                    if let Some((mid, _)) = matched {
                        let _ = self
                            .0
                            .store
                            .set_message_state(&mid, MessageState::Delivered, Utc::now())
                            .await;
                        let _ = self
                            .0
                            .emitter
                            .emit(
                                event_kind::MESSAGE_DELIVERED,
                                "system".to_string(),
                                Some(id.to_string()),
                                json!({"message": mid}),
                            )
                            .await;
                    } else {
                        tracing::debug!(agent = id, "unmatched replayed user message");
                    }
                }
            }
            ClaudeEventKind::Result(r) => {
                // Captured before `end_turn` commits this turn's
                // `context_tokens` below, not after (htp6b): a concurrent
                // `tick_context_check` can only see this turn's own crossing
                // once that commit lands, so a pending flag already true
                // *here* can only be an earlier turn's crossing — proof this
                // is genuinely the handoff turn ending, not the crossing
                // turn racing its own notice. Reading the flag fresh after
                // the commit (as the wind-down check below does, safely,
                // since nothing else writes context_tokens) would let this
                // turn's own crossing win that race and trigger renew
                // immediately, on `stop_grace` instead of the intended
                // `wind_down_grace`.
                let (delta, cumulative, n, was_context_renew_pending) = {
                    let mut st = runtime.state.lock().await;
                    // Fall back to the last known cumulative (a zero delta),
                    // not 0.0: claude's `total_cost_usd` is a running total,
                    // so treating a missing value as "zero so far" would
                    // manufacture a large negative delta.
                    let cumulative = r.total_cost_usd.unwrap_or(st.last_cumulative);
                    let delta = cumulative - st.last_cumulative;
                    st.last_cumulative = cumulative;
                    (delta, cumulative, st.turn_n, st.context_renew_pending)
                };
                let usage = r.usage.clone().unwrap_or_default();
                // `result.usage` is summed over every API call the turn made
                // (docs/spikes/01-stream-json-findings.md row 8), not the
                // current context size: a turn with N tool round-trips
                // re-reads the cached prompt N times, inflating this sum to
                // roughly N times the real context. `get_context_usage`
                // reports the actual current size directly (spike 01 §11);
                // fall back to the (inflated) sum only if that probe fails.
                let turn_usage_sum = usage.input_tokens
                    + usage.cache_read_input_tokens
                    + usage.cache_creation_input_tokens;
                let context_tokens = match runtime
                    .handle
                    .get_context_usage(CONTEXT_USAGE_TIMEOUT)
                    .await
                {
                    Ok(v) => v
                        .pointer("/response/response/totalTokens")
                        .and_then(Value::as_u64)
                        .unwrap_or_else(|| {
                            tracing::warn!(
                                agent = id,
                                response = %v,
                                "get_context_usage response missing totalTokens; \
                                 falling back to the (inflated) turn usage sum"
                            );
                            turn_usage_sum
                        }),
                    Err(e) => {
                        tracing::warn!(
                            agent = id,
                            error = %e,
                            "get_context_usage probe failed; falling back to \
                             the (inflated) turn usage sum"
                        );
                        turn_usage_sum
                    }
                };
                let turn_end = crate::store::TurnEnd {
                    subtype: r.subtype.clone(),
                    is_error: r.is_error,
                    terminal_reason: r.terminal_reason.clone(),
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    cache_read: usage.cache_read_input_tokens,
                    cache_write: usage.cache_creation_input_tokens,
                    cost_total: delta,
                    context_tokens,
                };
                let _ = self.0.store.end_turn(id, n, turn_end).await;

                if !r.permission_denials.is_empty() {
                    let _ = self
                        .0
                        .emitter
                        .emit(
                            event_kind::PERMISSION_DENIED,
                            "system".to_string(),
                            Some(id.to_string()),
                            json!({"denials": r.permission_denials}),
                        )
                        .await;
                }

                self.transition_state(id, runtime, AgentState::Idle).await;
                let _ = self
                    .0
                    .emitter
                    .emit(
                        event_kind::TURN_ENDED,
                        "system".to_string(),
                        Some(id.to_string()),
                        json!({
                            "n": n,
                            "subtype": r.subtype,
                            "is_error": r.is_error,
                            "terminal_reason": r.terminal_reason,
                            "usage": {
                                "input_tokens": usage.input_tokens,
                                "output_tokens": usage.output_tokens,
                                "cache_read": usage.cache_read_input_tokens,
                                "cache_write": usage.cache_creation_input_tokens,
                            },
                            "cost_total": cumulative,
                            "result": r.result.as_deref().map(|s| truncate_chars(s, TRUNCATE_TEXT)),
                        }),
                    )
                    .await;

                // Every later turn would fail at once without calling the
                // model, so stop the agent instead of leaving it idle and
                // useless. Its messages wait, pending, for a resume.
                if r.subtype == BUDGET_EXHAUSTED_SUBTYPE {
                    if !runtime.budget_exhausted.swap(true, Ordering::SeqCst) {
                        let _ = self
                            .0
                            .emitter
                            .emit(
                                event_kind::AGENT_BUDGET_EXHAUSTED,
                                "system".to_string(),
                                Some(id.to_string()),
                                json!({"cost_total": cumulative}),
                            )
                            .await;
                        let this = self.clone();
                        let id = id.to_string();
                        tokio::spawn(async move {
                            let _ = this.stop(&id, false, &system_principal()).await;
                        });
                    }
                    return;
                }

                // The governor's wind-down notice told this agent to end its
                // turn (usage-and-budget.md, The wind-down step 5); do that
                // now instead of starting another turn on a held message.
                // Spawned, not awaited, for the same reason as above: `stop`
                // joins this very task.
                let wind_down_pending = {
                    let st = runtime.state.lock().await;
                    st.wind_down_pending
                };
                if wind_down_pending {
                    runtime.budget_paused.store(true, Ordering::SeqCst);
                    let this = self.clone();
                    let id = id.to_string();
                    tokio::spawn(async move {
                        let _ = this.stop(&id, false, &system_principal()).await;
                    });
                    return;
                }

                // The context governor's handoff notice told this agent to
                // wrap up (htp6b); renew now instead of starting another
                // turn on a held message, the same way the budget wind-down
                // does above. Spawned, not awaited, for the same reason.
                // Uses `was_context_renew_pending`, captured before this
                // turn's own `context_tokens` commit above, not a fresh
                // read: this turn's own crossing (if any) can only have set
                // the flag *after* that commit, so it can't masquerade here
                // as an already-finished handoff turn (see the comment on
                // that capture).
                if was_context_renew_pending {
                    // Don't start another turn on a held message either
                    // way, but only the caller that wins `claim_context_renew`
                    // actually spawns `renew`: this turn's own end can race
                    // `expire_context_renews`'s sweep for the same pending
                    // renew.
                    if claim_context_renew(runtime) {
                        let this = self.clone();
                        let id = id.to_string();
                        tokio::spawn(async move {
                            let _ = this.renew(&id, &system_principal()).await;
                        });
                    }
                    return;
                }

                self.deliver_oldest_held(id).await;
            }
            ClaudeEventKind::RateLimit(rl) => {
                for w in rl.windows() {
                    let resets_at = w
                        .resets_at_epoch
                        .and_then(|e| chrono::DateTime::from_timestamp(e, 0));
                    let _ = self
                        .0
                        .store
                        .upsert_rate_limit(bridle_api::types::RateLimit {
                            window: w.window,
                            status: w.status,
                            utilization: w.utilization,
                            resets_at,
                            observed_at: Utc::now(),
                        })
                        .await;
                }
                let _ = self
                    .0
                    .emitter
                    .emit(
                        event_kind::RATE_LIMIT,
                        "system".to_string(),
                        Some(id.to_string()),
                        json!({"info": rl.rate_limit_info}),
                    )
                    .await;
            }
            ClaudeEventKind::ControlResponse(_)
            | ClaudeEventKind::ControlRequest(_)
            | ClaudeEventKind::Unknown { .. }
            | ClaudeEventKind::Unparsed { .. }
            | ClaudeEventKind::NotJson => {}
        }
    }

    /// Claude Code updates itself, and bridle leans on behaviour it doesn't
    /// document. A new version is accepted, not refused; this makes the
    /// change visible so the contract tests get run
    /// (docs/design/agent-host/agents.md, Claude Code upgrades).
    async fn note_claude_version(&self, id: &str, version: &str) {
        let previous = match self.0.store.swap_meta("claude_version", version).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, "recording claude version");
                return;
            }
        };
        if previous.as_deref() == Some(version) {
            return;
        }
        if let Some(prev) = &previous {
            tracing::warn!(
                from = prev.as_str(),
                to = version,
                "Claude Code version changed; run `just test-contract`"
            );
        }
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::CLAUDE_VERSION,
                "system".to_string(),
                Some(id.to_string()),
                json!({"version": version, "previous": previous}),
            )
            .await;
    }

    async fn transition_state(&self, id: &str, runtime: &Arc<AgentRuntime>, to: AgentState) {
        let from = {
            let mut st = runtime.state.lock().await;
            let from = st.current_state;
            st.current_state = to;
            from
        };
        if from == to {
            return;
        }
        let _ = self.0.store.set_agent_state(id, to).await;
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_STATE,
                "system".to_string(),
                Some(id.to_string()),
                json!({"from": from.as_str(), "to": to.as_str()}),
            )
            .await;
    }

    async fn touch_now(&self, id: &str) {
        let _ = self.0.store.touch_agent(id, Utc::now()).await;
    }

    /// Sweeps a tracker and emits `agent.orphans_killed` if it found
    /// anything. Shared between [`AgentManager::stop`] (which sweeps eagerly
    /// as part of the stop sequence, step 4) and
    /// [`AgentManager::finish_agent`] (which sweeps on every exit path, not
    /// just an explicit stop). Whichever call actually catches live
    /// descendants reports them; a sweep that finds nothing left (because
    /// an earlier one already reaped them) simply stays quiet, so the two
    /// calls are safe to both run without double-counting.
    async fn sweep_and_emit(
        &self,
        id: &str,
        tracker: &mut Tracker,
        snap: &[containment::ProcInfo],
    ) {
        let report = containment::sweep(tracker, snap, SWEEP_GRACE).await;
        if report.terminated > 0 {
            let _ = self
                .0
                .emitter
                .emit(
                    event_kind::AGENT_ORPHANS_KILLED,
                    "system".to_string(),
                    Some(id.to_string()),
                    json!({"count": report.terminated}),
                )
                .await;
        }
    }

    async fn finish_agent(&self, id: &str, runtime: &Arc<AgentRuntime>, outcome: ExitOutcome) {
        let snap = tokio::task::spawn_blocking(containment::snapshot)
            .await
            .ok()
            .and_then(Result::ok);
        let mut st = runtime.state.lock().await;
        if let Some(snap) = &snap {
            st.tracker.update(snap);
            self.sweep_and_emit(id, &mut st.tracker, snap).await;
        }
        let stop_requested = runtime.stop_requested.load(Ordering::SeqCst);
        let shutdown_requested = runtime.shutdown_requested.load(Ordering::SeqCst);
        let saw_any_line = st.saw_any_line;
        drop(st);

        let (state, mut exit) =
            classify_exit(stop_requested, shutdown_requested, saw_any_line, &outcome);
        // Otherwise a claude that dies on start (e.g. a `--resume` it can't
        // find) leaves its reason only in the transcript.
        if !stop_requested && !shutdown_requested && outcome.code != Some(0) {
            tracing::warn!(
                agent = %id, code = ?outcome.code, signal = ?outcome.signal,
                reason = %exit.reason, stderr = %outcome.stderr_tail.join("; "),
                "claude exited abnormally"
            );
        }
        if runtime.budget_exhausted.load(Ordering::SeqCst) {
            exit.reason = "budget_exhausted".to_string();
        } else if runtime.budget_paused.load(Ordering::SeqCst) {
            exit.reason = "budget_paused".to_string();
        }
        let _ = self.0.store.set_agent_exit(id, exit.clone()).await;
        self.transition_state(id, runtime, state).await;
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_EXITED,
                "system".to_string(),
                Some(id.to_string()),
                json!({"code": exit.code, "signal": exit.signal, "reason": exit.reason}),
            )
            .await;

        // Held messages go back to pending too: after a resume the agent is
        // idle, so `--when idle` has nothing left to wait for.
        if let Ok(undelivered) = self
            .0
            .store
            .messages_for_agent(id, &[MessageState::Written, MessageState::Held])
            .await
        {
            for m in undelivered {
                let _ = self
                    .0
                    .store
                    .set_message_state(&m.id, MessageState::Pending, Utc::now())
                    .await;
            }
        }
    }

    // ---------- messages ----------

    pub async fn send(
        &self,
        from: PrincipalId,
        to: ToTarget,
        kind: MessageKind,
        body: String,
        when: bridle_api::types::When,
        reply_to: Option<String>,
    ) -> Result<Message, SupervisorError> {
        let (to_id, to_kind) = match &to {
            ToTarget::Human => ("human".to_string(), crate::store::RecipientKind::Human),
            ToTarget::Agent(id) => (id.clone(), crate::store::RecipientKind::Agent),
            ToTarget::External(id) => (id.clone(), crate::store::RecipientKind::External),
        };
        let inserted = self
            .0
            .store
            .insert_message(NewMessage {
                from: from.clone(),
                to: to_id.clone(),
                to_kind,
                kind,
                body,
                reply_to,
                when,
                state: MessageState::Pending,
            })
            .await?;
        // A delegate's reply to a message addressed to the human closes it there.
        if let Some(qid) = inserted.reply_to.as_deref()
            && self.0.config.messages.answer_for_human.contains(&from)
            && let Some(q) = self.0.store.get_message(qid).await?
            && q.to == "human"
        {
            self.0
                .store
                .answer_message(
                    qid,
                    &from,
                    &inserted.id,
                    inserted
                        .body
                        .lines()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or(""),
                    Utc::now(),
                )
                .await?;
        }
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::MESSAGE_SENT,
                from,
                if matches!(to, ToTarget::Agent(_)) {
                    Some(to_id.clone())
                } else {
                    None
                },
                json!({"message": inserted.id, "to": to_id}),
            )
            .await;

        if let ToTarget::Agent(agent_id) = &to {
            let agent = self
                .0
                .store
                .get_agent(agent_id)
                .await?
                .ok_or_else(|| SupervisorError::NotFound(agent_id.clone()))?;
            if agent.state.is_running()
                && let Some(rt) = self.get_runtime(agent_id)
            {
                let mut write_now = matches!(when, bridle_api::types::When::Now)
                    || (matches!(when, bridle_api::types::When::Idle)
                        && agent.state == AgentState::Idle);
                // A message to an idle agent would start a new turn; while
                // the governor isn't normal, that's held instead
                // (usage-and-budget.md, hold_at). A message folding into an
                // already-running turn is unaffected: that turn was already
                // permitted to run.
                if agent.state == AgentState::Idle
                    && self.budget_block_for_model(&agent.model).state
                        != bridle_api::types::GovernorState::Normal
                {
                    write_now = false;
                }
                if write_now {
                    let _ = write_message(&self.0.store, &rt, &inserted).await;
                } else {
                    let _ = self
                        .0
                        .store
                        .set_message_state(&inserted.id, MessageState::Held, Utc::now())
                        .await;
                }
            }
        }

        self.0
            .store
            .get_message(&inserted.id)
            .await?
            .ok_or_else(|| SupervisorError::Internal("message vanished after insert".to_string()))
    }

    // ---------- budget governor wind-down ----------

    /// Marks `id` as wind-down-pending with grace deadline `deadline`
    /// (`Instant::now()` for an immediate `paused` escalation, otherwise
    /// `now + wind_down_grace`). Returns `Some(true)` the first time (the
    /// caller should send the notice), `Some(false)` on a later call for an
    /// already-pending agent (only the deadline is tightened, never
    /// loosened), `None` if the agent isn't running.
    pub async fn mark_wind_down(&self, id: &str, deadline: std::time::Instant) -> Option<bool> {
        let rt = self.get_runtime(id)?;
        let mut st = rt.state.lock().await;
        let first = !st.wind_down_pending;
        st.wind_down_pending = true;
        if st.wind_down_deadline.is_none_or(|d| deadline < d) {
            st.wind_down_deadline = Some(deadline);
        }
        Some(first)
    }

    /// Interrupts and stops, with exit reason `budget_paused`, every
    /// wind-down-pending agent whose grace deadline has passed. Called on
    /// every governor tick (usage-and-budget.md, The wind-down step 5).
    pub async fn expire_wind_downs(&self) {
        let now = std::time::Instant::now();
        let mut due = Vec::new();
        for (id, rt) in self.runtimes_snapshot() {
            let is_due = {
                let st = rt.state.lock().await;
                st.wind_down_pending && st.wind_down_deadline.is_some_and(|d| now >= d)
            };
            if is_due {
                due.push(id);
            }
        }
        for id in due {
            self.stop_for_budget(&id, true).await;
        }
    }

    /// Writes `id`'s oldest held message, if it has one and is running.
    /// Called at the end of every turn (so an agent that finishes with a
    /// message still held picks it up right away) and by the governor when
    /// it recovers to `normal` (so an agent that's already idle at that
    /// point isn't left holding forever with no turn ending to trigger it).
    pub async fn deliver_oldest_held(&self, id: &str) {
        let Some(rt) = self.get_runtime(id) else {
            return;
        };
        if let Ok(held) = self
            .0
            .store
            .messages_for_agent(id, &[MessageState::Held])
            .await
            && let Some(oldest) = held.into_iter().next()
        {
            let _ = write_message(&self.0.store, &rt, &oldest).await;
        }
    }

    /// Stops `id` with exit reason `budget_paused`: an idle agent stopped at
    /// once by the governor, or a wind-down-pending one whose grace expired
    /// (`interrupt_first`, since its turn is still running).
    pub async fn stop_for_budget(&self, id: &str, interrupt_first: bool) {
        if let Some(rt) = self.get_runtime(id) {
            rt.budget_paused.store(true, Ordering::SeqCst);
        }
        if interrupt_first {
            let _ = self.interrupt(id, false, &system_principal()).await;
        }
        let _ = self.stop(id, false, &system_principal()).await;
    }

    // ---------- interrupt / stop / resume / remove ----------

    pub async fn interrupt(
        &self,
        id_or_name: &str,
        drop_held: bool,
        principal: &Principal,
    ) -> Result<bridle_api::types::InterruptResponse, SupervisorError> {
        let agent = self
            .0
            .store
            .get_agent(id_or_name)
            .await?
            .ok_or_else(|| SupervisorError::NotFound(id_or_name.to_string()))?;
        if !agent.state.is_running() {
            return Err(SupervisorError::AgentNotRunning(agent.id));
        }
        let rt = self
            .get_runtime(&agent.id)
            .ok_or_else(|| SupervisorError::AgentNotRunning(agent.id.clone()))?;
        let receipt = rt
            .handle
            .interrupt(INTERRUPT_TIMEOUT)
            .await
            .map_err(|e| SupervisorError::Internal(format!("interrupt: {e}")))?;

        let mut dropped = 0u32;
        if drop_held {
            let held = self
                .0
                .store
                .messages_for_agent(&agent.id, &[MessageState::Held])
                .await?;
            for m in held {
                self.0
                    .store
                    .set_message_state(&m.id, MessageState::Dropped, Utc::now())
                    .await?;
                let _ = self
                    .0
                    .emitter
                    .emit(
                        event_kind::MESSAGE_DROPPED,
                        principal.id.clone(),
                        Some(agent.id.clone()),
                        json!({"message": m.id}),
                    )
                    .await;
                dropped += 1;
            }
        }
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_INTERRUPTED,
                principal.id.clone(),
                Some(agent.id.clone()),
                json!({"dropped_held": dropped}),
            )
            .await;
        Ok(bridle_api::types::InterruptResponse {
            receipt,
            dropped_held: dropped,
        })
    }

    pub async fn stop(
        &self,
        id_or_name: &str,
        now: bool,
        principal: &Principal,
    ) -> Result<Agent, SupervisorError> {
        let agent = self
            .0
            .store
            .get_agent(id_or_name)
            .await?
            .ok_or_else(|| SupervisorError::NotFound(id_or_name.to_string()))?;
        if !agent.state.is_running() {
            return Ok(agent);
        }
        let Some(rt) = self.get_runtime(&agent.id) else {
            return Ok(agent);
        };
        rt.stop_requested.store(true, Ordering::SeqCst);
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_STOP_REQUESTED,
                principal.id.clone(),
                Some(agent.id.clone()),
                json!({"now": now}),
            )
            .await;
        self.transition_state(&agent.id, &rt, AgentState::Stopping)
            .await;

        // Snapshot right before doing anything that might end the
        // process, so short-lived tool/child processes are caught while
        // they're still parented under the agent's pid (once the agent
        // exits, an orphan is reparented to init and this walk can no
        // longer find it by ancestry).
        let snap = tokio::task::spawn_blocking(containment::snapshot)
            .await
            .ok()
            .and_then(Result::ok);
        if let Some(snap) = &snap {
            let mut st = rt.state.lock().await;
            st.tracker.update(snap);
        }

        if !now {
            rt.handle.close_stdin();
            let mut exited = rt.exited.clone();
            let _ = tokio::time::timeout(self.0.config.stop_grace, exited.wait_for(|v| *v)).await;
        }

        containment::terminate_group(rt.handle.pid(), TERMINATE_GRACE).await;
        if let Some(snap) = &snap {
            let mut st = rt.state.lock().await;
            self.sweep_and_emit(&agent.id, &mut st.tracker, snap).await;
        }

        let task = rt.task.lock().await.take();
        if let Some(task) = task {
            let _ = task.await;
        } else {
            let mut exited = rt.exited.clone();
            let _ = exited.wait_for(|v| *v).await;
        }

        self.0
            .store
            .get_agent(&agent.id)
            .await?
            .ok_or_else(|| SupervisorError::Internal("agent vanished after stop".to_string()))
    }

    pub async fn resume(
        &self,
        id_or_name: &str,
        ignore_budget: bool,
        principal: &Principal,
    ) -> Result<Agent, SupervisorError> {
        let agent = self
            .0
            .store
            .get_agent(id_or_name)
            .await?
            .ok_or_else(|| SupervisorError::NotFound(id_or_name.to_string()))?;
        if !agent.state.is_resumable() {
            return Err(SupervisorError::Conflict(format!(
                "agent {} is not resumable (state {})",
                agent.id, agent.state
            )));
        }
        if !ignore_budget {
            self.refuse_if_holding(&agent.model)?;
        }
        let role = self
            .0
            .config
            .roles
            .get(&agent.role)
            .cloned()
            .unwrap_or_else(Role::worker_default);

        let token_path = self.0.workspace.agent_dir(&agent.id).join("token");
        let token = std::fs::read_to_string(&token_path)
            .map_err(|e| SupervisorError::Internal(format!("reading agent token: {e}")))?
            .trim()
            .to_string();

        let prompt_text = crate::config::render_system_prompt(
            &agent.role,
            &role,
            &self.0.workspace.repo,
            &self.0.config.branches,
            &self.0.config.commands,
            &agent.name,
            std::path::Path::new(&agent.cwd),
            agent.branch.as_deref(),
        );
        let system_prompt_path = self.0.workspace.system_prompt(&agent.id);
        std::fs::write(&system_prompt_path, &prompt_text)?;

        let session_uuid = Uuid::parse_str(&agent.session_id)
            .map_err(|e| SupervisorError::Internal(format!("bad session id: {e}")))?;
        // A session that never started a turn (a renew, then a restart before
        // its first turn) doesn't exist to claude, and `--resume` of it dies
        // on the first message (p4ks). Start a fresh one instead; the
        // continuation note below points it at the handoff.
        let (session_uuid, session) = if self.0.store.session_started(&agent.id).await? {
            (session_uuid, Session::Resume(session_uuid))
        } else {
            let fresh = Uuid::new_v4();
            tracing::warn!(
                agent = %agent.id, stale_session = %session_uuid, new_session = %fresh,
                "stored session never started a turn; resuming into a fresh session"
            );
            (fresh, Session::New(fresh))
        };
        let session_id = session_uuid.to_string();
        let cwd = std::path::PathBuf::from(&agent.cwd);
        let mut cmd = ClaudeCommand::new(cwd, session);
        cmd.program = self.0.claude_program.clone();
        cmd.model = Some(agent.model.clone());
        cmd.effort = role.effort.clone();
        cmd.append_system_prompt_file = Some(system_prompt_path);
        cmd.permission_mode = Some(role.permission_mode.clone());
        cmd.allowed_tools = role.effective_allowed_tools();
        let (extra_allowed_tools, extra_env) = self.0.store.get_agent_overrides(&agent.id).await?;
        for tool in &extra_allowed_tools {
            if !cmd.allowed_tools.contains(tool) {
                cmd.allowed_tools.push(tool.clone());
            }
        }
        cmd.disallowed_tools = role.disallowed_tools.clone();
        cmd.stop_check = role.stop_check;
        cmd.name = Some(agent.name.clone());
        cmd.max_budget_usd = role.max_budget_usd;
        cmd.env = agent_env(
            &self.0.workspace,
            &self.0.url,
            &self.0.project,
            &agent.id,
            &agent.name,
            &token,
            &agent.components,
        );
        if !extra_env.is_empty() {
            cmd.env.extend(extra_env);
        }

        let transcript = Transcript::open(
            &self.0.workspace.transcript(&agent.id),
            std::time::Instant::now(),
        )?;
        let spawned = bridle_claude::process::spawn(&cmd, transcript)
            .await
            .map_err(|e| SupervisorError::Internal(format!("spawning claude: {e}")))?;

        if session_id != agent.session_id {
            self.0
                .store
                .set_agent_session(&agent.id, &session_id)
                .await?;
        }

        self.register_and_start(
            StartingAgent {
                agent_id: &agent.id,
                session_id: &session_id,
                turns_so_far: agent.turns,
                cost_so_far: agent.cost_usd_total,
            },
            spawned,
            principal,
            None,
        )
        .await?;
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_RESUMED,
                principal.id.clone(),
                Some(agent.id.clone()),
                json!({"from": agent.state.as_str()}),
            )
            .await;

        let pending = self
            .0
            .store
            .messages_for_agent(&agent.id, &[MessageState::Pending])
            .await?;
        if let Some(rt) = self.get_runtime(&agent.id) {
            for m in pending {
                let _ = write_message(&self.0.store, &rt, &m).await;
            }
        }

        // Same gap as `renew` (br-ab66): `--resume` still just waits on
        // stdin, so with no pending messages the process would otherwise sit
        // idle forever.
        self.send_continuation_note(
            principal,
            &agent.id,
            "You were resumed after a daemon restart. Continue from your task's thread and \
             your own last handoff note",
        )
        .await?;

        self.0
            .store
            .get_agent(&agent.id)
            .await?
            .ok_or_else(|| SupervisorError::Internal("agent vanished after resume".to_string()))
    }

    /// Stops the agent if it's running, then starts its replacement: a fresh
    /// `claude` process, fresh session (`Session::New`, unlike `resume`'s
    /// `Session::Resume`), in the *same* worktree/branch/role/model. No
    /// worktree or branch is created or removed — `stop` never touches
    /// either, so the one from the agent's original spawn is still attached
    /// and checked out; renew just points a new process at it. The agent
    /// keeps its id and name throughout, so nothing else that addresses it
    /// (messages, tasks) needs to know it was renewed.
    // A plain `async fn` here would make its opaque return type part of a
    // mutually recursive cycle: `register_and_start` spawns
    // `run_event_task`, which calls `handle_claude_event`, which (htp6b)
    // calls `renew` on a context-handoff turn's end, which calls
    // `register_and_start` again. `rustc` can't resolve that cycle through
    // `impl Future` alone (each is a fresh task at runtime, but the
    // compiler still needs a non-opaque type to close the loop), so this
    // boxes the future explicitly instead.
    pub fn renew<'a>(
        &'a self,
        id_or_name: &'a str,
        principal: &'a Principal,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Agent, SupervisorError>> + Send + 'a>,
    > {
        Box::pin(self.renew_inner(id_or_name, principal))
    }

    async fn renew_inner(
        &self,
        id_or_name: &str,
        principal: &Principal,
    ) -> Result<Agent, SupervisorError> {
        let mut agent = self
            .0
            .store
            .get_agent(id_or_name)
            .await?
            .ok_or_else(|| SupervisorError::NotFound(id_or_name.to_string()))?;
        let from_state = agent.state;
        // No hold check: a renew replaces a session rather than adding load,
        // so it never waits on the governor (r3nh). That also makes the
        // stop below safe: nothing after it can be refused for budget.
        if agent.state.is_running() {
            agent = self.stop(&agent.id, false, principal).await?;
        }
        let role = self
            .0
            .config
            .roles
            .get(&agent.role)
            .cloned()
            .unwrap_or_else(Role::worker_default);

        let token_path = self.0.workspace.agent_dir(&agent.id).join("token");
        let token = std::fs::read_to_string(&token_path)
            .map_err(|e| SupervisorError::Internal(format!("reading agent token: {e}")))?
            .trim()
            .to_string();

        let prompt_text = crate::config::render_system_prompt(
            &agent.role,
            &role,
            &self.0.workspace.repo,
            &self.0.config.branches,
            &self.0.config.commands,
            &agent.name,
            std::path::Path::new(&agent.cwd),
            agent.branch.as_deref(),
        );
        let system_prompt_path = self.0.workspace.system_prompt(&agent.id);
        std::fs::write(&system_prompt_path, &prompt_text)?;

        // A renew's whole point is a clean context, so the replacement gets
        // a brand-new session, not `--resume` of the one it's leaving behind.
        // The store isn't updated with it until after claude actually spawns
        // (below): otherwise a failed spawn would leave the row pointing at
        // a session_id claude never started, and a later `resume` would
        // `--resume` a session that doesn't exist.
        let session_id = Uuid::new_v4();

        let cwd = std::path::PathBuf::from(&agent.cwd);
        let mut cmd = ClaudeCommand::new(cwd, Session::New(session_id));
        cmd.program = self.0.claude_program.clone();
        cmd.model = Some(agent.model.clone());
        cmd.effort = role.effort.clone();
        cmd.append_system_prompt_file = Some(system_prompt_path);
        cmd.permission_mode = Some(role.permission_mode.clone());
        cmd.allowed_tools = role.effective_allowed_tools();
        let (extra_allowed_tools, extra_env) = self.0.store.get_agent_overrides(&agent.id).await?;
        for tool in &extra_allowed_tools {
            if !cmd.allowed_tools.contains(tool) {
                cmd.allowed_tools.push(tool.clone());
            }
        }
        cmd.disallowed_tools = role.disallowed_tools.clone();
        cmd.stop_check = role.stop_check;
        cmd.name = Some(agent.name.clone());
        cmd.max_budget_usd = role.max_budget_usd;
        cmd.env = agent_env(
            &self.0.workspace,
            &self.0.url,
            &self.0.project,
            &agent.id,
            &agent.name,
            &token,
            &agent.components,
        );
        if !extra_env.is_empty() {
            cmd.env.extend(extra_env);
        }

        let transcript = Transcript::open(
            &self.0.workspace.transcript(&agent.id),
            std::time::Instant::now(),
        )?;
        let spawned = bridle_claude::process::spawn(&cmd, transcript)
            .await
            .map_err(|e| SupervisorError::Internal(format!("spawning claude: {e}")))?;

        self.0
            .store
            .set_agent_session(&agent.id, &session_id.to_string())
            .await?;

        self.register_and_start(
            StartingAgent {
                agent_id: &agent.id,
                session_id: &session_id.to_string(),
                turns_so_far: agent.turns,
                cost_so_far: agent.cost_usd_total,
            },
            spawned,
            principal,
            None,
        )
        .await?;
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_RENEWED,
                principal.id.clone(),
                Some(agent.id.clone()),
                json!({"from": from_state.as_str()}),
            )
            .await;

        let pending = self
            .0
            .store
            .messages_for_agent(&agent.id, &[MessageState::Pending])
            .await?;
        if let Some(rt) = self.get_runtime(&agent.id) {
            for m in pending {
                let _ = write_message(&self.0.store, &rt, &m).await;
            }
        }

        // Unlike `spawn`, a renew never has a `req.prompt`/`start_prompt` to
        // start the fresh process's first turn with, and the common case has
        // no pending messages either (the agent renewed itself mid-task,
        // nothing new was sent to it) — so without this, the process just
        // sits on stdin forever (br-ab66). The context governor already told
        // the outgoing process to leave a handoff note before renewing
        // (`tick_context_check` above); point the replacement at it.
        let handoff_note = match agent.context_tokens {
            Some(tokens) => format!(
                "You were renewed for context (you were at ~{tokens} tokens). Continue from \
                 your task's thread and your own last handoff note"
            ),
            None => "You were renewed for context. Continue from your task's thread and your \
                     own last handoff note"
                .to_string(),
        };
        self.send_continuation_note(principal, &agent.id, &handoff_note)
            .await?;

        self.0
            .store
            .get_agent(&agent.id)
            .await?
            .ok_or_else(|| SupervisorError::Internal("agent vanished after renew".to_string()))
    }

    /// Points a freshly (re)started process at where to pick up: sent as a
    /// `Note` the same way `spawn`'s `first_message` is, since a fresh
    /// `claude` process otherwise just waits on stdin (br-ab66).
    async fn send_continuation_note(
        &self,
        principal: &Principal,
        agent_id: &str,
        lead_in: &str,
    ) -> Result<(), SupervisorError> {
        self.send(
            principal.id.clone(),
            ToTarget::Agent(agent_id.to_string()),
            MessageKind::Note,
            format!("{lead_in} — check `bridle inbox` and the task body for where you left off."),
            bridle_api::types::When::Now,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn remove(
        &self,
        id_or_name: &str,
        force: bool,
        delete_branch: bool,
        principal: &Principal,
    ) -> Result<(), SupervisorError> {
        let mut agent = self
            .0
            .store
            .get_agent(id_or_name)
            .await?
            .ok_or_else(|| SupervisorError::NotFound(id_or_name.to_string()))?;
        // Refuse before touching anything, so a refused rm leaves no half-removed agent.
        if delete_branch
            && !force
            && let Some(branch) = &agent.branch
            && !worktree::is_merged(&self.0.workspace.repo, branch).await?
        {
            return Err(SupervisorError::Conflict(format!(
                "branch {branch} is not merged into HEAD; merge it, drop --delete-branch, or use --force"
            )));
        }
        let dirty_refusal = || {
            SupervisorError::Conflict("worktree has uncommitted changes; use --force".to_string())
        };
        let worktree_dirty = |wt: Option<String>| async move {
            match wt {
                Some(wt) => worktree::is_dirty(std::path::Path::new(&wt))
                    .await
                    .unwrap_or(false),
                None => false,
            }
        };
        if !force && worktree_dirty(agent.worktree.clone()).await {
            return Err(dirty_refusal());
        }
        if agent.state.is_running() {
            agent = self.stop(&agent.id, false, principal).await?;
            // The agent's last turn may have left changes behind.
            if !force && worktree_dirty(agent.worktree.clone()).await {
                return Err(dirty_refusal());
            }
        }
        if let Some(wt) = agent.worktree.clone() {
            let path = std::path::PathBuf::from(&wt);
            // Already gone (e.g. an earlier rm failed after removing it): just prune.
            if path.exists() {
                // Catches a process bridle's containment sweep missed that
                // still has a file open under the worktree: removing the
                // directory out from under it would corrupt its view.
                if !force && let Some(holder) = worktree::open_file_holder(&path).await? {
                    return Err(SupervisorError::Conflict(format!(
                        "worktree has an open file ({holder}); use --force"
                    )));
                }
                worktree::remove(&self.0.workspace.repo, &path, force).await?;
            } else {
                worktree::prune(&self.0.workspace.repo).await?;
            }
            if delete_branch && let Some(branch) = &agent.branch {
                // Past the merged check above (or forced), so `-D`: git itself
                // doesn't see a squash-landed branch as merged.
                worktree::delete_branch(&self.0.workspace.repo, branch, true).await?;
            }
        }
        let _ = self
            .0
            .store
            .revoke_principal(&format!("agent:{}", agent.name))
            .await;
        for state in [
            MessageState::Pending,
            MessageState::Held,
            MessageState::Written,
        ] {
            for m in self.0.store.messages_for_agent(&agent.id, &[state]).await? {
                self.0
                    .store
                    .set_message_state(&m.id, MessageState::Dropped, Utc::now())
                    .await?;
            }
        }
        self.0.store.delete_agent(&agent.id).await?;
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::AGENT_REMOVED,
                principal.id.clone(),
                Some(agent.id.clone()),
                json!({}),
            )
            .await;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ToTarget {
    Human,
    Agent(String),
    /// A `bridle token create`-minted external principal, addressed by its
    /// full id (`external:<name>`).
    External(String),
}

#[allow(clippy::too_many_arguments)]
fn agent_env(
    ws: &Workspace,
    url: &str,
    project: &str,
    id: &str,
    name: &str,
    token: &str,
    components: &[String],
) -> Vec<(String, String)> {
    let mut env = vec![
        ("BRIDLE_URL".to_string(), url.to_string()),
        ("BRIDLE_TOKEN".to_string(), token.to_string()),
        ("BRIDLE_AGENT_ID".to_string(), id.to_string()),
        ("BRIDLE_AGENT_NAME".to_string(), name.to_string()),
        (
            "BRIDLE_WORKSPACE".to_string(),
            ws.workspace.to_string_lossy().into_owned(),
        ),
        ("BRIDLE_PROJECT".to_string(), project.to_string()),
        ("PATH".to_string(), agent_path()),
    ];
    if !components.is_empty() {
        env.push(("BRIDLE_COMPONENTS".to_string(), components.join(",")));
    }
    env
}

/// Agents call `bridle` from their Bash tool, so the binary running this
/// daemon goes first on their PATH: they always talk to the same version,
/// even when it isn't installed (e.g. `cargo run`).
fn agent_path() -> String {
    let inherited = std::env::var("PATH").unwrap_or_default();
    match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        Some(dir) if inherited.is_empty() => dir.to_string_lossy().into_owned(),
        Some(dir) => format!("{}:{inherited}", dir.to_string_lossy()),
        None => inherited,
    }
}

fn principal_display(id: &str) -> String {
    if id == "human" {
        "human".to_string()
    } else if let Some(name) = id.strip_prefix("agent:") {
        format!("agent {name}")
    } else if let Some(name) = id.strip_prefix("external:") {
        format!("external {name}")
    } else {
        id.to_string()
    }
}

fn reply_target(id: &str) -> String {
    if id == "human" {
        "human".to_string()
    } else if let Some(name) = id.strip_prefix("agent:") {
        name.to_string()
    } else if let Some(name) = id.strip_prefix("external:") {
        name.to_string()
    } else {
        id.to_string()
    }
}

fn format_delivery(msg: &Message) -> String {
    let mut s = format!(
        "[bridle message {} from {}]\n{}",
        msg.id,
        principal_display(&msg.from),
        msg.body
    );
    if msg.kind == MessageKind::Question {
        s.push_str(&format!(
            "\nReply with: bridle send {} --reply-to {} \"<answer>\"",
            reply_target(&msg.from),
            msg.id
        ));
    }
    s
}

async fn write_message(
    store: &Store,
    runtime: &Arc<AgentRuntime>,
    msg: &Message,
) -> Result<(), SupervisorError> {
    let text = format_delivery(msg);
    // Queued *before* the process ever sees the text (htp6b's stress test
    // caught this under 30-agent SQLite contention, but the race is
    // general): `send_user` can trigger an echo fast enough that the
    // daemon's own replay-matching code runs before this function reaches
    // the `fifo.push_back` that used to come after `send_user` — with
    // nothing in the fifo yet, that match silently fails, the message
    // never reaches `Delivered`, and the plain `Written` set below (a
    // separate, slower store round trip on a separate task from the
    // replay handler's) can even land *after* and stomp a `Delivered` that
    // did land, so `mark_message_written` only applies over `pending`.
    {
        let mut st = runtime.state.lock().await;
        st.fifo.push_back((msg.id.clone(), text.clone()));
    }
    if runtime.handle.send_user(&text).is_err() {
        let mut st = runtime.state.lock().await;
        if let Some(pos) = st.fifo.iter().position(|(id, _)| *id == msg.id) {
            st.fifo.remove(pos);
        }
        return Err(SupervisorError::Internal("stdin is closed".to_string()));
    }
    store.mark_message_written(&msg.id, Utc::now()).await?;
    Ok(())
}

/// Reason on `ExitInfo` for an agent stopped as part of daemon shutdown
/// (`AgentManager::stop_all`), distinct from an ordinary `bridle stop`'s
/// `sigterm`/`sigkill`/`stdin_closed`. `run_autostart_and_resume` treats a
/// `resume_on_restart` role stopped this way the same as `lost`, so it comes
/// back after a clean restart, not only after a crash.
pub const DAEMON_SHUTDOWN_REASON: &str = "daemon_shutdown";

fn classify_exit(
    stop_requested: bool,
    shutdown_requested: bool,
    saw_any_line: bool,
    outcome: &ExitOutcome,
) -> (AgentState, ExitInfo) {
    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;

    if stop_requested {
        let reason = if shutdown_requested {
            DAEMON_SHUTDOWN_REASON
        } else {
            match outcome.signal {
                Some(SIGKILL) => "sigkill",
                Some(SIGTERM) => "sigterm",
                _ => "stdin_closed",
            }
        };
        return (
            AgentState::Stopped,
            ExitInfo {
                code: outcome.code,
                signal: outcome.signal,
                reason: reason.to_string(),
            },
        );
    }

    let clean = outcome.signal.is_none() && matches!(outcome.code, Some(0) | Some(1));
    if clean && saw_any_line {
        return (
            AgentState::Exited,
            ExitInfo {
                code: outcome.code,
                signal: outcome.signal,
                reason: "eof".to_string(),
            },
        );
    }

    let tail = outcome.stderr_tail.join("; ");
    let reason = if tail.is_empty() {
        "crashed".to_string()
    } else {
        format!("crashed: {}", truncate_chars(&tail, 500))
    };
    (
        AgentState::Crashed,
        ExitInfo {
            code: outcome.code,
            signal: outcome.signal,
            reason,
        },
    )
}

fn input_summary(input: &Value) -> String {
    for key in ["command", "file_path", "pattern"] {
        if let Some(s) = input.get(key).and_then(Value::as_str) {
            return truncate_chars(s, TRUNCATE_SUMMARY);
        }
    }
    truncate_chars(&input.to_string(), TRUNCATE_SUMMARY)
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}
