//! Wire types for the bridle daemon API (`/v1`). Shared by the daemon, the
//! CLI and any future TUI/GUI/MCP client. See docs/design/agent-host/api.md.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const API_VERSION: &str = "v1";

/// `human`, `agent:<name>`, `external:<name>` or `system`.
pub type PrincipalId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Agent,
    External,
    System,
}

// ---------- health / status / discovery ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub ok: bool,
    pub version: String,
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
    pub exit: Option<ExitInfo>,
    pub created_by: PrincipalId,
    /// Messages held for `when=idle` delivery.
    pub held_messages: u32,
    /// Written to stdin but not yet acknowledged by replay echo.
    pub unacked_messages: u32,
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
    /// Skip the budget governor's holding/paused check for this one call
    /// (usage-and-budget.md, Resuming: the escape hatch).
    #[serde(default)]
    pub ignore_budget: bool,
}

/// `POST /v1/agents/{id}/resume`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResumeRequest {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    #[default]
    Note,
    Question,
    Answer,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// `POST /v1/agents/{id}/messages` uses this with `to` ignored;
/// `POST /v1/messages` requires `to` (`human`, agent id or name).
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
pub mod event_kind {
    pub const DAEMON_STARTED: &str = "daemon.started";
    pub const DAEMON_STOPPING: &str = "daemon.stopping";
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
    /// data: {message, to}
    pub const MESSAGE_SENT: &str = "message.sent";
    pub const MESSAGE_DELIVERED: &str = "message.delivered";
    pub const MESSAGE_READ: &str = "message.read";
    pub const MESSAGE_DROPPED: &str = "message.dropped";
    /// data: {info}
    pub const RATE_LIMIT: &str = "rate_limit";
    /// data: {from, to, window, utilization, resets_at, reason}. Emitted on
    /// every governor state transition (usage-and-budget.md, the budget
    /// governor).
    pub const BUDGET_STATE: &str = "budget.state";
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
    pub thresholds: BudgetThresholds,
    /// Set while `bridle budget hold` is in force (usage-and-budget.md, The
    /// human's hold).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_hold: Option<HoldStatus>,
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

// ---------- tokens ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCreateRequest {
    /// Becomes principal `external:<name>`.
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCreated {
    pub principal: PrincipalId,
    /// Shown once; the daemon stores only a hash.
    pub token: String,
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
}
