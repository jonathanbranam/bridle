//! Daemon config: `<repo>/.bridle/config.toml` over built-in defaults.
//! See docs/design/agent-host/roles-and-config.md.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

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
            disallowed_tools: Vec::new(),
            system_prompt: None,
            autostart: false,
            resume_on_restart: false,
            start_prompt: None,
            max_budget_usd: None,
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
            disallowed_tools: Vec::new(),
            system_prompt: None,
            autostart: false,
            resume_on_restart: true,
            start_prompt: None,
            max_budget_usd: None,
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
            disallowed_tools: Vec::new(),
            system_prompt: None,
            autostart: false,
            resume_on_restart: true,
            start_prompt: None,
            max_budget_usd: None,
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
        if let Some(v) = raw.disallowed_tools {
            self.disallowed_tools = v;
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

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub stall_after: Duration,
    pub stop_grace: Duration,
    pub roles: BTreeMap<String, Role>,
    pub budget: BudgetConfig,
    pub models: ModelsConfig,
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
            stop_grace: Duration::from_secs(30),
            roles,
            budget: BudgetConfig::default(),
            models: ModelsConfig::default(),
        }
    }
}

impl Config {
    /// Loads the machine-wide `[budget]` section
    /// (`$BRIDLE_HOME/config.toml`, else `~/.bridle/config.toml`; see
    /// `bridle_api::discovery::bridle_home`) merged over the built-in
    /// defaults. Missing file is not an error.
    fn load_machine_budget() -> Result<BudgetConfig, ConfigError> {
        let path = bridle_api::discovery::bridle_home().join("config.toml");
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
        let machine_budget = Self::load_machine_budget()?;
        let path = repo.join(".bridle").join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse_with_budget(&text, machine_budget, &path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config {
                budget: machine_budget,
                ..Config::default()
            }),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
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
            if let Some(s) = d.stop_grace {
                config.stop_grace = parse_duration(&s)?;
            }
        }

        for (name, raw_role) in raw.roles.unwrap_or_default() {
            let base = config
                .roles
                .remove(&name)
                .unwrap_or_else(Role::worker_default);
            config.roles.insert(name, base.merge(raw_role));
        }

        if let Some(raw_budget) = raw.budget {
            config.budget = config.budget.merge_project(raw_budget, path)?;
        }

        if let Some(raw_models) = raw.models {
            config.models = config.models.merge(raw_models);
        }

        Ok(config)
    }
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
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDaemon {
    #[serde(default)]
    listen: Option<String>,
    #[serde(default)]
    stall_after: Option<String>,
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
pub fn stable_system_prompt(role_name: &str, role: &Role, repo: &Path) -> String {
    let mut out = String::from(PREAMBLE);
    if let Some(suffix) = role_preamble_suffix(role_name) {
        out.push_str(suffix);
    }
    if let Some(rel) = &role.system_prompt {
        let path = repo.join(rel);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                out.push('\n');
                out.push_str(&text);
            }
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "role system_prompt file not readable; using preamble only");
            }
        }
    }
    out
}

/// The rendered `--append-system-prompt-file` contents for one agent:
/// [`stable_system_prompt`], then a short identity sentence (name, role,
/// cwd, branch) appended last, so an agent always knows these facts even
/// with no first message (docs/questions/open/v1-follow-ups-from-the-build-9c6e.md).
/// The identity sentence goes at the end, not the start, so the long shared
/// prefix above it still matches across agents of the same role and the
/// prompt cache still holds for that part.
pub fn render_system_prompt(
    role_name: &str,
    role: &Role,
    repo: &Path,
    agent_name: &str,
    cwd: &Path,
    branch: Option<&str>,
) -> String {
    let mut out = stable_system_prompt(role_name, role, repo);
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

        let manager = &cfg.roles["manager"];
        assert_eq!(manager.workdir, Workdir::Repo);
        assert_eq!(manager.permission_mode, "dontAsk");
        assert!(manager.resume_on_restart);
        assert!(manager.allowed_tools.contains(&"Bash(git *)".to_string()));

        let orchestrator = &cfg.roles["orchestrator"];
        assert_eq!(orchestrator.workdir, Workdir::Repo);
        assert!(orchestrator.resume_on_restart);
        assert!(
            !orchestrator
                .allowed_tools
                .contains(&"Bash(git *)".to_string())
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
        assert_eq!(reviewer.base, "HEAD"); // inherited from worker defaults
    }

    #[test]
    fn bridles_own_config_parses() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = Config::load(&repo).expect("bridle's .bridle/config.toml parses");
        let manager = &config.roles["manager"];
        assert!(manager.start_prompt.is_some());
        assert!(manager.max_budget_usd.is_some());
        for role in ["worker", "manager"] {
            let prompt = config.roles[role].system_prompt.as_ref().expect("prompt");
            assert!(repo.join(prompt).is_file(), "{} exists", prompt.display());
        }
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
        for (name, role) in Config::default().roles {
            let rendered = stable_system_prompt(&name, &role, repo);
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
        let role = Role::worker_default();
        let a = stable_system_prompt("worker", &role, repo);
        let b = stable_system_prompt("worker", &role, repo);
        assert_eq!(a, b);
    }

    #[test]
    fn stable_prompt_differs_by_role() {
        let repo = Path::new("/repo/a");
        let worker = stable_system_prompt("worker", &Role::worker_default(), repo);
        let manager = stable_system_prompt("manager", &Role::manager_default(), repo);
        assert_ne!(worker, manager);
    }

    #[test]
    fn rendered_prompt_carries_identity_after_the_shared_prefix() {
        let repo = Path::new("/repo/a");
        let role = Role::worker_default();
        let stable = stable_system_prompt("worker", &role, repo);
        let a = render_system_prompt(
            "worker",
            &role,
            repo,
            "worker-1",
            Path::new("/repo/a/wt/worker-1"),
            Some("bridle/worker-1"),
        );
        let b = render_system_prompt(
            "worker",
            &role,
            repo,
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
    fn rendered_prompt_without_branch_omits_the_branch_clause() {
        let repo = Path::new("/repo/a");
        let role = Role::manager_default();
        let rendered = render_system_prompt("manager", &role, repo, "manager-1", repo, None);
        assert!(rendered.contains("You are manager-1 (role manager) in /repo/a.\n"));
    }
}
