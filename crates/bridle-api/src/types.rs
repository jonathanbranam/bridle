//! Wire types for the bridle daemon API (`/v1`). Shared by the daemon, the
//! CLI and any future TUI/GUI/MCP client. See docs/design/agent-host/api.md.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const API_VERSION: &str = "v1";

/// `human`, `agent:<name>`, `external:<name>`, `system` or `local`.
pub type PrincipalId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Agent,
    External,
    /// Another daemon forwarding mail (`peer:<machine>`, `bridle token create --peer`). The only
    /// kind whose claim about the original sender is believed (principals.md, "Peer tokens").
    Peer,
    System,
    /// A GET/HEAD request with no bearer token, synthesized by the daemon
    /// rather than authenticated against a stored principal (see
    /// `docs/design/agent-host/principals.md`, "Read access without a
    /// token"). Never persisted, and never reaches a write route.
    Local,
}

// ---------- health / status / discovery ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub ok: bool,
    pub version: String,
    /// Non-terminal agents on this daemon, for `bridle daemons` to show
    /// without probing every daemon's authenticated status endpoint.
    pub agent_count: u32,
}

/// Reply to `POST /v1/shutdown`: the daemon's cap on stopping its agents
/// (`stop_grace` + 5 s), so `stop-daemon` can say how long to expect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShutdownResponse {
    pub stop_limit_secs: u64,
}

/// `POST /v1/restart`. A plain restart drains the daemon (no new turns) and answers once it has
/// decided to go, however long the running turns take; there is no timeout.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RestartRequest {
    /// Build the newest green commit on the integration branch first, then restart into it. The
    /// reply comes at once; the build runs in the background and the drain starts after it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub upgrade: bool,
}

fn yes() -> bool {
    true
}

/// Reply to `POST /v1/restart`, sent once the daemon has decided to go: it stops its agents and
/// execs itself next, then resumes `agents`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RestartResponse {
    /// The integration branch's head before the restart (short sha).
    #[serde(default)]
    pub commit: Option<String>,
    /// Names of the agents that were running and will be resumed.
    pub agents: Vec<String>,
    /// The cap on stopping them (`stop_grace` + 5 s).
    pub stop_limit_secs: u64,
    /// False for an upgrade that isn't restarting yet: nothing newer to build (`message` says
    /// so) or a build under way (the daemon restarts itself when it's done).
    #[serde(default = "yes")]
    pub restarting: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Written by the daemon to `<workspace>/.bridle/daemon.json` and to the
/// machine registry `~/.bridle/daemons/<project>.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonInfo {
    pub project: String,
    pub workspace: String,
    pub repo: String,
    pub url: String,
    pub pid: i32,
    pub started_at: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub daemon: DaemonInfo,
    /// The principal the request was authenticated as.
    pub principal: PrincipalId,
    /// Agent counts keyed by `AgentState` in snake_case.
    pub agents_by_state: BTreeMap<String, u32>,
    pub unread_human_messages: u32,
    pub rate_limits: Vec<RateLimit>,
    /// The Claude Code version the daemon's agents last reported.
    #[serde(default)]
    pub claude_version: Option<String>,
    /// The budget governor's current state (usage-and-budget.md).
    #[serde(default = "GovernorState::default")]
    pub budget_state: GovernorState,
    /// The last finished CI run for the integration branch's tip; `None`
    /// until one finishes, or when `[ci] github` is off.
    #[serde(default)]
    pub ci: Option<CiStatus>,
    /// Names of stopped agents whose branch is already merged into the
    /// integration branch: leftovers `bridle task done` would have removed.
    #[serde(default)]
    pub merged_leftovers: Vec<String>,
    /// The state branch push (`[state] push`); `None` when pushing is off.
    #[serde(default)]
    pub state_push: Option<StatePushStatus>,
    /// Active incidents (tasks of kind `incident`, `planned`).
    #[serde(default)]
    pub incidents: Vec<IncidentSummary>,
    /// A `wait-for-wake` request is open now (the orchestrator is listening).
    #[serde(default)]
    pub waiter_open: bool,
    /// When the orchestrator's wake poll last answered with wakes; `None` since the daemon
    /// started if none has.
    #[serde(default)]
    pub last_wake_at: Option<DateTime<Utc>>,
    /// Short sha of the built commit a drain is restarting into; `None` for no drain or a plain
    /// restart's.
    #[serde(default)]
    pub upgrade_waiting: Option<String>,
    /// A restart is draining the daemon: no spawns, claims or new turns until it has restarted.
    #[serde(default)]
    pub draining: bool,
    /// While draining, the agents still mid-turn (what the restart waits on).
    #[serde(default)]
    pub draining_on: Vec<String>,
    /// Registered interactive sessions (advisors) still running.
    #[serde(default)]
    pub sessions: Vec<SessionInfo>,
    /// Tasks waiting for `bridle task ready`; created tasks start here, so a pile is
    /// something nobody has approved yet.
    #[serde(default)]
    pub pending_tasks: Vec<PendingTask>,
    /// The last machine load reading (`[machine]`); `None` until the first one, or when the
    /// watch is off.
    #[serde(default)]
    pub load: Option<LoadStatus>,
    /// Mail waiting in this daemon's outbox, one entry per destination daemon (3haz P6). Empty
    /// when nothing is queued.
    #[serde(default)]
    pub outbox: Vec<OutboxPeer>,
}

/// One destination daemon's queue in [`Status::outbox`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxPeer {
    /// The destination daemon's project.
    pub project: String,
    pub queued: u32,
    /// When the oldest queued message was accepted: "unreachable for" counts from here.
    pub oldest_queued_at: DateTime<Utc>,
    /// Why the last try failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// The machine load watch's last reading (docs/design/agent-host/operating-model.md, Load watch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadStatus {
    /// The 1-minute load average.
    pub load1: f64,
    pub cores: u32,
    /// `load1` divided by `cores`.
    pub per_core: f64,
    /// `[machine] load_per_core`; `holding` is `per_core` over it.
    pub threshold: f64,
    /// New agent spawns are held.
    pub holding: bool,
}

/// One `pending` task in [`Status::pending_tasks`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTask {
    pub id: String,
    pub title: String,
    pub created_by: String,
}

/// `POST /v1/sessions`: registers an interactive session, or updates the one with this pid
/// (the SessionStart hook adds the Claude session id; `/clear` changes it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRegister {
    /// `advisor` or `advisor/<name>`.
    pub identity: String,
    pub pid: i32,
    /// The pid's start time (`ps` lstart), so a reused pid is not the session.
    pub pid_start: String,
    #[serde(default)]
    pub pane: Option<String>,
    #[serde(default)]
    pub claude_session_id: Option<String>,
    /// The launcher's project (`bridle session --project`).
    #[serde(default)]
    pub project: Option<String>,
    /// The launcher's host name.
    #[serde(default)]
    pub machine: Option<String>,
}

/// `POST /v1/sessions/end`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEnd {
    pub pid: i32,
}

/// `POST /v1/sessions/keep`: the human's override of a session's planned handover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKeep {
    pub identity: String,
}

/// `POST /v1/review/now`: send a document's pending comment threads to its agent at once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewNowRequest {
    /// Repo-relative path, as registered with `bridle review add`.
    pub path: String,
    /// Send threads already marked sent too.
    #[serde(default)]
    pub resend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewNowResponse {
    pub path: String,
    pub agent: String,
    /// Threads sent; 0 when nothing was pending.
    pub threads: usize,
}

/// `POST /v1/review/add`: put a document under review, like `bridle review add`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAddRequest {
    /// Repo-relative path of an existing file.
    pub path: String,
    /// Add it only if it has a pending comment thread (the gateway's save of a UI comment).
    #[serde(default)]
    pub only_if_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAddResponse {
    pub path: String,
    /// Whether the document is under review now.
    pub under_review: bool,
}

/// One registered interactive session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub identity: String,
    pub pid: i32,
    #[serde(default)]
    pub pane: Option<String>,
    #[serde(default)]
    pub claude_session_id: Option<String>,
    pub started_at: DateTime<Utc>,
    /// The latest context reading; `None` until the session has reported one.
    #[serde(default)]
    pub tokens: Option<u64>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub machine: Option<String>,
    /// When the context file last changed (the statusline writes it on every update); `None`
    /// until the session has reported a context.
    #[serde(default)]
    pub last_activity: Option<DateTime<Utc>>,
}

/// One active incident, as `bridle status` lists it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentSummary {
    pub id: String,
    pub title: String,
    /// When it became active (its last state change).
    pub since: DateTime<Utc>,
}

/// How pushing `bridle/state` to origin is going.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePushStatus {
    /// The last successful push; `None` until one.
    #[serde(default)]
    pub last_pushed_at: Option<DateTime<Utc>>,
    /// Why the latest attempt failed; `None` when it worked or none was made.
    #[serde(default)]
    pub failing: Option<String>,
    /// The remote has commits this clone lacks: pushing has stopped.
    #[serde(default)]
    pub diverged: bool,
}

/// The outcome of GitHub Actions for one commit of the integration branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiStatus {
    pub sha: String,
    /// `success`, `failure` or `cancelled`.
    pub conclusion: String,
    #[serde(default)]
    pub url: Option<String>,
    pub completed_at: DateTime<Utc>,
}

// ---------- orchestrator wakes ----------

/// The wake reason a waiter gets when the daemon is stopping or restarting; its text says why.
/// The CLI exits 6 on it.
pub const DAEMON_STOPPING_WAKE: &str = "daemon_stopping";

/// The wake reason a waiter gets when it was ended (by a newer wait from the same session, or
/// `bridle agent wake --stop`). It marks nothing read. The CLI exits 5 on it.
pub const WAIT_SUPERSEDED_WAKE: &str = "superseded";

/// One reason to wake the orchestrator (`GET /v1/orchestrator/wake`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WakeReason {
    /// `agent_exited`, `agent_crashed`, `agent_stalled`, `question`, `message`, `usage`,
    /// `budget_hold`, `ci_failed`, or `context` (a context or uptime note).
    pub reason: String,
    /// One line for the model to read.
    pub text: String,
    /// The raw fact: the event, the message, the run.
    #[serde(default)]
    pub detail: Value,
}

/// The long poll's answer: empty when nothing came up before it timed out.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WakeResponse {
    pub wakes: Vec<WakeReason>,
}

/// `GET /v1/orchestrator/wake`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchestratorWakeQuery {
    /// Give up after this many seconds (default 25 minutes; the daemon caps it at 1 h 55 min).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// `GET /v1/wake`: hold until the daemon decides `principal` should wake.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrincipalWakeQuery {
    /// `external:advisor`, `external:advisor/<name>`, `human` or an agent name.
    pub principal: String,
    /// Give up after this many seconds (the daemon caps it at 1 h 55 min; absent is the cap).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// The launcher's session (`BRIDLE_SESSION_PID`): a newer wait from it replaces this one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    /// The waiter's process id, recorded on the `message.read` audit event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// `POST /v1/wake/stop`: end open waits. With `principal` those waiting as it, else those
/// started by `session`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopWakeRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
}

/// How many waits `POST /v1/wake/stop` ended.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopWakeResponse {
    pub stopped: usize,
}

/// One reason a principal should wake.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PrincipalWakeReason {
    /// `message` (the principal has unread messages, task updates included). `task` is no
    /// longer emitted: a task change is a `task_update` message to its watchers.
    pub reason: String,
    /// The messages behind a `message` reason.
    #[serde(default)]
    pub message_ids: Vec<String>,
    /// The messages themselves, in full. Returned to a non-human caller they are marked read
    /// by the same call; the human's stay unread until they say so.
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Unused since task changes became messages; kept so old clients still parse.
    /// The task behind a `task` reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    /// What happened to it: the event kind, e.g. `task.note_added`, `task.state`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// For `daemon_stopping`: why the daemon is stopping or restarting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// For the orchestrator's reasons (`agent_exited`, `usage`, `context`, ...): the raw fact,
    /// the same payload `wait-for-wake` has always returned. `message` carries it too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
}

/// The answer to `GET /v1/wake`: no reasons means the timeout came first.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PrincipalWakeResponse {
    pub reasons: Vec<PrincipalWakeReason>,
}

/// The answer to `POST /v1/orchestrator/handover`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandoverDone {
    pub marked_at: DateTime<Utc>,
}

// ---------- orchestrator handover notes ----------

/// One handover note (`bridle handover write`; orchestrator-supervision.md, section 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Handover {
    /// `h-0007`, from the insertion sequence.
    pub id: String,
    /// The writer's identity without `external:`/`@machine` (`aide`, `advisor/doc-review`,
    /// `agent:manager-2`); the human's notes are the orchestrator's.
    pub role: String,
    pub project: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    /// The writing principal.
    pub created_by: String,
}

/// The answer to `POST /v1/rebuild`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildResponse {
    /// What the fetch of `origin/bridle/state` did; absent unless `?from_origin=true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// `POST /v1/handovers`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteHandoverRequest {
    pub body: String,
}

// ---------- agents ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Starting,
    Idle,
    Working,
    Stopping,
    Stopped,
    Exited,
    Crashed,
    Lost,
}

impl AgentState {
    /// A live process exists (or is being started/stopped).
    pub fn is_running(self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Idle | Self::Working | Self::Stopping
        )
    }

    /// Can be restarted with `--resume`.
    pub fn is_resumable(self) -> bool {
        matches!(
            self,
            Self::Stopped | Self::Exited | Self::Crashed | Self::Lost
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Exited => "exited",
            Self::Crashed => "crashed",
            Self::Lost => "lost",
        }
    }
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AgentState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string()))
            .map_err(|_| format!("unknown agent state: {s}"))
    }
}

/// Where an agent runs. Serialized as `{"kind":"worktree","base":"HEAD"}` etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Workdir {
    /// New worktree `<workspace>/wt/<name>` on branch `bridle/<name>`.
    Worktree {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        base: Option<String>,
    },
    /// The clone's main checkout.
    Repo,
    /// An explicit existing directory.
    Path { path: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExitInfo {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    /// e.g. "stdin_closed", "sigterm", "sigkill", "eof", "startup_error: …", "daemon_restart"
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Stable id, e.g. `a-7f3k2`.
    pub id: String,
    /// Unique, human-friendly; usable wherever an agent is referenced.
    pub name: String,
    pub role: String,
    pub state: AgentState,
    pub model: String,
    /// Claude Code session id (used for `--resume`).
    pub session_id: String,
    pub pid: Option<i32>,
    pub cwd: String,
    pub worktree: Option<String>,
    pub branch: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_event_at: Option<DateTime<Utc>>,
    /// Completed turns.
    pub turns: u32,
    /// Set while `state == working`.
    pub turn_started_at: Option<DateTime<Utc>>,
    /// Cumulative for the session (survives resume).
    pub cost_usd_total: f64,
    /// Context size after the most recent turn (input + cache_read +
    /// cache_creation tokens from that turn's `result` event). Not
    /// cumulative like `cost_usd_total`: it's the latest known size, not a
    /// running total. `None` before any turn has ended.
    pub context_tokens: Option<u64>,
    pub exit: Option<ExitInfo>,
    pub created_by: PrincipalId,
    /// Messages held for `when=idle` delivery.
    pub held_messages: u32,
    /// Written to stdin but not yet acknowledged by replay echo.
    pub unacked_messages: u32,
    /// Component ids this agent is scoped to (`BRIDLE_COMPONENTS` in its
    /// env; docs/design/components.md). Empty = repo-wide.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpawnRequest {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// First user message; starts the first turn. None = spawn idle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// None = the role's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workdir: Option<Workdir>,
    /// None = the role's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Extra tools to grant for this one spawn only, beyond the role's
    /// `allowed_tools`. Never touches the role's `disallowed_tools`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_allowed_tools: Vec<String>,
    /// Environment variables set in this one spawn's process only (e.g. a
    /// secret). Never persisted: not written to the role or config, and not
    /// carried by any later spawn or resume of the same agent/role.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_env: Vec<(String, String)>,
    /// Skip the budget governor's holding/paused check for this one call
    /// (usage-and-budget.md, Resuming: the escape hatch).
    #[serde(default)]
    pub ignore_budget: bool,
    /// Component ids to scope the agent to. Empty = the claimed task's list
    /// (if the spawner has one), else repo-wide.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
}

/// `POST /v1/agents/{id}/resume`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResumeRequest {
    /// Skip the budget governor's holding/paused check for this one call.
    #[serde(default)]
    pub ignore_budget: bool,
}

/// `POST /v1/agents/{id}/renew`. Stops the agent if it's running and starts
/// its replacement fresh (`Session::New`, not `--resume`) in the same
/// worktree/branch/role/model — no new worktree is created.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenewRequest {
    /// Skip the budget governor's holding/paused check for this one call.
    #[serde(default)]
    pub ignore_budget: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterruptRequest {
    /// Also discard messages bridle is holding for `when=idle`.
    #[serde(default)]
    pub drop_held: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptResponse {
    /// The raw `control_response` from claude.
    pub receipt: Value,
    pub dropped_held: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopRequest {
    /// Skip the graceful stdin close; go straight to SIGTERM.
    #[serde(default)]
    pub now: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoveQuery {
    #[serde(default)]
    pub force: bool,
    #[serde(default)]
    pub delete_branch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptLine {
    /// 1-based line number in the transcript file; pass as `since` to page.
    pub n: u64,
    pub t_ms: u64,
    /// `in` | `out` | `err` | `note`
    pub dir: String,
    pub line: String,
}

/// `GET /v1/agents/{id}/transcript?since=&limit=`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptQuery {
    /// Return lines with `n > since`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// ---------- messages ----------

/// Not to be confused with [`TaskKind::Question`], a *kind of task* (research
/// work whose output is an answer, not code). A `MessageKind::Question` is a
/// message that blocks whatever task it's addressed to until answered
/// (docs/design/coordination.md, "Messages"); the two are unrelated and a
/// task of any `TaskKind` can have one addressed to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    Question,
    Answer,
    /// One line about a change to a task the recipient watches.
    TaskUpdate,
    /// A daemon's own news (a wake reason forwarded to the orchestrator's home daemon). Never
    /// counted as the human's unread or a to-do. Older builds read it as a `note`.
    System,
    /// Also what a kind this build doesn't know reads as (`other` must be last).
    #[default]
    #[serde(other)]
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum When {
    /// Write to stdin now (folds into a running turn at the next tool boundary).
    #[default]
    Now,
    /// Hold until the agent's current turn ends, then start a turn with it.
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageState {
    /// Accepted, not yet written (agent not running).
    Pending,
    /// Waiting for the agent's turn to end (`when=idle`).
    Held,
    /// Written to stdin, not yet echoed back.
    Written,
    /// Echoed by claude: the model has it. Terminal for agent recipients until read.
    Delivered,
    /// Marked read (human inbox, or agent via `inbox --mark-read`).
    Read,
    /// Discarded (e.g. `interrupt --drop-held`, agent removed).
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// e.g. `m-0042`
    pub id: String,
    pub from: PrincipalId,
    /// `human` or an agent id.
    pub to: String,
    pub kind: MessageKind,
    pub body: String,
    pub reply_to: Option<String>,
    pub when: When,
    pub state: MessageState,
    pub created_at: DateTime<Utc>,
    pub written_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    /// Set when a principal in `[messages] answer_for_human` replied to this
    /// message, which was addressed to the human: who answered, and the
    /// reply's id. An answered message is also `read`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answered_by: Option<PrincipalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answered_reply: Option<String>,
    /// The reply's first line, for showing in the inbox without a second fetch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answered_line: Option<String>,
    /// The incident task this notice announces; see incidents.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident_task: Option<String>,
    /// Set only on a send response: the named recipient isn't running, so the message waits in
    /// its inbox for its next session. Not stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_note: Option<String>,
}

/// `POST /v1/agents/{id}/messages` uses this with `to` ignored;
/// `POST /v1/messages` requires `to` (`human`, agent id or name, or
/// `role:<name>` for every live agent currently holding that role).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub body: String,
    #[serde(default)]
    pub kind: MessageKind,
    #[serde(default)]
    pub when: When,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    /// About this task: `body` is written in full as a note on the task's
    /// thread, and the recipient gets a short message naming the task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
}

/// `POST /v1/outbox`: mail for a principal on another daemon. The sender's own daemon accepts it
/// at once, keeps it in its outbox and delivers it to `project`'s daemon (3haz).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutboxSendRequest {
    /// The destination project (its daemon, on this machine or another).
    pub project: String,
    /// The recipient on that daemon, as `SendRequest::to`.
    pub to: String,
    pub body: String,
    #[serde(default)]
    pub kind: MessageKind,
    #[serde(default)]
    pub when: When,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// A scheduled message (`/v1/schedules`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    /// `sc-` and four characters.
    pub id: String,
    pub created_by: String,
    /// A principal as `bridle send` takes it (`external:orchestrator`, `agent:notes-1`).
    pub target: String,
    pub body: String,
    /// `once` or `cron`.
    pub kind: String,
    /// The cron expression, for `cron`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// The IANA zone the schedule is evaluated in.
    pub tz: String,
    /// The next firing; none once a `once` is `done`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_fire_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_fired_at: Option<DateTime<Utc>>,
    /// `active` or `done`.
    pub state: String,
    pub created_at: DateTime<Utc>,
}

/// `POST /v1/schedules`. Exactly one of `at` and `cron`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScheduleAddRequest {
    /// Defaults to the caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// RFC 3339, or `YYYY-MM-DD HH:MM` in `tz`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// Five fields: minute hour day-of-month month day-of-week.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// Defaults to the daemon's `[schedule] timezone`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    pub body: String,
}

/// `GET /v1/schedules`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleListQuery {
    /// Include `done` schedules.
    #[serde(default)]
    pub all: bool,
}

/// What the sender gets back at once: the outbox id and where it is going.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Queued {
    /// `o-0007`; also the message's id in the receiver's dedup record.
    pub id: String,
    pub project: String,
    pub to: String,
    /// Where the first try left it: `queued` (retrying), `delivered` or `failed` (refused for
    /// good). An older daemon sends none, which reads as `queued`.
    #[serde(default = "queued_state")]
    pub state: String,
    /// Why the first try did not deliver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

fn queued_state() -> String {
    "queued".to_string()
}

/// `GET /v1/outbox/{id}`: where a message the caller sent to another daemon has got to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub id: String,
    pub project: String,
    pub to: String,
    /// `queued` (in this daemon's outbox), `arrived` (stored on the recipient's daemon, the
    /// recipient not yet woken), `delivered` (the recipient received it; also read, for now), or
    /// `failed` (refused for good, or dropped on the far side).
    pub state: String,
    pub attempts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub queued_at: DateTime<Utc>,
    /// When the destination daemon accepted it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrived_at: Option<DateTime<Utc>>,
    /// The message ids on the destination daemon, once it has accepted it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_ids: Vec<String>,
}

/// `GET /v1/forward/{message_id}`: a message the caller forwarded here, as the sender's daemon
/// asks after it for `GET /v1/outbox/{id}`. Peer tokens only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardState {
    pub state: MessageState,
}

/// One address a session can send to (`GET /v1/recipients`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipient {
    /// As `bridle send` takes it.
    pub address: String,
    /// What it is: `human`, `agent`, `external principal` or `visitor`.
    pub kind: String,
}

/// `POST /v1/hello`: a daemon that has just started (or woken) tells a peer so; the peer flushes
/// its outbox to it at once. Peer tokens only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HelloRequest {
    /// The greeting daemon's project: the key of the receiver's outbox queue for it.
    pub daemon: String,
}

/// `POST /v1/forward`: one message from another daemon's outbox. Peer tokens only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardRequest {
    /// Where the message started: the same triple on every retry is how a repeat is spotted.
    pub origin_machine: String,
    pub origin_daemon: String,
    pub origin_id: String,
    /// The original sender, qualified with its machine (`agent:w1@nuc`).
    pub from: PrincipalId,
    pub to: String,
    pub body: String,
    #[serde(default)]
    pub kind: MessageKind,
    #[serde(default)]
    pub when: When,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// The acknowledgement: the receiver has the message (as these ids); a repeat gets the same.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardAck {
    pub message_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageQuery {
    /// `human`, agent id/name, or `me` (the calling principal's inbox).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default)]
    pub unread: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Only this message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Mark what is listed read, when the caller is an agent or an external principal and the
    /// messages are its own. Ignored for the human, whose reads are explicit.
    #[serde(default)]
    pub mark_read: bool,
    /// Only messages created within this many seconds before now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since_secs: Option<u64>,
}

// ---------- events ----------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Monotonic; also the SSE event id.
    pub seq: i64,
    pub ts: DateTime<Utc>,
    /// See [`event_kind`].
    pub kind: String,
    pub actor: PrincipalId,
    /// Agent id, when the event concerns one.
    pub agent: Option<String>,
    pub data: Value,
}

/// Known event kinds. Clients must tolerate kinds not listed here.
/// Values of the `channel` field on `message.*` events.
pub mod channel {
    /// `bridle agent wake`: delivered when the waiter was handed it, read (`acknowledged`) when
    /// its session next ran a wake or inbox. The orchestrator's wake queue marks read on hand-over.
    pub const WAITER: &str = "waiter";
    /// `bridle inbox` listing or showing it.
    pub const INBOX: &str = "inbox";
    /// The human's web UI or TUI (the human principal).
    pub const UI: &str = "ui";
    /// The email bridge.
    pub const MAIL: &str = "mail";
    /// Sent by a schedule firing.
    pub const SCHEDULE: &str = "schedule";
    /// Any other HTTP API caller.
    pub const API: &str = "api";
    /// Typed into a running agent's session by the supervisor.
    pub const AGENT: &str = "agent";
}

pub mod event_kind {
    pub const DAEMON_STARTED: &str = "daemon.started";
    pub const DAEMON_STOPPING: &str = "daemon.stopping";
    /// A schedule's message was sent on time. data: {id, target}.
    pub const SCHEDULE_FIRED: &str = "schedule.fired";
    /// A schedule's message was sent late (the daemon was down or asleep). data: {id, target}.
    pub const SCHEDULE_MISSED_FIRED: &str = "schedule.missed_fired";
    /// Self-upgrade steps (actor: whoever asked, `system` for the automatic one). data always has
    /// `commit`. Skipped: {reason}, nothing the binary is built from changed.
    pub const UPGRADE_SKIPPED: &str = "upgrade.skipped";
    pub const UPGRADE_BUILDING: &str = "upgrade.building";
    /// The build passed its self-check; the drain starts.
    pub const UPGRADE_BUILT: &str = "upgrade.built";
    /// The drain after a good build began: no new turns until the restart.
    pub const UPGRADE_DRAINING: &str = "upgrade.draining";
    /// Build, self-check or restart failed. data: {error}.
    pub const UPGRADE_FAILED: &str = "upgrade.failed";
    /// The upgraded binary failed to start and the previous one is back. data: {error}.
    pub const UPGRADE_ROLLED_BACK: &str = "upgrade.rolled_back";
    /// A restart's exec failed; the daemon started again in the same process. data: {error}.
    pub const RESTART_FAILED: &str = "restart.failed";
    /// Machine load went over the threshold; new spawns are held.
    /// data: {load1, cores, per_core, threshold, consumers}.
    pub const LOAD_HOLD_STARTED: &str = "load.hold.started";
    /// The load fell; spawns resume. data: {held_secs}, so total hold time can be summed.
    pub const LOAD_HOLD_ENDED: &str = "load.hold.ended";
    pub const AGENT_SPAWNED: &str = "agent.spawned";
    /// data: {from, to}
    pub const AGENT_STATE: &str = "agent.state";
    /// data: ExitInfo
    pub const AGENT_EXITED: &str = "agent.exited";
    pub const AGENT_STALLED: &str = "agent.stalled";
    /// data: {count}
    pub const AGENT_ORPHANS_KILLED: &str = "agent.orphans_killed";
    pub const AGENT_REMOVED: &str = "agent.removed";
    /// data: {dropped_held}. The actor is whoever asked.
    pub const AGENT_INTERRUPTED: &str = "agent.interrupted";
    /// The actor is whoever asked; the state change follows as `agent.state`.
    pub const AGENT_STOP_REQUESTED: &str = "agent.stop_requested";
    /// The actor is whoever asked (`system` for a restart).
    pub const AGENT_RESUMED: &str = "agent.resumed";
    /// A fresh process, fresh session, same worktree/branch/role/model as
    /// the agent it replaced (`bridle renew`). data: {from} (the old state).
    pub const AGENT_RENEWED: &str = "agent.renewed";
    /// data: {cost_total}. The role's `max_budget_usd` is spent; bridle
    /// stops the agent, and `resume` grants a fresh allowance.
    pub const AGENT_BUDGET_EXHAUSTED: &str = "agent.budget_exhausted";
    /// data: {version, previous}. The Claude Code version seen in an agent's
    /// `system/init` differs from the last one this daemon saw.
    pub const CLAUDE_VERSION: &str = "claude.version";
    /// data: {text} (≤ 2 KB)
    pub const AGENT_TEXT: &str = "agent.text";
    /// data: {name, input_summary}
    pub const TOOL_USE: &str = "tool.use";
    /// data: {denials}
    pub const PERMISSION_DENIED: &str = "permission.denied";
    pub const TURN_STARTED: &str = "turn.started";
    /// data: {n, subtype, is_error, terminal_reason, usage, cost_total, result}
    pub const TURN_ENDED: &str = "turn.ended";
    /// data: {message, to, channel?}. The `message.*` events carry an optional `channel` (see
    /// [`channel`]) saying how the step happened; the actor is who, `ts` is when. A `waiter`
    /// read adds `pid` and `session`. Old events have none of these.
    pub const MESSAGE_SENT: &str = "message.sent";
    /// data: {message, channel?, pid?, session?}
    pub const MESSAGE_DELIVERED: &str = "message.delivered";
    /// data: {message, channel?, pid?, session?, acknowledged?}
    pub const MESSAGE_READ: &str = "message.read";
    pub const MESSAGE_DROPPED: &str = "message.dropped";
    /// data: {info}
    pub const RATE_LIMIT: &str = "rate_limit";
    /// data: {from, to, window, utilization, resets_at, reason}. Emitted on
    /// every governor state transition (usage-and-budget.md, the budget
    /// governor).
    pub const BUDGET_STATE: &str = "budget.state";
    /// data: {sha, conclusion, url}. Every GitHub Actions run for the
    /// integration branch's new tip has finished (`[ci] github`).
    pub const CI_COMPLETED: &str = "ci.completed";
    /// data: {project, branch, error}. `bridle push` was rejected (or git failed): `error` is
    /// git's first stderr line. The orchestrator and the human are messaged too (ticket 8umh).
    pub const PUSH_FAILED: &str = "push.failed";
    /// data: {free_bytes, total_bytes, target_bytes, worktrees_bytes, data_bytes}. The
    /// periodic disk usage reading (`[disk]`).
    pub const DISK_CHECKED: &str = "disk.checked";
    /// data: {integration, ahead, behind}. The local integration branch differs from
    /// `origin/<integration>` (zeros when back in step; ticket k6jd).
    pub const GIT_DIVERGED: &str = "git.diverged";
    /// data: {text}. The orchestrator supervisor needs the human (interim, until incidents
    /// exist: docs/design/agent-host/orchestrator-supervision.md, section 8).
    pub const ORCHESTRATOR_INCIDENT: &str = "orchestrator.incident";
    /// data: {session, tokens, window_size, uptime_secs}. Session's starting context and growth.
    pub const ORCHESTRATOR_CONTEXT: &str = "orchestrator.context";
    /// data: {identity, session, tokens, threshold, step}. A registered interactive session's
    /// context crossed one of its `[sessions] warn` steps (0 warn, 1 plan a handover, 2 ceiling,
    /// 3 hard limit: interactive sessions, jttf).
    pub const SESSION_CONTEXT: &str = "session.context";
    /// data: {identity, tokens, step}. The human said to carry on past a session's last step.
    pub const SESSION_OVERRIDE: &str = "session.override";
    /// data: {identity, pid}. A registered interactive session ended (or its pid is gone).
    pub const SESSION_ENDED: &str = "session.ended";
    /// data: {task, branch}. `bridle land` began merging.
    pub const INTEGRATE_STARTED: &str = "integrate.started";
    /// data: {task, branch, ok, commit?, error?}
    pub const INTEGRATE_FINISHED: &str = "integrate.finished";
    pub const TASK_CREATED: &str = "task.created";
    /// data: {from, to}
    pub const TASK_STATE: &str = "task.state";
    /// data: {fields}, the names of the fields that changed.
    pub const TASK_EDITED: &str = "task.edited";
    /// data: {task, from, to}
    pub const TASK_PRIORITY: &str = "task.priority";
    /// data: {task, from, to}
    pub const TASK_KIND: &str = "task.kind";
    /// data: {from, to, kind}
    pub const EDGE_ADDED: &str = "edge.added";
    /// data: {from, to, kind}
    pub const EDGE_REMOVED: &str = "edge.removed";
    /// data: {task}
    pub const TASK_QUESTION_ASKED: &str = "task.question_asked";
    /// data: {task}
    pub const TASK_QUESTION_ANSWERED: &str = "task.question_answered";
    /// data: {task, reason}
    pub const TASK_SETTLE_SKIPPED: &str = "task.settle_skipped";
    /// data: {task, watching} (false: stopped watching). The actor is the principal that
    /// changed its own watching.
    pub const TASK_WATCHING: &str = "task.watching";
    /// data: {task}
    pub const TASK_NOTE_ADDED: &str = "task.note_added";
    /// data: {tiers} (the tier count after the change)
    pub const QUEUE_CHANGED: &str = "queue.changed";
    /// data: [`MigrationRecord`]. `bridle migrate` applied one project migration.
    pub const PROJECT_MIGRATED: &str = "project.migrated";
}

/// Body of `POST /v1/migrations`: one applied project migration, recorded as a
/// `project.migrated` event so agents can read it (`bridle events --kind project.migrated`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationRecord {
    pub id: String,
    pub files: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventQuery {
    /// Return events with seq > since.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Prefix match, e.g. `message.` or `agent.state`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Only events about this message (`data.message`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Only events about messages addressed to this principal (resolved like `MessageQuery::to`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

// ---------- usage ----------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimit {
    /// `five_hour`, `seven_day`, …
    pub window: String,
    /// `allowed` | `allowed_warning` | `rejected`, when known.
    pub status: Option<String>,
    /// 0–1 fraction.
    pub utilization: Option<f64>,
    pub resets_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
}

/// One reading in a window's history (`GET /v1/usage/history`): kept when the utilization or
/// `resets_at` changed, so a flat stretch is one point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimitPoint {
    pub window: String,
    /// 0-1 fraction.
    pub utilization: Option<f64>,
    pub resets_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistoryQuery {
    /// `five_hour`, `seven_day`, ...
    pub window: String,
    /// Only readings observed at or after this time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TokenTotals {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsage {
    pub agent: String,
    pub name: String,
    pub role: String,
    pub model: String,
    pub turns: u32,
    pub tokens: TokenTotals,
    pub cost_usd_total: f64,
    /// Sum of each turn's `ended_at - started_at`.
    #[serde(default)]
    pub busy_seconds: u64,
    /// MIN(started_at) to MAX(ended_at) across the agent's turns. `None`
    /// until at least one turn has ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_seconds: Option<u64>,
    /// The agent was removed with `rm`; its turns still count.
    #[serde(default)]
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub agents: Vec<AgentUsage>,
    pub total_turns: u32,
    pub total_tokens: TokenTotals,
    /// cache_read / (input + cache_read + cache_write); None when no tokens yet.
    pub cache_hit_ratio: Option<f64>,
    pub total_cost_usd: f64,
    pub rate_limits: Vec<RateLimit>,
    /// Today's `bridle statusline` snapshots: interactive sessions bridle
    /// doesn't host, so there's no agent to attach them to.
    #[serde(default)]
    pub interactive_today: Vec<InteractiveUsageRow>,
}

/// `GET /v1/usage/breakdown` grouping: role and model aggregate turns across
/// every agent that shares one, since there's no task/workflow-revision
/// column yet to group by (docs/design/usage-and-budget.md, "Tracking token
/// use over time"). `agent` groups the same way `GET /v1/usage` already does,
/// but through the turns ledger so `--since` applies to it too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageGroupBy {
    Role,
    Model,
    Agent,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageBreakdownQuery {
    /// Only turns started at or after this time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<UsageGroupBy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageGroup {
    /// The role name, model name, or agent id, depending on `by`.
    pub key: String,
    pub turns: u32,
    pub tokens: TokenTotals,
    pub cost_usd_total: f64,
    /// cache_read / (input + cache_read + cache_write) for this group alone.
    pub cache_hit_ratio: Option<f64>,
    /// Sum of each turn's `ended_at - started_at` in the group. Sums
    /// meaningfully regardless of grouping.
    pub busy_seconds: u64,
    /// MIN(started_at) to MAX(ended_at) across the group's turns. Only set
    /// when grouped by agent: for `role`/`model`, unrelated agents' turns
    /// can overlap, so a wall-clock span wouldn't mean anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBreakdown {
    pub by: UsageGroupBy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
    pub groups: Vec<UsageGroup>,
    pub total_turns: u32,
    pub total_tokens: TokenTotals,
    pub cache_hit_ratio: Option<f64>,
    pub total_cost_usd: f64,
}

// ---------- interactions ----------

/// One line of the machine's `~/.bridle/prompts.jsonl` (written by `bridle focus gate`,
/// docs/design/agent-host/roles-and-config.md): the human sent a prompt in an interactive
/// session, or (`event: reply`) the agent finished answering. Every field but `at` may be
/// missing in the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interaction {
    pub at: DateTime<Utc>,
    /// Lines from before the field existed are prompts.
    #[serde(default)]
    pub event: InteractionEvent,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub machine: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionEvent {
    #[default]
    Prompt,
    Reply,
}

/// `GET /v1/interactions` query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InteractionsQuery {
    /// Only prompts at or after this time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
}

// ---------- statusline ----------

/// `POST /v1/statusline` body: one snapshot from `bridle statusline`, read
/// off Claude Code's own statusLine stdin JSON. That schema isn't pinned
/// down in bridle's own docs (docs/design/usage-and-budget.md), so the CLI
/// parses it tolerantly (crates/bridle/src/statusline.rs, matching
/// bridle-claude's event-parsing convention) and only ever sends fields it
/// actually found.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusLineReport {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// `context_window.used_percentage` (0–100), normalized to 0–1 like
    /// `StatusLineRateLimitReading::utilization`. Precomputed by Claude Code
    /// from the last API response's input tokens; shown directly rather than
    /// recomputed from `context_used_tokens`/`context_max_tokens`, since it
    /// stays correct on extended-context (1M) models.
    #[serde(default)]
    pub context_used_percentage: Option<f64>,
    /// Sum of `context_window.current_usage`'s three input-token fields
    /// (input, cache creation, cache read) from the last API call. `None`
    /// before the first call, or right after `/compact`.
    #[serde(default)]
    pub context_used_tokens: Option<u64>,
    /// `context_window.context_window_size`: 200000, or 1000000 for
    /// extended-context models.
    #[serde(default)]
    pub context_max_tokens: Option<u64>,
    #[serde(default)]
    pub rate_limits: Vec<StatusLineRateLimitReading>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusLineRateLimitReading {
    /// `five_hour`, `seven_day`, …
    pub window: String,
    /// 0–1 fraction, normalized from Claude Code's `used_percentage`.
    #[serde(default)]
    pub utilization: Option<f64>,
    #[serde(default)]
    pub resets_at: Option<DateTime<Utc>>,
}

/// One recorded `bridle statusline` invocation, kept distinct from
/// per-agent turns (`AgentUsage`) since there's no hosted agent to attach it
/// to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveUsageRow {
    pub observed_at: DateTime<Utc>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub context_used_percentage: Option<f64>,
    pub context_used_tokens: Option<u64>,
    pub context_max_tokens: Option<u64>,
}

// ---------- budget governor ----------

/// The governor's state, most severe first. `usage-and-budget.md`, The
/// budget governor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernorState {
    #[default]
    Normal,
    Holding,
    WindingDown,
    Paused,
}

impl GovernorState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Holding => "holding",
            Self::WindingDown => "winding_down",
            Self::Paused => "paused",
        }
    }
}

impl std::fmt::Display for GovernorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One window's reading and the state it alone implies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowStatus {
    /// `five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet`, …
    pub window: String,
    pub state: GovernorState,
    pub status: Option<String>,
    /// 0-1 fraction.
    pub utilization: Option<f64>,
    pub resets_at: Option<DateTime<Utc>>,
    pub observed_at: Option<DateTime<Utc>>,
    /// No reading at all, or one older than `max_staleness`.
    pub stale: bool,
    /// Seconds since `observed_at`, computed by the daemon.
    #[serde(default)]
    pub age_secs: Option<u64>,
}

/// The `five_hour` thresholds in force and where they come from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppliedThresholds {
    /// `override` | `schedule` | `default`.
    pub source: String,
    /// The override's or schedule period's name; `None` for `default`.
    pub period: Option<String>,
    pub hold_at: f64,
    pub wind_down_at: f64,
    pub stop_at: f64,
    /// The current schedule period's span (`days` + `start`/`end`), when the
    /// thresholds come from a period.
    #[serde(default)]
    pub span: Option<ScheduleSpan>,
    /// The current period's `max_workers`, when it sets one.
    #[serde(default)]
    pub max_workers: Option<u32>,
    /// When the schedule next changes on its own, and to what.
    #[serde(default)]
    pub next_change: Option<NextScheduleChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduleSpan {
    /// `mon`, `tue`, …
    pub days: Vec<String>,
    /// Host-local `HH:MM`.
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NextScheduleChange {
    pub at: DateTime<Utc>,
    /// `None` = back to the plain `[budget]` defaults.
    pub period: Option<String>,
    pub hold_at: f64,
    pub wind_down_at: f64,
    pub stop_at: f64,
}

/// One resolved `[[budget.schedule]]` period.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchedulePeriodInfo {
    pub name: String,
    /// `None` for a schedule-less preset (only `bridle budget override`).
    #[serde(default)]
    pub span: Option<ScheduleSpan>,
    pub hold_at: f64,
    pub wind_down_at: f64,
    pub stop_at: f64,
    /// Applied as the live `max_workers` override while this period is
    /// forced with `bridle budget override`.
    #[serde(default)]
    pub max_workers: Option<u32>,
}

/// The effective thresholds in force, account-wide values merged with any
/// (lower) project overrides. Percentages, 0-100, matching config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetThresholds {
    pub max_workers: u32,
    pub hold_at: BTreeMap<String, f64>,
    pub wind_down_at: BTreeMap<String, f64>,
    pub stop_at: BTreeMap<String, f64>,
    pub resume_below: BTreeMap<String, f64>,
    pub wind_down_grace_secs: u64,
    pub max_staleness_secs: u64,
}

/// `GET /v1/budget`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// The overall (default-scoped windows + staleness) governor state.
    pub state: GovernorState,
    pub windows: Vec<WindowStatus>,
    /// The thresholds in force: the `five_hour` entries are the applied ones
    /// (see `five_hour`), not the plain config.
    pub thresholds: BudgetThresholds,
    /// Where the applied `five_hour` thresholds come from, and the next change.
    pub five_hour: AppliedThresholds,
    /// The whole resolved schedule, in match order.
    #[serde(default)]
    pub schedule: Vec<SchedulePeriodInfo>,
    /// Why the state is what it is: one line per window (or `staleness`)
    /// that is above `normal`.
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Set while `bridle budget hold` is in force (usage-and-budget.md, The
    /// human's hold).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_hold: Option<HoldStatus>,
    /// Set while `bridle budget override` is in force (usage-and-budget.md,
    /// Schedule override).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule_override: Option<ScheduleOverrideStatus>,
    /// Set while `bridle budget max-workers` has a live cap in force
    /// (usage-and-budget.md, Max-workers override); `thresholds.max_workers`
    /// stays the configured value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_workers_override: Option<u32>,
}

/// `POST /v1/budget/max-workers`: `None` clears the override and falls back
/// to the configured `max_workers`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxWorkersRequest {
    pub max_workers: Option<u32>,
}

/// `POST /v1/budget/hold`: `--for`/`--until` resolved to an absolute instant
/// by the CLI; `until: None` holds until `POST /v1/budget/release`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BudgetHoldRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldStatus {
    /// `None` = held until released.
    pub until: Option<DateTime<Utc>>,
}

/// `POST /v1/budget/override`: `--until` resolved to an absolute instant by
/// the CLI; `until: None` asks the daemon to compute the thermostat's
/// "until the schedule would next change on its own" instant itself, since
/// that needs the daemon's own `[[budget.schedule]]` config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetOverrideRequest {
    /// `None` = force the plain `[budget]` thresholds (`bridle budget
    /// override default`); `Some(name)` = force that `[[budget.schedule]]`
    /// period's thresholds.
    pub period: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleOverrideStatus {
    /// `None` = forced to the plain `[budget]` thresholds.
    pub period: Option<String>,
    /// `None` = no computed or given end (no schedule to revert to).
    pub until: Option<DateTime<Utc>>,
}

// ---------- tokens ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCreateRequest {
    /// Becomes principal `external:<name>`; `@` is refused (it marks a visitor).
    pub name: String,
    /// Mints a visitor, `external:<name>@<machine>`: another machine's principal on this daemon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    /// With `machine`: the project (daemon) on that machine the visitor's mail is forwarded to,
    /// as a destination this daemon can reach (3haz P2). Without it the visitor keeps a local
    /// inbox here, as tokens minted before this field do.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home: Option<String>,
}

/// `POST /v1/tokens/peer`: mints `peer:<machine>`, the token that daemon's forwarding presents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerTokenCreateRequest {
    /// The sending machine, as named in `[machine] name` there.
    pub machine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCreated {
    pub principal: PrincipalId,
    /// Shown once; the daemon stores only a hash.
    pub token: String,
}

/// One row of `GET /v1/tokens`. Never carries the token itself, which is
/// shown only once, at creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub principal: PrincipalId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
}

// ---------- tasks ----------

/// What kind of work a task is; changes gates and the agent's prime once
/// those exist (roles-and-lifecycle.md).
/// `Question` here is a *kind of task*, unrelated to [`MessageKind::Question`]
/// (a message that blocks the task it's addressed to). Don't conflate them:
/// a `TaskKind::Question` task can itself have a `MessageKind::Question`
/// asked against it, like any other task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskKind {
    Feature,
    Bug,
    Chore,
    Question,
    Research,
    Explore,
    ArchRevision,
    ReEvaluate,
    /// Owned by the orchestrator; active while `planned`, and never queued
    /// (docs/design/agent-host/incidents.md).
    Incident,
}

impl TaskKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Bug => "bug",
            Self::Chore => "chore",
            Self::Question => "question",
            Self::Research => "research",
            Self::Explore => "explore",
            Self::ArchRevision => "arch-revision",
            Self::ReEvaluate => "re-evaluate",
            Self::Incident => "incident",
        }
    }
}

impl std::fmt::Display for TaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TaskKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string()))
            .map_err(|_| format!("unknown task kind: {s}"))
    }
}

/// A rough estimate of a task's size, so small ones can be picked when the
/// budget runs short. Optional on a task; nothing derives or acts on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSize {
    #[serde(rename = "S")]
    S,
    #[serde(rename = "M")]
    M,
    #[serde(rename = "L")]
    L,
    /// Marker used in EditTaskRequest to clear a task's size.
    #[serde(rename = "none")]
    None,
}

impl TaskSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::S => "S",
            Self::M => "M",
            Self::L => "L",
            Self::None => "none",
        }
    }
}

impl std::fmt::Display for TaskSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How soon the requester wants a task done; ranks the human's to-dos. Agent work is ranked by
/// the queue's tiers, not this.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    Urgent,
    High,
    #[default]
    Normal,
    Low,
}

impl TaskPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Urgent => "urgent",
            Self::High => "high",
            Self::Normal => "normal",
            Self::Low => "low",
        }
    }

    pub fn is_normal(&self) -> bool {
        *self == Self::Normal
    }
}

/// Sorts the human's to-dos: higher level first; at `high` and above the most recently ranked
/// goes first (a newer urgent thing outranks older ones at its level), below that oldest first.
pub fn sort_by_priority(tasks: &mut [Task]) {
    tasks.sort_by_key(|t| {
        let at = t.priority_at.unwrap_or(t.created_at);
        let newest_first = t.priority <= TaskPriority::High;
        (
            t.priority,
            if newest_first {
                -at.timestamp_millis()
            } else {
                at.timestamp_millis()
            },
        )
    });
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `POST /v1/tasks/{id}/priority`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPriorityRequest {
    pub priority: TaskPriority,
}

/// `POST /v1/tasks/{id}/kind`; only while the task is `open`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetKindRequest {
    pub kind: TaskKind,
}

/// The lifecycle states this build knows about
/// ([[docs/design/roles-and-lifecycle#Task lifecycle|task lifecycle]]).
/// `in_review` and `accepted` arrive with later tasks that build on top of
/// this record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    /// Every task starts here. Nobody plans it until `bridle task ready` opens it (k7tm
    /// decisions 11-14).
    Pending,
    Open,
    Planned,
    /// A worker holds this task's lease (storage.md, "claims"). Only
    /// reachable from `planned`, and only returns to `planned` (release, or
    /// the lease expiring).
    Claimed,
    /// Requires a reason, recorded in the thread.
    Dropped,
    /// Merged; requires the merge commit, recorded in the thread. Terminal
    /// and resolves `blocks` edges; leaves the queue and `ready`.
    Integrated,
    /// A dropped or integrated task brought back.
    Reopened,
}

impl TaskState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Open => "open",
            Self::Planned => "planned",
            Self::Claimed => "claimed",
            Self::Dropped => "dropped",
            Self::Integrated => "integrated",
            Self::Reopened => "reopened",
        }
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for TaskState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string()))
            .map_err(|_| format!("unknown task state: {s}"))
    }
}

/// One entry in a task's thread, on the state branch
/// ([[docs/design/storage#The state branch|storage.md]]). `note`, `question`
/// and `answer` are produced by this build; `handoff`, `conflict` and
/// `system` (coordination.md, Messages) arrive with later tasks and reuse
/// this same shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadEntryKind {
    Note,
    Question,
    Answer,
}

impl ThreadEntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Question => "question",
            Self::Answer => "answer",
        }
    }
}

impl std::fmt::Display for ThreadEntryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ThreadEntryKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string()))
            .map_err(|_| format!("unknown thread entry kind: {s}"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadEntry {
    pub kind: ThreadEntryKind,
    pub from: PrincipalId,
    pub body: String,
    pub at: DateTime<Utc>,
}

/// `Task::created_by` when the creator isn't known.
pub const UNKNOWN_CREATOR: &str = "unknown";

fn unknown_creator() -> PrincipalId {
    UNKNOWN_CREATOR.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// e.g. `tw-7fa2`: a per-project prefix and a random suffix
    /// (docs/design/storage.md).
    pub id: String,
    pub title: String,
    pub kind: TaskKind,
    pub state: TaskState,
    /// The task's description. Durable only on the state branch: a crash
    /// between a write and the next batched flush can lose an edit to this
    /// (docs/design/storage.md).
    pub body: String,
    pub thread: Vec<ThreadEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// The principal that created the task; [`UNKNOWN_CREATOR`] for a task from before the
    /// field existed whose creator couldn't be recovered. Old task files and old daemons
    /// omit it, hence the default.
    #[serde(default = "unknown_creator")]
    pub created_by: PrincipalId,
    /// Principals who follow the task: the creator and the claimer are added automatically,
    /// anyone can `task watch`/`unwatch` themselves. Old task files and old daemons omit it.
    #[serde(default)]
    pub watchers: Vec<PrincipalId>,
    /// The claim, if any. Null when unclaimed. Kept in SQLite and mirrored
    /// to `claims.toml` on the state branch (docs/design/storage.md); the
    /// task file itself carries no claim, so it comes from the claim set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<PrincipalId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_at: Option<DateTime<Utc>>,
    /// Component ids the task is scoped to; empty = repo-wide. Naming a
    /// child implies its ancestors (docs/design/components.md).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    /// Estimated size; null when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<TaskSize>,
    /// Absent (normal) unless set; ranks the human's to-dos.
    #[serde(default, skip_serializing_if = "TaskPriority::is_normal")]
    pub priority: TaskPriority,
    /// When the priority was last set; orders to-dos within a level. Null until first set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_at: Option<DateTime<Utc>>,
    /// Branch that did the work, recorded by `task done --branch`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Commit that landed the task, recorded by `task done --commit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Short account of how it was implemented, set by `task summary`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The id of the ticket the task was made from (front matter `ticket`); null for a task
    /// with no ticket. Old task files and old daemons omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    /// The task this one was split from or made for (`task new --from`); it inherited that
    /// task's watchers. Null for an ordinary task; old task files and daemons omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Spec ids and file globs the task declares it will touch, set by
    /// `impact set` (docs/design/impact-and-conflicts.md). Empty = undeclared.
    #[serde(default, skip_serializing_if = "Impact::is_empty")]
    pub impact: Impact,
    /// While the task is still settling (ny9u): when it becomes startable.
    /// Computed by the daemon, never stored; null once settled.
    #[serde(default)]
    pub settle_until: Option<DateTime<Utc>>,
}

/// A task's declared impact. Ids are validated by shape only (`r-`/`s-`/`g-`/`a-`
/// plus hex); nothing checks they exist.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Impact {
    /// Scenario/requirement ids the task changes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modify: Vec<String>,
    /// Requirement/capability ids the task adds new spec under.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_under: Vec<String>,
    /// Spec ids the task removes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<String>,
    /// File globs the task touches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

impl Impact {
    pub fn is_empty(&self) -> bool {
        self.modify.is_empty()
            && self.add_under.is_empty()
            && self.remove.is_empty()
            && self.files.is_empty()
    }
}

/// `POST /v1/tasks/{id}/impact`: replaces the whole declared impact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetImpactRequest {
    pub impact: Impact,
}

/// Where a spec id lives, for `impact check`: the requirement (itself, for an `r-` id;
/// the parent, for an `s-` id) and the capability file it is in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecRef {
    pub requirement: String,
    pub capability: String,
}

/// `POST /v1/impact/check`. The client reads `design/specs` (the daemon doesn't) and
/// sends the id map; empty = skip the capability (info) level and match requirements
/// only by `r-` ids named directly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImpactCheckRequest {
    #[serde(default)]
    pub spec_map: std::collections::BTreeMap<String, SpecRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlapLevel {
    Info,
    Warn,
    Conflict,
}

/// One overlap between two in-flight tasks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overlap {
    pub level: OverlapLevel,
    /// The two tasks, in id order.
    pub tasks: [String; 2],
    /// `scenario`, `requirement`, `capability` or `files`.
    pub kind: String,
    /// The shared scenario/requirement/capability id, or the pair of globs.
    pub key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImpactReport {
    pub overlaps: Vec<Overlap>,
    /// Ids (`C12`) of conflicts this check opened; already-known overlaps aren't repeated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opened: Vec<String>,
    /// `git merge-tree` findings for claimed tasks' branches; clean merges aren't listed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub probes: Vec<MergeProbe>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeOutcome {
    Clean,
    Conflict,
    /// git older than 2.38, which has no `merge-tree --write-tree`.
    Unsupported,
}

/// The result of merging two branches in memory (`git merge-tree --write-tree`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeResult {
    pub branch: String,
    /// The branch merged into: the integration branch, or another task's branch.
    pub against: String,
    pub outcome: ProbeOutcome,
    /// Conflicting paths, for `conflict`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
}

/// One non-clean probe in an impact check. `conflict` level against the integration
/// branch, `warn` between two tasks' branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeProbe {
    pub level: OverlapLevel,
    /// The task, and the other task for a pairwise probe.
    pub tasks: Vec<String>,
    pub result: ProbeResult,
}

/// `POST /v1/probe`: one of a task id, an agent name or a branch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeRequest {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
}

/// A conflict-level overlap between two tasks that the claimants must settle
/// (impact-and-conflicts.md, "The conflict protocol").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    /// `C<n>`.
    pub id: String,
    /// The two tasks, in id order.
    pub tasks: [String; 2],
    /// The overlap's `kind` and `key` (see [`Overlap`]).
    pub kind: String,
    pub key: String,
    /// `open` or `resolved`.
    pub state: String,
    /// `compatible: <reason>`, `order: A blocks B` or `merge-into: A`, once resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    pub opened_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
}

/// A port handed out by `bridle port alloc` (worktrees-and-ports.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortAllocation {
    pub port: u16,
    /// The owning agent's stable id, or the principal id (`human`) for anyone else.
    pub agent: String,
    /// The task the owner had claimed when it allocated, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    /// The process that uses the port; the daemon frees the port once it's dead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub allocated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocPortRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Exactly one field is set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolveConflictRequest {
    /// Not a real conflict; the reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatible: Option<String>,
    /// `[first, second]`: the first blocks the second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<[String; 2]>,
    /// The task that absorbs the other's change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_into: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskRequest {
    pub title: String,
    pub kind: TaskKind,
    #[serde(default)]
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<TaskSize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    /// A human to-do: the task is created `planned` and claimed by the human
    /// principal in one step (coordination.md, "Human to-dos").
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub for_human: bool,
    /// Default normal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,
    /// The ticket the task is made from; the task's id takes the ticket's id when free.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    /// The task this one is split from: stored as its parent, and it inherits the parent's
    /// watchers (the creator is still added).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

/// A submission to the project's triage (`POST /v1/tasks/submit`): always an `open` task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskRequest {
    pub title: String,
    pub kind: TaskKind,
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditTaskRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Replaces the whole list; `Some(vec![])` clears it (repo-wide).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<TaskSize>,
    /// Sets the task's ticket link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropTaskRequest {
    pub reason: String,
}

/// `POST /v1/tasks/{id}/done`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DoneTaskRequest {
    /// May be empty only for a task claimed by the human (a to-do, no code).
    #[serde(default)]
    pub commit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// For an incident: how it ended, recorded in its thread and sent in the "resolved" note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// `POST /v1/push` result: the integration branch was pushed to origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub branch: String,
}

/// `POST /v1/tasks/{id}/land`: the integrator merges the task's branch, checks it, and marks
/// the task done.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LandRequest {
    /// Defaults to the claimant's branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Overrides `[integration] check`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_cmd: Option<String>,
    /// The commit the worker reported a green `just check` on. The check is skipped only when
    /// the branch tip is exactly this commit and the landing is a fast-forward.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandResult {
    pub task: Task,
    pub commit: String,
    pub notes: Vec<String>,
}

/// `POST /v1/tasks/{id}/summary`: replaces any earlier summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSummaryRequest {
    pub text: String,
}

/// `POST /v1/tasks/{id}/ask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskQuestionRequest {
    pub body: String,
    /// Who gets the pointer message: `human`, `role:NAME`, `external:NAME` or
    /// an agent. Default: the caller's spawner, or `human`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

/// `POST /v1/tasks/{id}/answer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerQuestionRequest {
    pub body: String,
}

/// A time of day in the human's zone (this machine's local zone, which
/// the daemon already treats as Eastern), written bare: "10:42 AM".
pub fn settle_clock_text(at: DateTime<Utc>) -> String {
    at.with_timezone(&chrono::Local)
        .format("%-I:%M %p")
        .to_string()
}

/// `POST /v1/tasks/{id}/skip-settle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipSettleRequest {
    pub reason: String,
}

/// `POST /v1/tasks/{id}/note`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTaskRequest {
    pub body: String,
}

/// One task's unanswered question (coordination.md, "Questions do not stop
/// work"), for `GET /v1/questions` and `bridle inbox`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestion {
    pub task_id: String,
    pub asked_by: PrincipalId,
    pub body: String,
    pub asked_at: DateTime<Utc>,
}

/// `GET /v1/tasks?ready=true` filters to ready tasks only (roles-and-lifecycle.md,
/// "ready is computed"); omitted or `false` returns every task, as before.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready: Option<bool>,
    /// `?claimed_by=me` resolves to the calling principal's own id;
    /// anything else is matched against `Task::claimed_by` verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    /// `?top_tier=true`: just the highest queue tier with a startable task
    /// (roles-and-lifecycle.md, "the queue"), rather than every ready task
    /// project-wide. Takes precedence over `ready` when both are set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_tier: Option<bool>,
    /// `?component=<id>`: tasks naming that component or any descendant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// `?kind=<kind>`: only tasks of that kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<TaskKind>,
}

// ---------- queue ----------

/// The queue's own record: an ordered list of tiers, each a set of
/// equally-ranked task ids — tier 1 (`tiers[0]`) before tier 2
/// (roles-and-lifecycle.md, "the queue"). A task not listed in any tier is
/// backlog. PM-written (and human, to override); the manager only reads it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Queue {
    pub tiers: Vec<Vec<String>>,
}

/// `POST /v1/queue`: replaces the whole queue. The one write primitive —
/// reorder, add and remove are all "resend the tiers in the shape they
/// should be" (docs/design/storage.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetQueueRequest {
    pub tiers: Vec<Vec<String>>,
}

/// `POST /v1/queue/tiers`: appends one new tier, ranked after every existing
/// one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddQueueTierRequest {
    pub tasks: Vec<String>,
}

// ---------- edges ----------

/// Coordination edges between tasks (coordination.md, Edges). Only `blocks`
/// affects readiness; the rest are provenance, recorded but not yet acted on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    /// `to` cannot start until `from` is done.
    Blocks,
    /// `from` decomposes into `to`.
    Parent,
    DiscoveredFrom,
    Related,
    Supersedes,
    Duplicates,
}

impl EdgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blocks => "blocks",
            Self::Parent => "parent",
            Self::DiscoveredFrom => "discovered-from",
            Self::Related => "related",
            Self::Supersedes => "supersedes",
            Self::Duplicates => "duplicates",
        }
    }
}

impl std::fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EdgeKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string()))
            .map_err(|_| format!("unknown edge kind: {s}"))
    }
}

/// A directed edge `from` -> `to`, e.g. `from` blocks `to`. Durable the same
/// way a task is: a SQLite fast index plus a copy on the state branch
/// (docs/design/storage.md).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEdgeRequest {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

/// `DELETE /v1/edges?from=&to=&kind=`: the same triple identifies the edge
/// to remove, since there's no separate edge id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEdgeQuery {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
}

// ---------- errors ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Machine-readable: `not_found`, `conflict`, `bad_request`, `unauthorized`,
    /// `forbidden`, `agent_not_running`, `internal`.
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ErrorBody,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workdir_wire_shape() {
        let w = Workdir::Worktree {
            base: Some("main".into()),
        };
        assert_eq!(
            serde_json::to_value(&w).unwrap(),
            serde_json::json!({"kind":"worktree","base":"main"})
        );
        let r: Workdir = serde_json::from_str(r#"{"kind":"repo"}"#).unwrap();
        assert_eq!(r, Workdir::Repo);
    }

    #[test]
    fn agent_state_round_trip() {
        for s in [
            "starting", "idle", "working", "stopping", "stopped", "exited", "crashed", "lost",
        ] {
            let st: AgentState = s.parse().unwrap();
            assert_eq!(st.as_str(), s);
        }
    }

    #[test]
    fn task_kind_round_trip() {
        for s in [
            "feature",
            "bug",
            "chore",
            "question",
            "research",
            "explore",
            "arch-revision",
            "re-evaluate",
            "incident",
        ] {
            let k: TaskKind = s.parse().unwrap();
            assert_eq!(k.as_str(), s);
        }
    }

    #[test]
    fn task_state_round_trip() {
        for s in ["pending", "open", "planned", "dropped", "reopened"] {
            let st: TaskState = s.parse().unwrap();
            assert_eq!(st.as_str(), s);
        }
    }

    #[test]
    fn edge_kind_round_trip() {
        for s in [
            "blocks",
            "parent",
            "discovered-from",
            "related",
            "supersedes",
            "duplicates",
        ] {
            let k: EdgeKind = s.parse().unwrap();
            assert_eq!(k.as_str(), s);
        }
    }

    #[test]
    fn task_size_preserves_wire_format() {
        assert_eq!(
            serde_json::to_value(TaskSize::S).unwrap(),
            serde_json::json!("S")
        );
        assert_eq!(
            serde_json::to_value(TaskSize::M).unwrap(),
            serde_json::json!("M")
        );
        assert_eq!(
            serde_json::to_value(TaskSize::L).unwrap(),
            serde_json::json!("L")
        );
        let s: TaskSize = serde_json::from_value(serde_json::json!("S")).unwrap();
        assert_eq!(s, TaskSize::S);
    }

    #[test]
    fn priority_sort_levels_then_newest_first_above_normal() {
        let mk = |id: &str, p: &str, day: u32, set: Option<u32>| -> Task {
            let at = |d: u32| format!("2026-01-{d:02}T00:00:00Z");
            let mut v = serde_json::json!({
                "id": id, "title": id, "kind": "feature", "state": "claimed", "body": "",
                "thread": [], "created_at": at(day), "updated_at": at(day), "priority": p,
            });
            if let Some(d) = set {
                v["priority_at"] = serde_json::json!(at(d));
            }
            serde_json::from_value(v).unwrap()
        };
        let mut tasks = vec![
            mk("n-old", "normal", 1, None),
            mk("n-new", "normal", 2, None),
            mk("low", "low", 1, None),
            mk("h-old", "high", 1, Some(5)),
            mk("h-new", "high", 1, Some(9)),
            mk("u-old", "urgent", 1, Some(3)),
            mk("u-new", "urgent", 1, Some(4)),
            mk("crit", "critical", 1, Some(2)),
        ];
        sort_by_priority(&mut tasks);
        let ids: Vec<_> = tasks.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "crit", "u-new", "u-old", "h-new", "h-old", "n-old", "n-new", "low"
            ]
        );
    }
}
