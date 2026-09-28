//! Daemon config: `<repo>/.bridle/config.toml` over built-in defaults.
//! See docs/design/agent-host/roles-and-config.md.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Datelike, Local, NaiveTime, Weekday};
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parsing {path}: {source}")]
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("invalid duration {0:?}: expected a number followed by s, m or h")]
    BadDuration(String),
    #[error("invalid listen address {0:?}: {1}")]
    BadListen(String, std::net::AddrParseError),
    #[error("[budget] {field} needs a \"default\" entry")]
    MissingDefault { field: &'static str },
    #[error(
        "{path}: [budget] {field}.{window} = {value} raises the machine-wide threshold {machine_value}; a project config may only lower it"
    )]
    ThresholdTooHigh {
        path: PathBuf,
        field: &'static str,
        window: String,
        value: f64,
        machine_value: f64,
    },
    #[error("invalid [[budget.schedule]] {name:?}: {reason}")]
    BadSchedule { name: String, reason: String },
    #[error("[components.{id}] parent {parent:?} is not a defined component")]
    UnknownComponentParent { id: String, parent: String },
    #[error("[components] parent cycle: {}", .0.join(" -> "))]
    ComponentCycle(Vec<String>),
}

/// `[components.<id>]`: a named scope inside the project (docs/design/components.md).
/// Everything is optional; `paths` and `consumers` are metadata for tooling, `docs` a
/// folder pointer, `parent` the nesting.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub consumers: Vec<String>,
}

/// Where an agent's process runs, before a worktree path is resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Workdir {
    Worktree,
    Repo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    pub model: String,
    pub effort: Option<String>,
    pub workdir: Workdir,
    /// Ref new worktrees branch from. Unused when `workdir == Repo`.
    pub base: String,
    pub permission_mode: String,
    pub allowed_tools: Vec<String>,
    /// Built-in roles start from [`DENY_MESSAGING_AND_SUBAGENTS`] and friends;
    /// a project's `.bridle/config.toml` can only add to this list
    /// ([`Role::merge`]), never remove a built-in denial.
    pub disallowed_tools: Vec<String>,
    /// Relative to the repo, per roles-and-config.md.
    pub system_prompt: Option<PathBuf>,
    pub autostart: bool,
    pub resume_on_restart: bool,
    /// The first message when an agent is spawned without a prompt,
    /// including by `autostart`, so it starts working.
    pub start_prompt: Option<String>,
    /// Passed as `--max-budget-usd`. Claude applies it per process, and
    /// checks it after each model call, so a turn can overshoot it.
    pub max_budget_usd: Option<f64>,
    /// Registers the `bridle stop-check` Stop hook (docs/design/
    /// coordination.md): worker only for now, not project-configurable.
    pub stop_check: bool,
}

/// Claude Code built-ins that let an agent bypass bridle's own coordination
/// the way `SendMessage` did (docs/questions/open/agents-can-use-claude-codes-own-sendmessage-78sp.md):
/// reporting straight to another session instead of `bridle send`, or
/// spawning subagents bridle never sees. `SendMessage` and `Workflow` are denied for
/// every built-in role; `Agent` is allowed since subagents run inside the same
/// agent's process and cost counts toward that agent's budget, so it doesn't
/// bypass bridle's coordination (ticket htp6).
const DENY_MESSAGING_AND_SUBAGENTS: [&str; 2] = ["SendMessage", "Workflow"];

/// Bypasses that also fire outside bridle's supervision: waiting via
/// Claude Code's own scheduler instead of bridle's task/claim lifecycle, or
/// kicking off a run on claude.ai directly. Denied for worker and manager;
/// the orchestrator is exempted from the scheduling half (below) since it
/// legitimately paces its own loop with `ScheduleWakeup`.
const DENY_SCHEDULING: [&str; 4] = ["ScheduleWakeup", "CronCreate", "CronDelete", "CronList"];
const DENY_REMOTE_TRIGGERS: [&str; 1] = ["RemoteTrigger"];

fn deny_list(extra: &[&[&str]]) -> Vec<String> {
    DENY_MESSAGING_AND_SUBAGENTS
        .iter()
        .chain(extra.iter().flat_map(|s| s.iter()))
        .map(|s| s.to_string())
        .collect()
}

impl Role {
    /// A fallback for an agent whose role no longer exists in config (e.g.
    /// on resume after the role was removed).
    pub(crate) fn worker_default() -> Self {
        Role {
            model: "sonnet".into(),
            effort: Some("medium".into()),
            workdir: Workdir::Worktree,
            base: "HEAD".into(),
            permission_mode: "acceptEdits".into(),
            allowed_tools: vec![
                "Bash".into(),
                "Read".into(),
                "Edit".into(),
                "Write".into(),
                "Glob".into(),
                "Grep".into(),
            ],
            disallowed_tools: deny_list(&[&DENY_SCHEDULING, &DENY_REMOTE_TRIGGERS]),
            system_prompt: None,
            autostart: false,
            resume_on_restart: false,
            start_prompt: None,
            max_budget_usd: None,
            stop_check: true,
        }
    }

    fn manager_default() -> Self {
        Role {
            model: "sonnet".into(),
            effort: None,
            workdir: Workdir::Repo,
            base: "HEAD".into(),
            permission_mode: "dontAsk".into(),
            allowed_tools: vec![
                "Bash(bridle *)".into(),
                "Bash(git *)".into(),
                "Read".into(),
                "Glob".into(),
                "Grep".into(),
            ],
            disallowed_tools: deny_list(&[&DENY_SCHEDULING, &DENY_REMOTE_TRIGGERS]),
            system_prompt: None,
            autostart: false,
            resume_on_restart: true,
            start_prompt: None,
            max_budget_usd: None,
            stop_check: false,
        }
    }

    fn orchestrator_default() -> Self {
        Role {
            model: "sonnet".into(),
            effort: None,
            workdir: Workdir::Repo,
            base: "HEAD".into(),
            permission_mode: "dontAsk".into(),
            allowed_tools: vec![
                "Bash(bridle *)".into(),
                "Read".into(),
                "Glob".into(),
                "Grep".into(),
            ],
            // Scheduling is exempted: the orchestrator paces its own loop
            // with `ScheduleWakeup` (docs/questions/open/agents-can-use-claude-codes-own-sendmessage-78sp.md).
            disallowed_tools: deny_list(&[&DENY_REMOTE_TRIGGERS]),
            system_prompt: None,
            autostart: false,
            resume_on_restart: true,
            start_prompt: None,
            max_budget_usd: None,
            stop_check: false,
        }
    }

    /// `Bash(bridle *)` is always available, so an agent can always reach
    /// bridle even if a config role forgot to list it.
    pub fn effective_allowed_tools(&self) -> Vec<String> {
        let mut tools = self.allowed_tools.clone();
        if !tools.iter().any(|t| t == "Bash(bridle *)") {
            tools.push("Bash(bridle *)".into());
        }
        tools
    }

    fn merge(mut self, raw: RawRole) -> Self {
        if let Some(v) = raw.model {
            self.model = v;
        }
        if let Some(v) = raw.effort {
            self.effort = Some(v);
        }
        if let Some(v) = raw.workdir {
            self.workdir = v;
        }
        if let Some(v) = raw.base {
            self.base = v;
        }
        if let Some(v) = raw.permission_mode {
            self.permission_mode = v;
        }
        if let Some(v) = raw.allowed_tools {
            self.allowed_tools = v;
        }
        // Additive, unlike `allowed_tools`: a project config can only add
        // denials to the built-in defaults, never drop one by omission.
        if let Some(v) = raw.disallowed_tools {
            for tool in v {
                if !self.disallowed_tools.contains(&tool) {
                    self.disallowed_tools.push(tool);
                }
            }
        }
        if let Some(v) = raw.system_prompt {
            self.system_prompt = Some(v);
        }
        if let Some(v) = raw.autostart {
            self.autostart = v;
        }
        if let Some(v) = raw.resume_on_restart {
            self.resume_on_restart = v;
        }
        if let Some(v) = raw.start_prompt {
            self.start_prompt = Some(v);
        }
        if let Some(v) = raw.max_budget_usd {
            self.max_budget_usd = Some(v);
        }
        if let Some(v) = raw.stop_check {
            self.stop_check = v;
        }
        self
    }
}

/// A percentage threshold (0-100) per window, with a required fallback for
/// any window not named explicitly (usage-and-budget.md, Thresholds: `{
/// default = 80 }`).
#[derive(Debug, Clone, PartialEq)]
pub struct WindowThresholds {
    default: f64,
    overrides: BTreeMap<String, f64>,
}

impl WindowThresholds {
    fn constant(default: f64) -> Self {
        WindowThresholds {
            default,
            overrides: BTreeMap::new(),
        }
    }

    /// The effective threshold for `window` (its own override, else the
    /// section's default).
    pub fn get(&self, window: &str) -> f64 {
        self.overrides.get(window).copied().unwrap_or(self.default)
    }

    fn from_raw(map: &BTreeMap<String, f64>, field: &'static str) -> Result<Self, ConfigError> {
        let default = *map
            .get("default")
            .ok_or(ConfigError::MissingDefault { field })?;
        let overrides = map
            .iter()
            .filter(|(k, _)| k.as_str() != "default")
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        Ok(WindowThresholds { default, overrides })
    }

    /// All entries this threshold is aware of, `default` first, for the
    /// wire representation (`GET /v1/budget`).
    fn to_map(&self) -> BTreeMap<String, f64> {
        let mut m = self.overrides.clone();
        m.insert("default".to_string(), self.default);
        m
    }

    /// A project override may only lower a threshold, never raise it
    /// (usage-and-budget.md, Thresholds).
    fn merge_lower_only(
        &self,
        raw: &BTreeMap<String, f64>,
        field: &'static str,
        path: &Path,
    ) -> Result<Self, ConfigError> {
        let mut out = self.clone();
        for (window, &value) in raw {
            let machine_value = self.get(window);
            if value > machine_value {
                return Err(ConfigError::ThresholdTooHigh {
                    path: path.to_path_buf(),
                    field,
                    window: window.clone(),
                    value,
                    machine_value,
                });
            }
            if window == "default" {
                out.default = value;
            } else {
                out.overrides.insert(window.clone(), value);
            }
        }
        Ok(out)
    }
}

/// One `[[budget.schedule]]` period: a set of host-local days and a
/// time-of-day window, with its own `five_hour` thresholds
/// (usage-and-budget.md, Schedule; ticket n9qh). Time is read from
/// `chrono::Local`, i.e. the host machine's own timezone — there's no
/// per-project timezone config, since this machine is effectively US
/// Eastern already; a daemon running elsewhere is a future ticket.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulePeriod {
    pub name: String,
    pub days: Vec<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub hold_at: f64,
    pub wind_down_at: f64,
    pub stop_at: f64,
}

impl SchedulePeriod {
    /// Whether host-local `now` falls in this period: today's weekday is
    /// listed, and the time-of-day is within `start..end`, a range that may
    /// cross midnight (e.g. `23:00..07:00`).
    pub fn matches(&self, now: DateTime<Local>) -> bool {
        if !self.days.contains(&now.weekday()) {
            return false;
        }
        let t = now.time();
        if self.start <= self.end {
            t >= self.start && t < self.end
        } else {
            t >= self.start || t < self.end
        }
    }
}

/// `[budget]`: the account-wide usage governor's thresholds
/// (usage-and-budget.md, The budget governor). Lives in
/// `~/.bridle/config.toml`; a project's `.bridle/config.toml` may lower
/// (never raise) the four percentage thresholds.
#[derive(Debug, Clone)]
pub struct BudgetConfig {
    pub max_workers: u32,
    pub hold_at: WindowThresholds,
    pub wind_down_at: WindowThresholds,
    pub stop_at: WindowThresholds,
    pub resume_below: WindowThresholds,
    pub wind_down_grace: Duration,
    pub max_staleness: Duration,
    /// Named time-of-day periods that replace the `five_hour` thresholds
    /// above while they're in effect (usage-and-budget.md, Schedule).
    /// `seven_day` and every other window are never affected. Walked in
    /// order; the first match wins. Empty by default.
    pub schedule: Vec<SchedulePeriod>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        let mut wind_down_at = WindowThresholds::constant(90.0);
        wind_down_at
            .overrides
            .insert("seven_day_opus".to_string(), 85.0);
        BudgetConfig {
            max_workers: 2,
            hold_at: WindowThresholds::constant(80.0),
            wind_down_at,
            stop_at: WindowThresholds::constant(95.0),
            resume_below: WindowThresholds::constant(70.0),
            wind_down_grace: Duration::from_secs(5 * 60),
            max_staleness: Duration::from_secs(10 * 60),
            schedule: Vec::new(),
        }
    }
}

impl BudgetConfig {
    /// Machine-level merge: an unrestricted overlay, like [`Role::merge`].
    fn merge(mut self, raw: RawBudget) -> Result<Self, ConfigError> {
        if let Some(v) = raw.max_workers {
            self.max_workers = v;
        }
        if let Some(m) = &raw.hold_at {
            self.hold_at = WindowThresholds::from_raw(m, "hold_at")?;
        }
        if let Some(m) = &raw.wind_down_at {
            self.wind_down_at = WindowThresholds::from_raw(m, "wind_down_at")?;
        }
        if let Some(m) = &raw.stop_at {
            self.stop_at = WindowThresholds::from_raw(m, "stop_at")?;
        }
        if let Some(m) = &raw.resume_below {
            self.resume_below = WindowThresholds::from_raw(m, "resume_below")?;
        }
        if let Some(s) = &raw.wind_down_grace {
            self.wind_down_grace = parse_duration(s)?;
        }
        if let Some(s) = &raw.max_staleness {
            self.max_staleness = parse_duration(s)?;
        }
        if let Some(list) = raw.schedule {
            self.schedule = list
                .into_iter()
                .map(RawSchedulePeriod::into_period)
                .collect::<Result<_, _>>()?;
        }
        Ok(self)
    }

    /// Project-level merge: `max_workers` and the durations may be set
    /// freely, but the four percentage thresholds may only come down from
    /// the machine-wide value (usage-and-budget.md, Thresholds).
    fn merge_project(mut self, raw: RawBudget, path: &Path) -> Result<Self, ConfigError> {
        if let Some(v) = raw.max_workers {
            self.max_workers = v;
        }
        if let Some(m) = &raw.hold_at {
            self.hold_at = self.hold_at.merge_lower_only(m, "hold_at", path)?;
        }
        if let Some(m) = &raw.wind_down_at {
            self.wind_down_at = self
                .wind_down_at
                .merge_lower_only(m, "wind_down_at", path)?;
        }
        if let Some(m) = &raw.stop_at {
            self.stop_at = self.stop_at.merge_lower_only(m, "stop_at", path)?;
        }
        if let Some(m) = &raw.resume_below {
            self.resume_below = self
                .resume_below
                .merge_lower_only(m, "resume_below", path)?;
        }
        if let Some(s) = &raw.wind_down_grace {
            self.wind_down_grace = parse_duration(s)?;
        }
        if let Some(s) = &raw.max_staleness {
            self.max_staleness = parse_duration(s)?;
        }
        if let Some(list) = raw.schedule {
            let periods = list
                .into_iter()
                .map(RawSchedulePeriod::into_period)
                .collect::<Result<Vec<_>, _>>()?;
            for period in &periods {
                check_schedule_ceiling(
                    period.hold_at,
                    self.hold_at.get("five_hour"),
                    &period.name,
                    "hold_at",
                    path,
                )?;
                check_schedule_ceiling(
                    period.wind_down_at,
                    self.wind_down_at.get("five_hour"),
                    &period.name,
                    "wind_down_at",
                    path,
                )?;
                check_schedule_ceiling(
                    period.stop_at,
                    self.stop_at.get("five_hour"),
                    &period.name,
                    "stop_at",
                    path,
                )?;
            }
            self.schedule = periods;
        }
        Ok(self)
    }

    pub fn to_wire(&self) -> bridle_api::types::BudgetThresholds {
        bridle_api::types::BudgetThresholds {
            max_workers: self.max_workers,
            hold_at: self.hold_at.to_map(),
            wind_down_at: self.wind_down_at.to_map(),
            stop_at: self.stop_at.to_map(),
            resume_below: self.resume_below.to_map(),
            wind_down_grace_secs: self.wind_down_grace.as_secs(),
            max_staleness_secs: self.max_staleness.as_secs(),
        }
    }
}

/// `[models]`: default model preference lists by role, strongest first
/// (usage-and-budget.md, Model choice). A project's `.bridle/config.toml`
/// may replace a role's list outright — unlike `[budget]`'s thresholds,
/// this is an ordered preference, not a ceiling, so there's no lower-only
/// restriction.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelsConfig {
    pub by_role: BTreeMap<String, Vec<String>>,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        let mut by_role = BTreeMap::new();
        by_role.insert("manager".to_string(), vec!["opus".into(), "sonnet".into()]);
        by_role.insert("planner".to_string(), vec!["opus".into(), "sonnet".into()]);
        by_role.insert("reviewer".to_string(), vec!["sonnet".into(), "opus".into()]);
        by_role.insert("worker".to_string(), vec!["sonnet".into(), "haiku".into()]);
        by_role.insert("chore".to_string(), vec!["haiku".into()]);
        by_role.insert("explore".to_string(), vec!["sonnet".into()]);
        ModelsConfig { by_role }
    }
}

impl ModelsConfig {
    /// The role's preference list, strongest first, for stepping down
    /// (usage-and-budget.md, Model choice). Falls back to `role.model` as a
    /// single-entry list when the role has no `[models]` entry, so a
    /// project's custom role that only sets `model` still works unchanged.
    pub fn candidates<'a>(&'a self, role_name: &str, role: &'a Role) -> Vec<&'a str> {
        match self.by_role.get(role_name) {
            Some(list) if !list.is_empty() => list.iter().map(String::as_str).collect(),
            _ => vec![role.model.as_str()],
        }
    }

    fn merge(mut self, raw: BTreeMap<String, Vec<String>>) -> Self {
        for (role, list) in raw {
            self.by_role.insert(role, list);
        }
        self
    }
}

/// `[context]`: the context governor's `wind_down_at` token threshold, per
/// role (htp6). Every role bridle spawns — including manager and
/// orchestrator — is governed; workers default well under the 200k window
/// so a long tool-heavy run still has room for a handoff message before the
/// window fills (docs/design/agent-host/, htp6 task notes).
#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub wind_down_at: WindowThresholds,
    /// How long a notified agent gets to finish a handoff turn (commit WIP,
    /// write a note) on its own before it's renewed regardless — the same
    /// shape as `[budget] wind_down_grace`, since `stop_grace`'s 30s is
    /// meant for an idle process noticing stdin EOF, not a whole turn.
    pub wind_down_grace: Duration,
}

impl Default for ContextConfig {
    fn default() -> Self {
        let mut wind_down_at = WindowThresholds::constant(200_000.0);
        wind_down_at
            .overrides
            .insert("worker".to_string(), 120_000.0);
        ContextConfig {
            wind_down_at,
            wind_down_grace: Duration::from_secs(5 * 60),
        }
    }
}

impl ContextConfig {
    fn merge(mut self, raw: RawContext) -> Result<Self, ConfigError> {
        if let Some(m) = &raw.wind_down_at {
            self.wind_down_at = WindowThresholds::from_raw(m, "wind_down_at")?;
        }
        if let Some(s) = &raw.wind_down_grace {
            self.wind_down_grace = parse_duration(s)?;
        }
        Ok(self)
    }
}

/// `[commands]`: shell commands a project binds for the workflow to invoke,
/// so `workflow/base/skills/worker/SKILL.md` (and friends) can reference
/// `{{commands.check}}` instead of hardcoding a build tool that differs
/// per project (`just check` here, `make check` for data-contracts).
#[derive(Debug, Clone, PartialEq)]
pub struct CommandsConfig {
    pub check: String,
}

/// `[branches]`: the project's branch pattern (docs/design/agent-host/operating-model.md,
/// "Branch pattern"). Exactly two shapes, per the human's own KISS ask (ticket br-29f9):
/// trunk (`release` unset — work merges into `integration`, releases are tags on it, bridle's
/// own pattern) or dev+release (`release` set — work merges into `integration`, and `release`
/// only moves when `integration` is merged into it for a release, done by whoever cuts
/// releases, not by workers or the manager's normal merge). A project trial
/// (docs/questions/open/trial-adoption-*.md, 63rv) sets `integration` to its trial branch
/// (e.g. `bridle-adopt`) with `release` left unset, so nothing here ever targets the
/// project's real `main`/`dev`.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchesConfig {
    pub integration: String,
    /// True when `integration` came from the config file rather than the `main` default;
    /// startup only insists the default exists (g3ck).
    pub integration_set: bool,
    pub release: Option<String>,
}

impl Default for BranchesConfig {
    fn default() -> Self {
        BranchesConfig {
            integration: "main".to_string(),
            integration_set: false,
            release: None,
        }
    }
}

impl BranchesConfig {
    fn merge(mut self, raw: RawBranches) -> Self {
        if let Some(v) = raw.integration {
            self.integration = v;
            self.integration_set = true;
        }
        if let Some(v) = raw.release {
            self.release = Some(v);
        }
        self
    }
}

/// `[ci]`: opt-in watching of the integration branch's GitHub Actions runs (`crate::ci`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CiConfig {
    pub github: bool,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        CommandsConfig {
            check: "just check".to_string(),
        }
    }
}

impl CommandsConfig {
    fn merge(mut self, raw: RawCommands) -> Self {
        if let Some(v) = raw.check {
            self.check = v;
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub stall_after: Duration,
    /// How long a claimed task's lease survives without the claiming
    /// agent's own activity (`last_event_at`/`turn_started_at`) before it's
    /// released back to `planned` (docs/design/storage.md, claims).
    pub claim_lease_after: Duration,
    pub stop_grace: Duration,
    pub roles: BTreeMap<String, Role>,
    pub budget: BudgetConfig,
    pub models: ModelsConfig,
    pub context: ContextConfig,
    pub commands: CommandsConfig,
    pub branches: BranchesConfig,
    pub ci: CiConfig,
    /// The prefix new task ids get (storage.md: `<prefix>-<4 hex chars>`,
    /// e.g. `tw-7fa2`). `None` means derive one from the project name
    /// ([`default_task_prefix`]).
    pub task_prefix: Option<String>,
    /// Where the L1/L2 workflow layers live (docs/design/workflow-layers.md):
    /// a path (relative to the repo root) or a git url. `None` until a
    /// project opts in. Read by `bridle_daemon::rules::discover_layers` as
    /// `<workflow>/base/rules` (L1) and `<workflow>/packs/<name>/rules` (L2,
    /// one per entry in `packs`).
    pub workflow: Option<String>,
    /// L2 pack names to layer in, in listed order. Pack content itself is
    /// out of scope for now (docs/design/workflow-layers.md); this is just
    /// which pack directories to include.
    pub packs: Vec<String>,
    /// `[components.<id>]`, validated: parents exist and there are no cycles.
    pub components: BTreeMap<String, Component>,
}

impl Default for Config {
    fn default() -> Self {
        let mut roles = BTreeMap::new();
        roles.insert("worker".to_string(), Role::worker_default());
        roles.insert("manager".to_string(), Role::manager_default());
        roles.insert("orchestrator".to_string(), Role::orchestrator_default());
        Config {
            listen: "127.0.0.1:0".parse().expect("valid default listen addr"),
            stall_after: Duration::from_secs(10 * 60),
            claim_lease_after: Duration::from_secs(10 * 60),
            stop_grace: Duration::from_secs(30),
            roles,
            budget: BudgetConfig::default(),
            models: ModelsConfig::default(),
            context: ContextConfig::default(),
            commands: CommandsConfig::default(),
            branches: BranchesConfig::default(),
            ci: CiConfig::default(),
            task_prefix: None,
            workflow: None,
            packs: Vec::new(),
            components: BTreeMap::new(),
        }
    }
}

/// The default task id prefix when `[tasks] prefix` isn't set: the
/// project name's first two ASCII alphanumeric characters, lowercased (e.g.
/// "bridle" -> "br"). Storage.md's own examples (`tw-7fa2`, `hx-19ab`)
/// aren't derived from any project name shown in these docs, so this is a
/// judgment call, not a rule pinned down anywhere else; picked over hashing
/// or a random prefix because it's the same prefix every time a given
/// project is served, without needing to persist it anywhere. Falls back to
/// `"tk"` if the project name has no alphanumeric characters at all, and
/// pads with `'x'` if it has exactly one.
pub fn default_task_prefix(project: &str) -> String {
    let alnum: String = project
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .take(2)
        .collect();
    match alnum.len() {
        0 => "tk".to_string(),
        1 => format!("{alnum}x"),
        _ => alnum,
    }
}

impl Config {
    /// Loads the machine-wide `[budget]` section
    /// (`$BRIDLE_HOME/config.toml`, else `~/.bridle/config.toml`; see
    /// `bridle_api::discovery::bridle_home`) merged over the built-in
    /// defaults. Missing file is not an error.
    ///
    /// `home_override` lets callers (tests, via [`crate::Overrides`]) bypass
    /// the real machine home instead of mutating `$BRIDLE_HOME`, which would
    /// need `std::env::set_var` -- unsafe as of edition 2024, and `unsafe`
    /// is forbidden in this workspace.
    fn load_machine_budget(home_override: Option<&Path>) -> Result<BudgetConfig, ConfigError> {
        let home = home_override
            .map(Path::to_path_buf)
            .unwrap_or_else(bridle_api::discovery::bridle_home);
        let path = home.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let raw: RawConfig =
                    toml::from_str(&text).map_err(|source| ConfigError::Parse {
                        path: path.clone(),
                        source: Box::new(source),
                    })?;
                BudgetConfig::default().merge(raw.budget.unwrap_or_default())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BudgetConfig::default()),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// Loads `<repo>/.bridle/config.toml` over the built-in defaults,
    /// merged with the machine-wide `[budget]` section. Missing file is not
    /// an error: the file is optional (docs/design/agent-host/operating-model.md).
    pub fn load(repo: &Path) -> Result<Self, ConfigError> {
        Self::load_with_home(repo, None)
    }

    /// [`Config::load`], with the machine-wide home directory overridable
    /// (see [`Self::load_machine_budget`]). Tests use this, via
    /// [`crate::Overrides::bridle_home`], to stay isolated from the real
    /// machine's `~/.bridle/config.toml`.
    pub fn load_with_home(repo: &Path, home_override: Option<&Path>) -> Result<Self, ConfigError> {
        let machine_budget = Self::load_machine_budget(home_override)?;
        let path = repo.join(".bridle").join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse_with_budget(&text, machine_budget, &path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut config = Config {
                    budget: machine_budget,
                    ..Config::default()
                };
                apply_branches(&mut config);
                Ok(config)
            }
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// A component's chain, root-most ancestor first, itself last. `None` if `id` isn't
    /// defined. Config is validated on load, so the walk terminates; the length bound is
    /// only a guard for a hand-built `Config`.
    pub fn component_chain(&self, id: &str) -> Option<Vec<&str>> {
        let mut chain = Vec::new();
        let mut cur = self.components.get_key_value(id)?;
        loop {
            chain.push(cur.0.as_str());
            match cur
                .1
                .parent
                .as_deref()
                .and_then(|p| self.components.get_key_value(p))
            {
                Some(next) if chain.len() <= self.components.len() => cur = next,
                _ => break,
            }
        }
        chain.reverse();
        Some(chain)
    }

    /// Checks each id names a defined component and drops repeats, keeping
    /// order. Ancestors are implied by naming a child, so they aren't added.
    pub fn normalize_components(&self, ids: &[String]) -> Result<Vec<String>, String> {
        let mut out: Vec<String> = Vec::new();
        for id in ids {
            if !self.components.contains_key(id) {
                return Err(format!("no component {id:?} in .bridle/config.toml"));
            }
            if !out.contains(id) {
                out.push(id.clone());
            }
        }
        Ok(out)
    }

    /// Whether work naming `named` is in `filter`'s scope: `filter` is one of
    /// them or an ancestor of one (i.e. a named component is `filter` or a
    /// descendant of it).
    pub fn components_match(&self, named: &[String], filter: &str) -> bool {
        named.iter().any(|n| {
            self.component_chain(n)
                .is_some_and(|chain| chain.contains(&filter))
        })
    }

    fn validate_components(&self) -> Result<(), ConfigError> {
        for (id, c) in &self.components {
            if let Some(p) = &c.parent
                && !self.components.contains_key(p)
            {
                return Err(ConfigError::UnknownComponentParent {
                    id: id.clone(),
                    parent: p.clone(),
                });
            }
        }
        for start in self.components.keys() {
            let mut path = vec![start.clone()];
            let mut cur = start;
            while let Some(p) = self.components[cur].parent.as_ref() {
                path.push(p.clone());
                if p == start {
                    return Err(ConfigError::ComponentCycle(path));
                }
                if path.len() > self.components.len() + 1 {
                    break; // a cycle not through `start`; reported from its own members
                }
                cur = p;
            }
        }
        Ok(())
    }

    /// Parses config TOML text over the built-in defaults, with the
    /// built-in `[budget]` defaults as the "machine" side (no lower-only
    /// restriction applies). Split out from [`Config::load`] so tests don't
    /// need real files.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        Self::parse_with_budget(text, BudgetConfig::default(), Path::new("config.toml"))
    }

    fn parse_with_budget(
        text: &str,
        machine_budget: BudgetConfig,
        path: &Path,
    ) -> Result<Self, ConfigError> {
        let raw: RawConfig = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })?;

        let mut config = Config {
            budget: machine_budget,
            ..Config::default()
        };

        if let Some(d) = raw.daemon {
            if let Some(listen) = d.listen {
                config.listen = listen
                    .parse()
                    .map_err(|e| ConfigError::BadListen(listen.clone(), e))?;
            }
            if let Some(s) = d.stall_after {
                config.stall_after = parse_duration(&s)?;
            }
            if let Some(s) = d.claim_lease_after {
                config.claim_lease_after = parse_duration(&s)?;
            }
            if let Some(s) = d.stop_grace {
                config.stop_grace = parse_duration(&s)?;
            }
        }

        if let Some(raw_branches) = raw.branches {
            config.branches = config.branches.merge(raw_branches);
        }
        if let Some(github) = raw.ci.and_then(|c| c.github) {
            config.ci.github = github;
        }

        // A role's own `model` pins the step-down floor unless the project
        // also gives that role an explicit `[models]` list (which wins
        // outright, below). Collect those names before merging `[models]`
        // in, so the pin and the explicit override stay distinguishable.
        let explicit_models: std::collections::BTreeSet<&str> = raw
            .models
            .iter()
            .flat_map(|m| m.keys())
            .map(String::as_str)
            .collect();

        for (name, raw_role) in raw.roles.unwrap_or_default() {
            let pinned_model = raw_role.model.clone();
            let base = config
                .roles
                .remove(&name)
                .unwrap_or_else(Role::worker_default);
            config.roles.insert(name.clone(), base.merge(raw_role));

            if let Some(model) = pinned_model
                && !explicit_models.contains(name.as_str())
                && let Some(list) = config.models.by_role.get_mut(&name)
            {
                *list = match list.iter().position(|m| *m == model) {
                    Some(pos) => list[pos..].to_vec(),
                    None => vec![model],
                };
            }
        }

        if let Some(raw_budget) = raw.budget {
            config.budget = config.budget.merge_project(raw_budget, path)?;
        }

        if let Some(raw_models) = raw.models {
            config.models = config.models.merge(raw_models);
        }

        if let Some(raw_context) = raw.context {
            config.context = config.context.merge(raw_context)?;
        }

        if let Some(raw_commands) = raw.commands {
            config.commands = config.commands.merge(raw_commands);
        }

        if let Some(t) = raw.tasks {
            config.task_prefix = t.prefix;
        }

        config.workflow = raw.workflow;
        config.packs = raw.packs.unwrap_or_default();
        config.components = raw.components.unwrap_or_default();
        config.validate_components()?;

        apply_branches(&mut config);

        Ok(config)
    }
}

/// Wires `[branches]` through the roles, once every other section has been
/// merged (roles-and-config.md, "Branch pattern"):
/// - A worktree role's `base` still defaults to the built-in `"HEAD"`
///   sentinel unless a project explicitly overrides it; when untouched, it's
///   set to the integration branch, so new agent worktrees (and the worker's
///   own handoff merge, via `{{branches.integration}}`) branch from and
///   merge into the same ref the manager merges into -- not from whatever
///   happens to be checked out.
/// - When a separate release branch is configured, every role except
///   `orchestrator` (which legitimately cuts releases, docs/design/
///   agent-host/operating-model.md) is mechanically denied `git push` to it,
///   the same additive `disallowed_tools` hook a project already uses for
///   `Bash(git push *)` (roles-and-config.md) -- a locked rule
///   (workflow-layers.md override semantics), not just written prose in a
///   role prompt.
fn apply_branches(config: &mut Config) {
    let integration = config.branches.integration.clone();
    let release = config.branches.release.clone();
    for (name, role) in config.roles.iter_mut() {
        if role.workdir == Workdir::Worktree && role.base == "HEAD" {
            role.base = integration.clone();
        }
        if let Some(release) = &release
            && release != &integration
            && name != "orchestrator"
        {
            let deny = format!("Bash(git push origin {release})");
            if !role.disallowed_tools.contains(&deny) {
                role.disallowed_tools.push(deny);
            }
        }
    }
}

/// A `[[budget.schedule]]` period's threshold may only come down from the
/// machine-wide plain `five_hour` value, the same `ThresholdTooHigh` rule as
/// the four percentage thresholds themselves.
fn check_schedule_ceiling(
    value: f64,
    machine_value: f64,
    period_name: &str,
    kind: &'static str,
    path: &Path,
) -> Result<(), ConfigError> {
    if value > machine_value {
        return Err(ConfigError::ThresholdTooHigh {
            path: path.to_path_buf(),
            field: "schedule",
            window: format!("{period_name}.{kind}"),
            value,
            machine_value,
        });
    }
    Ok(())
}

fn parse_weekday(s: &str, period_name: &str) -> Result<Weekday, ConfigError> {
    use Weekday::*;
    match s.to_ascii_lowercase().as_str() {
        "mon" => Ok(Mon),
        "tue" => Ok(Tue),
        "wed" => Ok(Wed),
        "thu" => Ok(Thu),
        "fri" => Ok(Fri),
        "sat" => Ok(Sat),
        "sun" => Ok(Sun),
        other => Err(ConfigError::BadSchedule {
            name: period_name.to_string(),
            reason: format!("invalid day {other:?}: expected mon, tue, wed, thu, fri, sat or sun"),
        }),
    }
}

fn parse_time_of_day(s: &str, period_name: &str) -> Result<NaiveTime, ConfigError> {
    NaiveTime::parse_from_str(s, "%H:%M").map_err(|_| ConfigError::BadSchedule {
        name: period_name.to_string(),
        reason: format!("invalid time {s:?}: expected HH:MM"),
    })
}

/// Parses "10m"-style durations: an integer followed by `s`, `m` or `h`.
fn parse_duration(s: &str) -> Result<Duration, ConfigError> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let n: u64 = num
        .parse()
        .map_err(|_| ConfigError::BadDuration(s.to_string()))?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        _ => return Err(ConfigError::BadDuration(s.to_string())),
    };
    Ok(Duration::from_secs(secs))
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    daemon: Option<RawDaemon>,
    #[serde(default)]
    roles: Option<BTreeMap<String, RawRole>>,
    #[serde(default)]
    budget: Option<RawBudget>,
    #[serde(default)]
    models: Option<BTreeMap<String, Vec<String>>>,
    #[serde(default)]
    context: Option<RawContext>,
    #[serde(default)]
    commands: Option<RawCommands>,
    #[serde(default)]
    branches: Option<RawBranches>,
    #[serde(default)]
    ci: Option<RawCi>,
    #[serde(default)]
    tasks: Option<RawTasks>,
    #[serde(default)]
    workflow: Option<String>,
    #[serde(default)]
    packs: Option<Vec<String>>,
    #[serde(default)]
    components: Option<BTreeMap<String, Component>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContext {
    #[serde(default)]
    wind_down_at: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    wind_down_grace: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCommands {
    #[serde(default)]
    check: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBranches {
    #[serde(default)]
    integration: Option<String>,
    #[serde(default)]
    release: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCi {
    #[serde(default)]
    github: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTasks {
    #[serde(default)]
    prefix: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBudget {
    #[serde(default)]
    max_workers: Option<u32>,
    #[serde(default)]
    hold_at: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    wind_down_at: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    stop_at: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    resume_below: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    wind_down_grace: Option<String>,
    #[serde(default)]
    max_staleness: Option<String>,
    #[serde(default)]
    schedule: Option<Vec<RawSchedulePeriod>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSchedulePeriod {
    name: String,
    days: RawDays,
    start: String,
    end: String,
    hold_at: f64,
    wind_down_at: f64,
    stop_at: f64,
}

impl RawSchedulePeriod {
    fn into_period(self) -> Result<SchedulePeriod, ConfigError> {
        let days = match self.days {
            RawDays::All(s) if s.eq_ignore_ascii_case("all") => {
                vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                    Weekday::Sat,
                    Weekday::Sun,
                ]
            }
            RawDays::All(s) => {
                return Err(ConfigError::BadSchedule {
                    name: self.name,
                    reason: format!("invalid days {s:?}: expected \"all\" or a list of mon..sun"),
                });
            }
            RawDays::List(days) => days
                .iter()
                .map(|d| parse_weekday(d, &self.name))
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(SchedulePeriod {
            start: parse_time_of_day(&self.start, &self.name)?,
            end: parse_time_of_day(&self.end, &self.name)?,
            name: self.name,
            days,
            hold_at: self.hold_at,
            wind_down_at: self.wind_down_at,
            stop_at: self.stop_at,
        })
    }
}

/// `days = "all"`, or a list of `mon`..`sun` (roles-and-config.md, `[budget]`).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawDays {
    All(String),
    List(Vec<String>),
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDaemon {
    #[serde(default)]
    listen: Option<String>,
    #[serde(default)]
    stall_after: Option<String>,
    #[serde(default)]
    claim_lease_after: Option<String>,
    #[serde(default)]
    stop_grace: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRole {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    effort: Option<String>,
    #[serde(default)]
    workdir: Option<Workdir>,
    #[serde(default)]
    base: Option<String>,
    #[serde(default)]
    permission_mode: Option<String>,
    #[serde(default)]
    allowed_tools: Option<Vec<String>>,
    #[serde(default)]
    disallowed_tools: Option<Vec<String>>,
    #[serde(default)]
    system_prompt: Option<PathBuf>,
    #[serde(default)]
    autostart: Option<bool>,
    #[serde(default)]
    resume_on_restart: Option<bool>,
    #[serde(default)]
    start_prompt: Option<String>,
    #[serde(default)]
    max_budget_usd: Option<f64>,
    #[serde(default)]
    stop_check: Option<bool>,
}

impl<'de> Deserialize<'de> for Workdir {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "worktree" => Ok(Workdir::Worktree),
            "repo" => Ok(Workdir::Repo),
            other => Err(serde::de::Error::custom(format!(
                "invalid workdir {other:?}: expected \"worktree\" or \"repo\""
            ))),
        }
    }
}

/// The fixed bridle preamble, identical for every agent of a role so the
/// prompt cache holds across worktrees (docs/design/usage-and-budget.md, rule 2). No
/// agent names, paths or timestamps: [`render_system_prompt`] appends those,
/// as a short sentence after this shared, cacheable part, so the long
/// prefix still matches across agents of the same role.
const PREAMBLE: &str = "\
You are an agent run by bridle, a local daemon that spawns, supervises and \
records headless Claude Code agents for a software project. You are one of \
several agents; others may be working on the same repository at the same \
time.

Your identity is set by environment variables:
- BRIDLE_AGENT_ID: your stable agent id.
- BRIDLE_AGENT_NAME: your human-friendly name.
- BRIDLE_URL: the daemon's API base URL.
- BRIDLE_TOKEN: your own API token. Never share it or read anyone else's.
- BRIDLE_WORKSPACE: the workspace root (the clone and all worktrees live under it).

Messages from other principals (the human, another agent, or bridle itself)
arrive as user messages that start with a line like:
  [bridle message m-0042 from human (Jo)]

To reply or send a message of your own, run:
  bridle send <agent|human> \"text\" [--question] [--reply-to m-...]

Check what's waiting for you with `bridle inbox --json`. See the system's
state with `bridle status --json` and `bridle agents --json`. Always pass
--json: it gives you structured output instead of a table meant for humans.

Never read another principal's token, and never read anything under
.bridle/tokens. Your own token is already in BRIDLE_TOKEN.

Claude Code's memory is off, and you must not keep notes outside the
repository. Anything worth keeping (a decision, a fact, a gotcha) goes in the
repository's docs or rules, or in a message to whoever gave you the task.
";

fn role_preamble_suffix(role_name: &str) -> Option<&'static str> {
    match role_name {
        "worker" => Some(
            "\nYou are a worker: implement the task you were given in your own worktree, commit your work on your branch, and report back to whoever gave you the task.\n",
        ),
        "manager" => Some(
            "\nYou are the manager: coordinate work in this repository and spawn workers with `bridle spawn worker --name ... --prompt ...` to do it; you never edit code yourself.\n",
        ),
        "orchestrator" => Some(
            "\nYou are the orchestrator: you are the human's delegate, driving bridle's agents to get work done.\n",
        ),
        _ => None,
    }
}

/// The role-scoped, agent-independent part of the system prompt: the fixed
/// preamble, a role-specific sentence for the three built-in roles, then the
/// role's own prompt file if it has one. Byte-identical for every agent of
/// `role_name` in this project, so it's the shared, cacheable prefix
/// `render_system_prompt` builds on (docs/design/usage-and-budget.md, rule 2).
///
/// Public so `bridle cost audit` ([`crate::cost_audit`]) can measure the exact bytes
/// that go on the wire, from a fresh render rather than a cached figure.
pub fn stable_system_prompt(
    role_name: &str,
    role: &Role,
    repo: &Path,
    branches: &BranchesConfig,
    commands: &CommandsConfig,
) -> String {
    let mut out = String::from(PREAMBLE);
    if let Some(suffix) = role_preamble_suffix(role_name) {
        out.push_str(suffix);
    }
    out.push_str(&branches_sentence(branches));
    if let Some(rel) = &role.system_prompt {
        let path = repo.join(rel);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                out.push('\n');
                out.push_str(&substitute_role_text(&text, branches, commands));
            }
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "role system_prompt file not readable; using preamble only");
            }
        }
    }
    out
}

/// A short, project-scoped statement of the branch pattern (`[branches]`,
/// docs/design/agent-host/operating-model.md), so every role always knows
/// which branch is the integration target and whether a separate release
/// branch exists, without needing to hardcode either name in role prose.
/// Same text for every agent of a project (not per-agent), so it stays part
/// of the cacheable stable prefix.
fn branches_sentence(branches: &BranchesConfig) -> String {
    match &branches.release {
        Some(release) => format!(
            "\nThis project's integration branch is `{}`: merge and push completed work there. \
            `{}` is the release branch -- never merge or push it except as the release step \
            (done by the orchestrator or the human, not by the ordinary worker/manager merge).\n",
            branches.integration, release
        ),
        None => format!(
            "\nThis project's integration branch is `{}`: merge and push completed work there. \
            There is no separate release branch; releases are tags on it.\n",
            branches.integration
        ),
    }
}

/// Substitutes `{{commands.check}}` and `{{branches.integration}}`/`{{branches.release}}`
/// in a role's own `system_prompt` file text, the same convention skill files use
/// (`sync::substitute_placeholders`). `{{branches.release}}`
/// is left as-is (nothing sensible to substitute) when no release branch is
/// configured; [`branches_sentence`] already states that plainly.
fn substitute_role_text(
    text: &str,
    branches: &BranchesConfig,
    commands: &CommandsConfig,
) -> String {
    let out = text.replace("{{commands.check}}", &commands.check);
    let out = out.replace("{{branches.integration}}", &branches.integration);
    match &branches.release {
        Some(release) => out.replace("{{branches.release}}", release),
        None => out,
    }
}

/// The rendered `--append-system-prompt-file` contents for one agent:
/// [`stable_system_prompt`], then a short identity sentence (name, role,
/// cwd, branch) appended last, so an agent always knows these facts even
/// with no first message (docs/questions/open/v1-follow-ups-from-the-build-9c6e.md).
/// The identity sentence goes at the end, not the start, so the long shared
/// prefix above it still matches across agents of the same role and the
/// prompt cache still holds for that part.
#[allow(clippy::too_many_arguments)] // flat per-agent facts; a struct would only rename them
pub fn render_system_prompt(
    role_name: &str,
    role: &Role,
    repo: &Path,
    branches: &BranchesConfig,
    commands: &CommandsConfig,
    agent_name: &str,
    cwd: &Path,
    branch: Option<&str>,
) -> String {
    let mut out = stable_system_prompt(role_name, role, repo, branches, commands);
    let branch_clause = branch
        .map(|b| format!(" on branch {b}"))
        .unwrap_or_default();
    out.push_str(&format!(
        "\nYou are {agent_name} (role {role_name}) in {}{branch_clause}.\n",
        cwd.display()
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_builtin_roles() {
        let cfg = Config::default();
        let worker = &cfg.roles["worker"];
        assert_eq!(worker.model, "sonnet");
        assert_eq!(worker.effort.as_deref(), Some("medium"));
        assert_eq!(worker.workdir, Workdir::Worktree);
        assert_eq!(worker.permission_mode, "acceptEdits");
        assert!(worker.allowed_tools.contains(&"Edit".to_string()));
        for tool in ["SendMessage", "Workflow", "ScheduleWakeup", "RemoteTrigger"] {
            assert!(
                worker.disallowed_tools.contains(&tool.to_string()),
                "worker should deny {tool}"
            );
        }
        assert!(
            !worker.disallowed_tools.contains(&"Agent".to_string()),
            "worker should allow Agent"
        );
        assert!(worker.stop_check, "worker should have stop_check on");

        let manager = &cfg.roles["manager"];
        assert_eq!(manager.workdir, Workdir::Repo);
        assert_eq!(manager.permission_mode, "dontAsk");
        assert!(manager.resume_on_restart);
        assert!(manager.allowed_tools.contains(&"Bash(git *)".to_string()));
        for tool in ["SendMessage", "Workflow", "ScheduleWakeup", "RemoteTrigger"] {
            assert!(
                manager.disallowed_tools.contains(&tool.to_string()),
                "manager should deny {tool}"
            );
        }
        assert!(
            !manager.disallowed_tools.contains(&"Agent".to_string()),
            "manager should allow Agent"
        );
        assert!(!manager.stop_check, "manager should have stop_check off");

        let orchestrator = &cfg.roles["orchestrator"];
        assert_eq!(orchestrator.workdir, Workdir::Repo);
        assert!(orchestrator.resume_on_restart);
        assert!(
            !orchestrator
                .allowed_tools
                .contains(&"Bash(git *)".to_string())
        );
        for tool in ["SendMessage", "Workflow", "RemoteTrigger"] {
            assert!(
                orchestrator.disallowed_tools.contains(&tool.to_string()),
                "orchestrator should deny {tool}"
            );
        }
        assert!(
            !orchestrator.disallowed_tools.contains(&"Agent".to_string()),
            "orchestrator should allow Agent"
        );
        // Exempted: the orchestrator paces its own loop with these.
        for tool in ["ScheduleWakeup", "CronCreate", "CronDelete", "CronList"] {
            assert!(
                !orchestrator.disallowed_tools.contains(&tool.to_string()),
                "orchestrator should not deny {tool}"
            );
        }
        assert!(
            !orchestrator.stop_check,
            "orchestrator should have stop_check off"
        );
    }

    #[test]
    fn effective_allowed_tools_always_has_bridle() {
        let cfg = Config::default();
        for role in cfg.roles.values() {
            assert!(
                role.effective_allowed_tools()
                    .contains(&"Bash(bridle *)".to_string())
            );
        }
    }

    #[test]
    fn parses_sample_toml_with_overrides_and_custom_role() {
        let toml = r#"
            [daemon]
            listen = "0.0.0.0:7433"
            stall_after = "5m"
            claim_lease_after = "15m"
            stop_grace = "45s"

            [roles.worker]
            model = "opus"
            allowed_tools = ["Bash", "Read"]

            [roles.reviewer]
            model = "sonnet"
            workdir = "repo"
            permission_mode = "plan"
            allowed_tools = ["Read", "Glob"]
        "#;
        let cfg = Config::parse(toml).expect("parse");

        assert_eq!(cfg.listen, "0.0.0.0:7433".parse().expect("addr"));
        assert_eq!(cfg.stall_after, Duration::from_secs(5 * 60));
        assert_eq!(cfg.claim_lease_after, Duration::from_secs(15 * 60));
        assert_eq!(cfg.stop_grace, Duration::from_secs(45));

        // Overridden field changes; untouched fields keep the built-in default.
        let worker = &cfg.roles["worker"];
        assert_eq!(worker.model, "opus");
        assert_eq!(
            worker.allowed_tools,
            vec!["Bash".to_string(), "Read".to_string()]
        );
        assert_eq!(worker.workdir, Workdir::Worktree);
        assert_eq!(worker.permission_mode, "acceptEdits");

        // Unknown role name: new role, worker-like defaults except what's set.
        let reviewer = &cfg.roles["reviewer"];
        assert_eq!(reviewer.model, "sonnet");
        assert_eq!(reviewer.workdir, Workdir::Repo);
        assert_eq!(reviewer.permission_mode, "plan");
        // Inherited from worker defaults ("HEAD"); `apply_branches` only
        // resolves "HEAD" to the integration branch for worktree roles, and
        // this custom role's workdir is "repo".
        assert_eq!(reviewer.base, "HEAD");
    }

    #[test]
    fn project_config_adds_to_the_default_deny_list_without_repeating_it() {
        let toml = r#"
            [roles.worker]
            disallowed_tools = ["Bash(git push *)"]
        "#;
        let cfg = Config::parse(toml).expect("parse");
        let worker = &cfg.roles["worker"];
        // The project's own addition is there...
        assert!(
            worker
                .disallowed_tools
                .contains(&"Bash(git push *)".to_string())
        );
        // ...and so is everything the built-in default already denied, with
        // no need to re-list it.
        for tool in ["SendMessage", "Workflow", "ScheduleWakeup", "RemoteTrigger"] {
            assert!(
                worker.disallowed_tools.contains(&tool.to_string()),
                "built-in denial {tool} should survive a project addition"
            );
        }
        // Agent is not in the deny list by default.
        assert!(
            !worker.disallowed_tools.contains(&"Agent".to_string()),
            "Agent should not be in the deny list"
        );
    }

    #[test]
    fn bridles_own_config_parses() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = Config::load(&repo).expect("bridle's .bridle/config.toml parses");
        assert_eq!(config.commands.check, "just check");
        let manager = &config.roles["manager"];
        assert!(manager.start_prompt.is_some());
        assert!(manager.max_budget_usd.is_some());
        for role in ["worker", "manager"] {
            let prompt = config.roles[role].system_prompt.as_ref().expect("prompt");
            assert!(repo.join(prompt).is_file(), "{} exists", prompt.display());
        }
        // product-manager is a custom role, so it falls back to
        // Role::worker_default() as its merge base (stop_check: true); config
        // must turn it back off explicitly, since it isn't the worker role.
        assert!(!config.roles["product-manager"].stop_check);
    }

    #[test]
    fn base_role_prompts_render_project_neutral() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let branches = BranchesConfig {
            integration: "trunk-x".to_string(),
            ..BranchesConfig::default()
        };
        let commands = CommandsConfig {
            check: "make ci".to_string(),
        };
        for name in ["worker", "manager", "product-manager"] {
            let role = Role {
                system_prompt: Some(format!("workflow/base/roles/{name}.md").into()),
                ..Role::worker_default()
            };
            let rendered = stable_system_prompt(name, &role, &repo, &branches, &commands);
            assert!(rendered.contains("make ci"), "{name}: check command");
            assert!(rendered.contains("trunk-x"), "{name}: integration branch");
            assert!(
                !rendered.contains("just check"),
                "{name}: stray `just check`"
            );
            assert!(
                !rendered.contains("{{"),
                "{name}: unsubstituted placeholder"
            );
            for word in rendered.split(|c: char| !(c.is_alphanumeric() || c == '/' || c == '-')) {
                assert!(
                    word != "main" && word != "origin/main",
                    "{name}: stray `main`"
                );
            }
        }
    }

    #[test]
    fn bridles_own_manager_prompt_has_no_unsubstituted_branch_placeholder() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = Config::load(&repo).expect("bridle's .bridle/config.toml parses");
        let manager = &config.roles["manager"];
        let rendered = stable_system_prompt(
            "manager",
            manager,
            &repo,
            &config.branches,
            &config.commands,
        );
        assert!(
            !rendered.contains("{{branches."),
            "manager.md should have every {{{{branches.*}}}} placeholder substituted"
        );
        // Bridle's own project defaults to trunk on "main".
        assert!(rendered.contains("integration branch is `main`"));
        assert!(rendered.contains("push origin main"));
    }

    #[test]
    fn custom_role_falling_back_to_worker_default_can_turn_stop_check_back_off() {
        let toml = r#"
            [roles.product-manager]
            model = "sonnet"
            stop_check = false
        "#;
        let cfg = Config::parse(toml).expect("parse");
        let pm = &cfg.roles["product-manager"];
        // Confirms the fallback base really is worker_default (stop_check: true)
        // and that the project config can override it.
        assert!(Role::worker_default().stop_check);
        assert!(!pm.stop_check);
    }

    #[test]
    fn context_config_defaults_and_parses_overrides() {
        let cfg = Config::default();
        assert_eq!(cfg.context.wind_down_at.get("default"), 200_000.0);
        assert_eq!(cfg.context.wind_down_at.get("worker"), 120_000.0);
        assert_eq!(cfg.context.wind_down_grace, Duration::from_secs(5 * 60));

        let toml = r#"
            [context]
            wind_down_grace = "45s"

            [context.wind_down_at]
            default = 50000
            worker = 30000
        "#;
        let cfg = Config::parse(toml).expect("parse");
        assert_eq!(cfg.context.wind_down_at.get("default"), 50_000.0);
        assert_eq!(cfg.context.wind_down_at.get("worker"), 30_000.0);
        assert_eq!(cfg.context.wind_down_grace, Duration::from_secs(45));
    }

    #[test]
    fn budget_schedule_defaults_to_empty() {
        assert!(BudgetConfig::default().schedule.is_empty());
    }

    /// Parses `[[budget.schedule]]` the way `~/.bridle/config.toml` (the
    /// machine-wide side) does: unrestricted, since only a *project*
    /// config's periods are held to the machine-wide ceiling.
    fn parse_machine_budget(toml: &str) -> BudgetConfig {
        let raw: RawConfig = toml::from_str(toml).expect("toml");
        BudgetConfig::default()
            .merge(raw.budget.unwrap_or_default())
            .expect("merge")
    }

    #[test]
    fn budget_schedule_parses_named_periods_with_all_and_a_day_list() {
        let toml = r#"
            [[budget.schedule]]
            name = "night"
            days = "all"
            start = "23:00"
            end = "07:00"
            hold_at = 90
            wind_down_at = 93
            stop_at = 95

            [[budget.schedule]]
            name = "workday"
            days = ["mon", "tue", "wed", "thu", "fri"]
            start = "09:00"
            end = "17:00"
            hold_at = 85
            wind_down_at = 92
            stop_at = 95
        "#;
        let budget = parse_machine_budget(toml);
        assert_eq!(budget.schedule.len(), 2);

        let night = &budget.schedule[0];
        assert_eq!(night.name, "night");
        assert_eq!(night.days.len(), 7);
        assert_eq!(night.hold_at, 90.0);
        assert_eq!(night.wind_down_at, 93.0);
        assert_eq!(night.stop_at, 95.0);

        let workday = &budget.schedule[1];
        assert_eq!(
            workday.days,
            vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ]
        );
        assert_eq!(workday.hold_at, 85.0);
    }

    #[test]
    fn budget_schedule_rejects_a_bad_day_name() {
        let toml = r#"
            [[budget.schedule]]
            name = "oops"
            days = ["someday"]
            start = "09:00"
            end = "17:00"
            hold_at = 85
            wind_down_at = 92
            stop_at = 95
        "#;
        let err = Config::parse(toml).expect_err("bad day should fail");
        assert!(matches!(err, ConfigError::BadSchedule { .. }), "{err}");
    }

    #[test]
    fn budget_schedule_rejects_a_bad_time() {
        let toml = r#"
            [[budget.schedule]]
            name = "oops"
            days = "all"
            start = "9am"
            end = "17:00"
            hold_at = 85
            wind_down_at = 92
            stop_at = 95
        "#;
        let err = Config::parse(toml).expect_err("bad time should fail");
        assert!(matches!(err, ConfigError::BadSchedule { .. }), "{err}");
    }

    #[test]
    fn budget_schedule_period_may_not_raise_five_hour_above_machine_wide() {
        // The machine-wide default five_hour hold_at is 80; a project's
        // schedule period may not raise it, same ThresholdTooHigh rule as
        // the plain [budget] thresholds.
        let toml = r#"
            [[budget.schedule]]
            name = "night"
            days = "all"
            start = "23:00"
            end = "07:00"
            hold_at = 96
            wind_down_at = 97
            stop_at = 98
        "#;
        let err = Config::parse(toml).expect_err("raising the ceiling should fail");
        assert!(matches!(err, ConfigError::ThresholdTooHigh { .. }), "{err}");
    }

    #[test]
    fn schedule_period_matches_a_plain_and_a_midnight_crossing_range() {
        use chrono::{NaiveDate, TimeZone};

        // 2024-01-01 is a Monday.
        let at = |h: u32, m: u32| {
            Local
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2024, 1, 1)
                        .expect("date")
                        .and_hms_opt(h, m, 0)
                        .expect("time"),
                )
                .single()
                .expect("unambiguous")
        };

        let workday = SchedulePeriod {
            name: "workday".to_string(),
            days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            start: NaiveTime::from_hms_opt(9, 0, 0).expect("time"),
            end: NaiveTime::from_hms_opt(17, 0, 0).expect("time"),
            hold_at: 85.0,
            wind_down_at: 92.0,
            stop_at: 95.0,
        };
        assert!(workday.matches(at(10, 0)));
        assert!(!workday.matches(at(8, 59)));
        assert!(!workday.matches(at(17, 0)));

        let night = SchedulePeriod {
            name: "night".to_string(),
            days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ],
            start: NaiveTime::from_hms_opt(23, 0, 0).expect("time"),
            end: NaiveTime::from_hms_opt(7, 0, 0).expect("time"),
            hold_at: 90.0,
            wind_down_at: 93.0,
            stop_at: 95.0,
        };
        // Both sides of midnight match.
        assert!(night.matches(at(23, 30)));
        assert!(night.matches(at(6, 0)));
        assert!(!night.matches(at(12, 0)));
    }

    #[test]
    fn default_task_prefix_from_project_name() {
        assert_eq!(default_task_prefix("bridle"), "br");
        assert_eq!(default_task_prefix("Hexworks"), "he");
        assert_eq!(default_task_prefix("9lives"), "9l");
        assert_eq!(default_task_prefix("a"), "ax");
        assert_eq!(default_task_prefix("---"), "tk");
        assert_eq!(default_task_prefix(""), "tk");
    }

    #[test]
    fn tasks_prefix_config_overrides_the_default() {
        let cfg = Config::parse("[tasks]\nprefix = \"hx\"\n").expect("parse");
        assert_eq!(cfg.task_prefix.as_deref(), Some("hx"));

        let cfg = Config::default();
        assert_eq!(cfg.task_prefix, None);
    }

    #[test]
    fn duration_parsing() {
        assert_eq!(
            parse_duration("10m").expect("10m"),
            Duration::from_secs(600)
        );
        assert_eq!(parse_duration("30s").expect("30s"), Duration::from_secs(30));
        assert_eq!(parse_duration("1h").expect("1h"), Duration::from_secs(3600));
        assert!(parse_duration("10").is_err());
        assert!(parse_duration("m").is_err());
        assert!(parse_duration("10x").is_err());
    }

    #[test]
    fn stable_prompt_has_no_per_agent_data() {
        let repo = Path::new("/does/not/matter");
        let branches = BranchesConfig::default();
        for (name, role) in Config::default().roles {
            let rendered =
                stable_system_prompt(&name, &role, repo, &branches, &CommandsConfig::default());
            assert!(
                !rendered.contains("BRIDLE_AGENT_ID="),
                "stable prompt must not embed an id"
            );
            assert!(
                !rendered.contains(repo.to_str().expect("utf8 path")),
                "stable prompt must not embed a path"
            );
            assert!(
                !rendered.contains("2026"),
                "stable prompt must not embed a timestamp"
            );
        }
    }

    #[test]
    fn stable_prompt_identical_for_two_agents_of_the_same_role() {
        let repo = Path::new("/repo/a");
        let branches = BranchesConfig::default();
        let role = Role::worker_default();
        let a = stable_system_prompt("worker", &role, repo, &branches, &CommandsConfig::default());
        let b = stable_system_prompt("worker", &role, repo, &branches, &CommandsConfig::default());
        assert_eq!(a, b);
    }

    #[test]
    fn stable_prompt_differs_by_role() {
        let repo = Path::new("/repo/a");
        let branches = BranchesConfig::default();
        let worker = stable_system_prompt(
            "worker",
            &Role::worker_default(),
            repo,
            &branches,
            &CommandsConfig::default(),
        );
        let manager = stable_system_prompt(
            "manager",
            &Role::manager_default(),
            repo,
            &branches,
            &CommandsConfig::default(),
        );
        assert_ne!(worker, manager);
    }

    #[test]
    fn rendered_prompt_carries_identity_after_the_shared_prefix() {
        let repo = Path::new("/repo/a");
        let branches = BranchesConfig::default();
        let role = Role::worker_default();
        let stable =
            stable_system_prompt("worker", &role, repo, &branches, &CommandsConfig::default());
        let a = render_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "worker-1",
            Path::new("/repo/a/wt/worker-1"),
            Some("bridle/worker-1"),
        );
        let b = render_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "worker-2",
            Path::new("/repo/a/wt/worker-2"),
            Some("bridle/worker-2"),
        );

        // The long, cacheable part is unaffected by which agent is asking.
        assert!(a.starts_with(&stable));
        assert!(b.starts_with(&stable));
        // Each agent's own identity comes after it.
        assert!(a.contains(
            "You are worker-1 (role worker) in /repo/a/wt/worker-1 on branch bridle/worker-1."
        ));
        assert!(b.contains(
            "You are worker-2 (role worker) in /repo/a/wt/worker-2 on branch bridle/worker-2."
        ));
        assert_ne!(a, b);
    }

    #[test]
    fn models_config_defaults_match_usage_and_budget_md() {
        let cfg = Config::default();
        assert_eq!(cfg.models.by_role["manager"], vec!["opus", "sonnet"]);
        assert_eq!(cfg.models.by_role["planner"], vec!["opus", "sonnet"]);
        assert_eq!(cfg.models.by_role["reviewer"], vec!["sonnet", "opus"]);
        assert_eq!(cfg.models.by_role["worker"], vec!["sonnet", "haiku"]);
        assert_eq!(cfg.models.by_role["chore"], vec!["haiku"]);
        assert_eq!(cfg.models.by_role["explore"], vec!["sonnet"]);
    }

    #[test]
    fn models_config_candidates_falls_back_to_role_model_when_unlisted() {
        let cfg = Config::default();
        let mut role = Role::worker_default();
        role.model = "opus".to_string();
        assert_eq!(
            cfg.models.candidates("some_custom_role", &role),
            vec!["opus"]
        );
    }

    #[test]
    fn a_roles_model_below_the_builtin_top_choice_trims_but_does_not_step_back_up() {
        // The built-in list for `manager` is ["opus", "sonnet"]. Pinning the
        // role to "sonnet" with no explicit `[models]` entry should trim off
        // "opus" (never step back up to it) but keep "sonnet" onward.
        let toml = r#"
            [roles.manager]
            model = "sonnet"
        "#;
        let cfg = Config::parse(toml).expect("parse");
        let manager = &cfg.roles["manager"];
        assert_eq!(manager.model, "sonnet");
        assert_eq!(cfg.models.candidates("manager", manager), vec!["sonnet"]);
    }

    #[test]
    fn a_project_config_can_replace_a_roles_model_list_outright() {
        let toml = r#"
            [models]
            worker = ["haiku"]
            explore = ["opus", "haiku"]
        "#;
        let cfg = Config::parse(toml).expect("parse");
        assert_eq!(cfg.models.by_role["worker"], vec!["haiku"]);
        assert_eq!(cfg.models.by_role["explore"], vec!["opus", "haiku"]);
        // Untouched roles keep the built-in default.
        assert_eq!(cfg.models.by_role["manager"], vec!["opus", "sonnet"]);
    }

    #[test]
    fn commands_check_defaults_to_just_check_and_can_be_overridden() {
        let cfg = Config::default();
        assert_eq!(cfg.commands.check, "just check");

        let cfg = Config::parse("[commands]\ncheck = \"make check\"\n").expect("parse");
        assert_eq!(cfg.commands.check, "make check");
    }

    #[test]
    fn branches_default_to_trunk_on_main_with_no_release_branch() {
        let cfg = Config::default();
        assert_eq!(cfg.branches.integration, "main");
        assert_eq!(cfg.branches.release, None);
        // Bridle's own project needs no config change: the default worker
        // base is "HEAD" until `apply_branches` (only run by `Config::load`/
        // `parse`, not bare `Config::default`) resolves it.
        assert_eq!(cfg.roles["worker"].base, "HEAD");
    }

    #[test]
    fn dev_and_release_pattern_targets_dev_not_main() {
        let cfg = Config::parse(
            r#"
            [branches]
            integration = "dev"
            release = "main"
        "#,
        )
        .expect("parse");
        assert_eq!(cfg.branches.integration, "dev");
        assert_eq!(cfg.branches.release.as_deref(), Some("main"));

        // The manager's and worker's worktree base -- what new work branches
        // from and what a merge/push targets -- follows the integration
        // branch, not "main".
        assert_eq!(cfg.roles["worker"].base, "dev");

        // Pushing the release branch is mechanically denied for every
        // ordinary role, not just documented.
        for role in ["worker", "manager"] {
            assert!(
                cfg.roles[role]
                    .disallowed_tools
                    .contains(&"Bash(git push origin main)".to_string()),
                "{role} should be denied pushing the release branch"
            );
        }
        // The orchestrator legitimately cuts releases, so it's exempt.
        assert!(
            !cfg.roles["orchestrator"]
                .disallowed_tools
                .contains(&"Bash(git push origin main)".to_string())
        );
    }

    #[test]
    fn a_trial_branch_name_is_the_only_integration_target_and_main_dev_are_never_touched() {
        // Stands in for the trial's own fixed branch name, `bridle-adopt`
        // (63rv): any name works, nothing here is specific to that literal
        // string.
        let cfg = Config::parse(
            r#"
            [branches]
            integration = "bridle-adopt"
        "#,
        )
        .expect("parse");
        assert_eq!(cfg.branches.integration, "bridle-adopt");
        assert_eq!(cfg.branches.release, None);

        assert_eq!(cfg.roles["worker"].base, "bridle-adopt");
        assert_ne!(cfg.roles["worker"].base, "main");
        assert_ne!(cfg.roles["worker"].base, "dev");
        // No release branch is configured, so no push is mechanically
        // denied on its account -- there is no `main`/`dev` for a trial to
        // touch in the first place.
        assert!(
            !cfg.roles["manager"]
                .disallowed_tools
                .iter()
                .any(|t| t.contains("push origin main") || t.contains("push origin dev"))
        );
    }

    #[test]
    fn an_explicit_role_base_is_not_overridden_by_branches() {
        let cfg = Config::parse(
            r#"
            [branches]
            integration = "dev"

            [roles.worker]
            base = "some-other-ref"
        "#,
        )
        .expect("parse");
        assert_eq!(cfg.roles["worker"].base, "some-other-ref");
    }

    #[test]
    fn workflow_path_is_parsed_and_defaults_to_none() {
        assert_eq!(Config::parse("").expect("parse").workflow, None);
        let cfg = Config::parse(r#"workflow = "workflow""#).expect("parse");
        assert_eq!(cfg.workflow.as_deref(), Some("workflow"));
    }

    #[test]
    fn rendered_prompt_without_branch_omits_the_branch_clause() {
        let repo = Path::new("/repo/a");
        let role = Role::manager_default();
        let rendered = render_system_prompt(
            "manager",
            &role,
            repo,
            &BranchesConfig::default(),
            &CommandsConfig::default(),
            "manager-1",
            repo,
            None,
        );
        assert!(rendered.contains("You are manager-1 (role manager) in /repo/a.\n"));
    }

    #[test]
    fn components_parse_with_chain_and_validation() {
        let cfg = Config::parse(
            r#"
            [components.client-games]
            paths = ["client-games/**"]
            docs = "docs/games"
            [components.dungeon]
            parent = "client-games"
            consumers = ["harness"]
            "#,
        )
        .expect("parse");
        assert_eq!(
            cfg.component_chain("dungeon"),
            Some(vec!["client-games", "dungeon"])
        );
        assert_eq!(
            cfg.component_chain("client-games"),
            Some(vec!["client-games"])
        );
        assert_eq!(cfg.component_chain("nope"), None);
        assert!(Config::parse("").expect("parse").components.is_empty());

        let err = Config::parse("[components.a]\nparent = \"zzz\"").unwrap_err();
        assert!(err.to_string().contains("not a defined component"), "{err}");
        let err = Config::parse("[components.a]\nparent = \"b\"\n[components.b]\nparent = \"a\"")
            .unwrap_err();
        assert!(err.to_string().contains("cycle"), "{err}");
        let err = Config::parse("[components.a]\nbogus = 1").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }
}
