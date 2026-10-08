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
    #[error("invalid [tmux]: {0}")]
    BadTmux(String),
    #[error("invalid [orchestrator]: {0}")]
    BadOrchestrator(String),
    #[error("invalid [sessions]: {0}")]
    BadSessions(String),
    #[error("invalid [[budget.schedule]] {name:?}: {reason}")]
    BadSchedule { name: String, reason: String },
    #[error("[components.{id}] parent {parent:?} is not a defined component")]
    UnknownComponentParent { id: String, parent: String },
    #[error("[components] parent cycle: {}", .0.join(" -> "))]
    ComponentCycle(Vec<String>),
    #[error("[ports] range = [{0}, {1}] must have low <= high")]
    BadPortRange(u16, u16),
    #[error("[worktrees] copy entry {0:?} must be a relative path without \"..\"")]
    BadCopyPath(String),
    #[error("[worktrees] {0}")]
    BadWorktreeLayout(String),
    #[error("workflow path {value:?}: {reason}")]
    BadWorkflowPath { value: String, reason: String },
    #[error(
        "workflow directory {} is missing or unreadable ({reason}); set `workflow` in \
         ~/.bridle/config.toml to this machine's checkout, or fix it in .bridle/config.toml",
        path.display()
    )]
    WorkflowDirMissing { path: PathBuf, reason: String },

    #[error(
        "workflow = {url:?} is a git url, which bridle does not resolve; set `workflow` in \
         ~/.bridle/config.toml to a local checkout of it, or fix it in .bridle/config.toml"
    )]
    WorkflowGitUrl { url: String },
}

/// Where `bridle init` vendors the base workflow, repo-relative.
/// A role name as the daemon looks it up: the pre-9j2h `product-manager` reads as
/// `project-manager`, so stored agents and unmigrated configs keep PM permissions.
pub fn canonical_role(name: &str) -> String {
    if name == "product-manager" {
        "project-manager".to_string()
    } else {
        name.to_string()
    }
}

pub const VENDORED_WORKFLOW: &str = ".bridle/workflow";

/// Expands a leading `~` (to `$HOME`) and `$VAR` / `${VAR}` in a `workflow` value.
/// An unset variable is an error, not an empty string: that would silently point the
/// workflow at the wrong directory.
pub fn expand_path(value: &str) -> Result<String, ConfigError> {
    let bad = |reason: String| ConfigError::BadWorkflowPath {
        value: value.to_string(),
        reason,
    };
    let lookup = |name: &str| {
        std::env::var(name)
            .ok()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| bad(format!("${name} is not set")))
    };
    let mut out = String::new();
    let mut rest = value;
    if rest == "~" || rest.starts_with("~/") {
        out.push_str(&lookup("HOME")?);
        rest = &rest[1..];
    }
    let mut chars = rest.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        let tail = &rest[i + 1..];
        let (name, consumed) = if let Some(inner) = tail.strip_prefix('{') {
            let end = inner
                .find('}')
                .ok_or_else(|| bad("unterminated ${".to_string()))?;
            (&inner[..end], end + 2)
        } else {
            let end = tail
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(tail.len());
            (&tail[..end], end)
        };
        if name.is_empty() {
            out.push('$');
            continue;
        }
        out.push_str(&lookup(name)?);
        for _ in 0..consumed {
            chars.next();
        }
    }
    Ok(out)
}

/// `[worktrees] layout`: where a new worker worktree is created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeLayout {
    /// `<workspace>/wt/<agent>`.
    Default,
    /// An explicit path template (absolute; `{task}`, `{agent}`, `{project}`).
    Root(String),
    /// Like `Root`, but the template names a directory holding the project's
    /// worktree (`<root>/<project>`) and each `[worktrees.pair.<name>]` member
    /// (`<root>/<name>`), side by side.
    Paired(String),
}

/// `[worktrees.pair.<name>] mode`: how a sibling repo joins a paired worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairMode {
    /// `git worktree add` on `bridle/<agent>` from the sibling's HEAD.
    Worktree,
    /// A symlink to the sibling's checkout, for read-only use.
    Symlink,
}

/// `[worktrees.pair.<name>]`: a sibling repo created alongside the project's worktree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairMember {
    pub path: PathBuf,
    pub mode: PairMode,
}

impl WorktreeLayout {
    fn parse(layout: Option<&str>, root: Option<String>) -> Result<Self, ConfigError> {
        let bad = |m: String| Err(ConfigError::BadWorktreeLayout(m));
        match (layout.unwrap_or("default"), root) {
            ("default", None) => Ok(Self::Default),
            ("default", Some(_)) => bad("root needs layout = \"root\" or \"paired\"".into()),
            ("root" | "paired", None) => bad("layout = \"root\" or \"paired\" needs root".into()),
            (kind @ ("root" | "paired"), Some(root)) => {
                if !Path::new(&root).is_absolute() {
                    return bad(format!("root {root:?} must be an absolute path"));
                }
                if root.split('/').any(|c| c == "..") {
                    return bad(format!("root {root:?} must not contain \"..\""));
                }
                let mut rest = root.as_str();
                while let Some(i) = rest.find('{') {
                    let tail = &rest[i..];
                    let Some(end) = tail.find('}') else {
                        return bad(format!("root {root:?} has an unclosed placeholder"));
                    };
                    let name = &tail[1..end];
                    if !["task", "agent", "project"].contains(&name) {
                        return bad(format!(
                            "root {root:?}: unknown placeholder {{{name}}} (use {{task}}, {{agent}}, {{project}})"
                        ));
                    }
                    rest = &tail[end + 1..];
                }
                if !root.contains("{task}") && !root.contains("{agent}") {
                    return bad(format!(
                        "root {root:?} must contain {{task}} or {{agent}} so each worktree gets its own path"
                    ));
                }
                Ok(if kind == "paired" {
                    Self::Paired(root)
                } else {
                    Self::Root(root)
                })
            }
            (other, _) => bad(format!(
                "invalid layout {other:?}: expected \"default\", \"root\" or \"paired\""
            )),
        }
    }

    /// The worktree path for a new agent. `task` is the agent's claimed task
    /// id, or its name when it has none (always so at spawn). `None` for
    /// `Default`: the caller uses the workspace's `wt/<agent>`. For `Paired`
    /// it is the project's member, `<root>/<project>`.
    pub fn resolve(&self, repo: &Path, task: &str, agent: &str) -> Option<PathBuf> {
        let (Self::Root(root) | Self::Paired(root)) = self else {
            return None;
        };
        let project = repo
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        let path = PathBuf::from(
            root.replace("{task}", task)
                .replace("{agent}", agent)
                .replace("{project}", project),
        );
        Some(if matches!(self, Self::Paired(_)) {
            path.join(project)
        } else {
            path
        })
    }
}

fn parse_pairs(
    raw: BTreeMap<String, RawPair>,
    layout: &WorktreeLayout,
) -> Result<BTreeMap<String, PairMember>, ConfigError> {
    let bad = |m: String| Err(ConfigError::BadWorktreeLayout(m));
    if raw.is_empty() {
        return if matches!(layout, WorktreeLayout::Paired(_)) {
            bad("layout = \"paired\" needs at least one [worktrees.pair.<name>]".into())
        } else {
            Ok(BTreeMap::new())
        };
    }
    if !matches!(layout, WorktreeLayout::Paired(_)) {
        return bad("[worktrees.pair.*] needs layout = \"paired\"".into());
    }
    let mut out = BTreeMap::new();
    for (name, p) in raw {
        if crate::worktree::validate_agent_name(&name).is_err() {
            return bad(format!(
                "pair name {name:?} must match [a-z0-9][a-z0-9-]{{0,39}}"
            ));
        }
        if !Path::new(&p.path).is_absolute() {
            return bad(format!("pair {name}: path {:?} must be absolute", p.path));
        }
        let mode = match p.mode.as_deref().unwrap_or("worktree") {
            "worktree" => PairMode::Worktree,
            "symlink" => PairMode::Symlink,
            other => {
                return bad(format!(
                    "pair {name}: invalid mode {other:?}: expected \"worktree\" or \"symlink\""
                ));
            }
        };
        out.insert(
            name,
            PairMember {
                path: PathBuf::from(p.path),
                mode,
            },
        );
    }
    Ok(out)
}

impl Config {
    /// Where each paired member lives beside `worktree` (the project's member
    /// of a paired layout): `<parent>/<name>`. Empty for other layouts.
    pub fn pair_paths(&self, worktree: &Path) -> Vec<(String, PathBuf)> {
        let Some(dir) = worktree.parent() else {
            return Vec::new();
        };
        self.worktree_pairs
            .keys()
            .map(|n| (n.clone(), dir.join(n)))
            .collect()
    }
}

/// True for a path that stays inside the repo root: relative, no `..`, not empty.
pub fn is_safe_relative(path: &str) -> bool {
    let p = Path::new(path);
    !path.is_empty()
        && p.components().all(|c| {
            matches!(
                c,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
}

/// `[components.<id>]`: a named scope inside the project (docs/design/components.md).
/// Everything is optional; `paths` and `consumers` are metadata for tooling, `docs` a
/// folder pointer, `parent` the nesting.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
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
    /// The built-in tool set the agent gets at all (`--tools`): unlisted
    /// tools' definitions are removed from its context, not just denied
    /// (docs/spikes/08-lean-context-findings.md). `None` leaves Claude Code's
    /// full set. Project config replaces the default, unlike `disallowed_tools`.
    pub tools: Option<Vec<String>>,
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
/// the way `SendMessage` did (docs/tickets/open/agents-can-use-claude-codes-own-sendmessage-78sp.md):
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

/// Agents may not write the focus-hours override or the `[[focus]]` config (ticket cvaq): only
/// the human, by hand. `Edit` rules cover `Write` too.
pub const DENY_FOCUS_FILES: [&str; 2] = ["Edit(~/.bridle/focus*)", "Edit(~/.bridle/config.toml)"];

/// Killing by name or pattern is never needed (rule `no-kill-by-name`, ticket 75h2); the
/// `bridle kill-guard` hook refuses the compound forms these rules can't match.
pub const DENY_PATTERN_KILLS: [&str; 2] = ["Bash(pkill *)", "Bash(killall *)"];

fn deny_list(extra: &[&[&str]]) -> Vec<String> {
    DENY_MESSAGING_AND_SUBAGENTS
        .iter()
        .chain(DENY_FOCUS_FILES.iter())
        .chain(DENY_PATTERN_KILLS.iter())
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
            tools: Some(vec![
                "Bash".into(),
                "Read".into(),
                "Edit".into(),
                "Write".into(),
                "Glob".into(),
                "Grep".into(),
            ]),
            disallowed_tools: deny_list(&[&DENY_SCHEDULING, &DENY_REMOTE_TRIGGERS]),
            system_prompt: None,
            autostart: false,
            resume_on_restart: false,
            start_prompt: None,
            max_budget_usd: None,
            stop_check: true,
        }
    }

    /// The worker plus WebSearch and WebFetch (the worker has no web tools, so a research task
    /// needs this role; br-rz4e).
    fn researcher_default() -> Self {
        let mut role = Self::worker_default();
        for tool in ["WebSearch", "WebFetch"] {
            role.allowed_tools.push(tool.into());
            role.tools.get_or_insert_with(Vec::new).push(tool.into());
        }
        role
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
                "Agent".into(),
            ],
            // Agent for `Explore` subagents; no Edit/Write (dontAsk denies them anyway).
            tools: Some(vec![
                "Bash".into(),
                "Read".into(),
                "Glob".into(),
                "Grep".into(),
                "Agent".into(),
            ]),
            disallowed_tools: deny_list(&[&DENY_SCHEDULING, &DENY_REMOTE_TRIGGERS]),
            system_prompt: None,
            autostart: true,
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
            // No whitelist until the orchestrator's launch is measured (ct8m step 4).
            tools: None,
            // Scheduling is exempted: the orchestrator paces its own loop
            // with `ScheduleWakeup` (docs/tickets/open/agents-can-use-claude-codes-own-sendmessage-78sp.md).
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
        if let Some(v) = raw.tools {
            self.tools = Some(v);
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
    /// Empty, with `start`/`end` `None`, for a schedule-less preset: it
    /// never matches by the clock and is reachable only through
    /// `bridle budget override <name>`.
    pub days: Vec<Weekday>,
    pub start: Option<NaiveTime>,
    pub end: Option<NaiveTime>,
    pub hold_at: f64,
    pub wind_down_at: f64,
    pub stop_at: f64,
    /// Applied through the live `max_workers` override while a
    /// `bridle budget override` of this period is in force.
    pub max_workers: Option<u32>,
}

impl SchedulePeriod {
    /// Whether host-local `now` falls in this period: today's weekday is
    /// listed, and the time-of-day is within `start..end`, a range that may
    /// cross midnight (e.g. `23:00..07:00`).
    pub fn matches(&self, now: DateTime<Local>) -> bool {
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return false;
        };
        in_window(&self.days, start, end, now)
    }
}

/// Whether `now` is within a period, which belongs to the day it starts: `start < end` is one
/// day; `end < start` runs from `start` on a listed day to `end` the next day, so the part after
/// midnight is matched against the previous day's `days`. `start == end` is empty. Shared by
/// `[[budget.schedule]]` and `[[focus]]`.
fn in_window(days: &[Weekday], start: NaiveTime, end: NaiveTime, now: DateTime<Local>) -> bool {
    let t = now.time();
    if start < end {
        days.contains(&now.weekday()) && t >= start && t < end
    } else if start > end {
        (t >= start && days.contains(&now.weekday()))
            || (t < end && days.contains(&now.weekday().pred()))
    } else {
        false
    }
}

/// `[[focus]]` mode. `Locked` parses but nothing acts on it yet (ticket cvaq, slice C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    Quiet,
    Locked,
}

/// One `[[focus]]` period (ticket cvaq): host-local days and a time-of-day window, like a
/// `[[budget.schedule]]` period, with a mode.
#[derive(Debug, Clone, PartialEq)]
pub struct FocusPeriod {
    pub name: String,
    pub days: Vec<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub mode: FocusMode,
}

impl FocusPeriod {
    pub fn matches(&self, now: DateTime<Local>) -> bool {
        in_window(&self.days, self.start, self.end, now)
    }

    /// The instant the matching run containing `now` ends: this period's own end.
    fn end_instant(&self, now: DateTime<Local>) -> Option<DateTime<Local>> {
        let mut date = now.date_naive();
        if self.start > self.end && now.time() >= self.start {
            date = date.succ_opt()?;
        }
        date.and_time(self.end).and_local_timezone(Local).earliest()
    }
}

/// How far `focus_end` follows touching or overlapping periods before giving up.
const FOCUS_FOLLOW_CAP_DAYS: i64 = 7;

/// When quiet (or locked) actually ends: starting from `first`, which matches `now`, follow
/// periods of the same mode that match at the end instant to the last end. The flag is true when
/// the follow hit the 7-day cap (periods covering the whole week), and the end is then `now`
/// plus the cap.
pub fn focus_end(
    periods: &[FocusPeriod],
    first: &FocusPeriod,
    now: DateTime<Local>,
) -> (DateTime<Local>, bool) {
    let cap = now + chrono::Duration::days(FOCUS_FOLLOW_CAP_DAYS);
    let mut at = now;
    let mut period = first;
    loop {
        let Some(end) = period.end_instant(at) else {
            return (at, false);
        };
        if end >= cap {
            return (cap, true);
        }
        match periods
            .iter()
            .find(|p| p.mode == first.mode && p.matches(end))
        {
            Some(next) => {
                at = end;
                period = next;
            }
            None => return (end, false),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawFocusPeriod {
    name: String,
    days: RawDays,
    start: String,
    end: String,
    #[serde(default)]
    mode: Option<String>,
}

fn weekdays(days: RawDays, name: &str) -> Result<Vec<Weekday>, ConfigError> {
    match days {
        RawDays::All(s) if s.eq_ignore_ascii_case("all") => Ok(vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]),
        RawDays::All(s) => Err(ConfigError::BadSchedule {
            name: name.to_string(),
            reason: format!("invalid days {s:?}: expected \"all\" or a list of mon..sun"),
        }),
        RawDays::List(days) => days.iter().map(|d| parse_weekday(d, name)).collect(),
    }
}

impl RawFocusPeriod {
    fn into_period(self) -> Result<FocusPeriod, ConfigError> {
        let mode = match self.mode.as_deref() {
            None | Some("quiet") => FocusMode::Quiet,
            Some("locked") => FocusMode::Locked,
            Some(other) => {
                return Err(ConfigError::BadSchedule {
                    name: self.name,
                    reason: format!("invalid mode {other:?}: expected \"quiet\" or \"locked\""),
                });
            }
        };
        warn_overnight(&self.name, &self.start, &self.end);
        Ok(FocusPeriod {
            start: parse_time_of_day(&self.start, &self.name)?,
            end: parse_time_of_day(&self.end, &self.name)?,
            days: weekdays(self.days, &self.name)?,
            name: self.name,
            mode,
        })
    }
}

/// The `[[focus]]` periods of `<home>/config.toml`; empty when the file or section is absent,
/// which turns focus hours off entirely.
pub fn focus_periods(home: &Path) -> Result<Vec<FocusPeriod>, ConfigError> {
    let path = home.join("config.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(ConfigError::Read { path, source }),
    };
    let raw: RawConfig = bridle_api::config_warn::parse(&text, &path.display().to_string())
        .map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source: Box::new(source),
        })?;
    raw.focus
        .unwrap_or_default()
        .into_iter()
        .map(RawFocusPeriod::into_period)
        .collect()
}

/// `focus_override_delay_minutes` of `<home>/config.toml`: how long a hand-written override
/// waits before it takes effect. Default 10. (Not `[focus] override_delay`: `[[focus]]` is an
/// array of tables, so `[focus]` can't also exist.)
pub fn focus_override_delay_minutes(home: &Path) -> u32 {
    std::fs::read_to_string(home.join("config.toml"))
        .ok()
        .and_then(|t| toml::from_str::<RawConfig>(&t).ok())
        .and_then(|raw| raw.focus_override_delay_minutes)
        .unwrap_or(10)
}

/// Whether the project at `repo` opted out of focus hours with `focus_hours = false` in its
/// `.bridle/config.toml`. Unreadable or absent config means no opt-out.
pub fn focus_opted_out(repo: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(repo.join(".bridle/config.toml")) else {
        return false;
    };
    toml::from_str::<RawConfig>(&text).is_ok_and(|raw| raw.focus_hours == Some(false))
}

/// The `[auto_mode] environment` lines of a config file; empty when the file or section is
/// absent or the file doesn't parse (a session must start regardless).
fn auto_mode_lines(path: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    toml::from_str::<RawConfig>(&text)
        .ok()
        .and_then(|raw| raw.auto_mode)
        .map(|a| a.environment)
        .unwrap_or_default()
}

/// A project line may only tighten the classifier: it has to open with one of these. A
/// committed project file must not be able to mark a repo, domain or bucket as trusted
/// (Claude Code ignores project settings for the same reason).
const AUTO_MODE_PROJECT_PREFIXES: [&str; 2] = ["sensitive:", "prod host:"];

/// The `autoMode.environment` array for a `bridle session` launch at `repo` (br-fc9a):
/// `"$defaults"`, a derived line naming the workspace, the machine's lines (trust lines), then
/// the project's lines that tighten. Project lines that don't start with `Sensitive:` or
/// `Prod host:` are dropped. Nothing is stored, so nothing goes stale.
pub fn auto_mode_environment(machine: &[String], project: &[String], repo: &Path) -> Vec<String> {
    let mut out = vec![
        "$defaults".to_string(),
        format!(
            "Trusted repo: {0} and its worktrees under {0}/wt and {0}/.bridle/state",
            repo.display()
        ),
    ];
    out.extend(machine.iter().cloned());
    for line in project {
        let head = line.trim_start().to_lowercase();
        if AUTO_MODE_PROJECT_PREFIXES
            .iter()
            .any(|p| head.starts_with(p))
        {
            out.push(line.clone());
        } else {
            tracing::warn!(line = %line, "ignoring a project [auto_mode] line that isn't Sensitive:/Prod host:");
        }
    }
    out
}

/// [`auto_mode_environment`] with the machine lines from `<home>/config.toml` and the project
/// lines from `<repo>/.bridle/config.toml`.
pub fn auto_mode_environment_for(home: &Path, repo: &Path) -> Vec<String> {
    auto_mode_environment(
        &auto_mode_lines(&home.join("config.toml")),
        &auto_mode_lines(&repo.join(".bridle/config.toml")),
        repo,
    )
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
    /// What a worker runs as its own gate (`{{commands.check_worker}}`); `None` means the same
    /// as `check`. A project may bind a lighter gate here (e.g. `just check-affected`) while
    /// the manager and orchestrator keep the full `check`.
    pub check_worker: Option<String>,
}

impl CommandsConfig {
    pub fn worker_check(&self) -> &str {
        self.check_worker.as_deref().unwrap_or(&self.check)
    }
}

/// `[branches]`: the project's branch pattern (docs/design/agent-host/operating-model.md,
/// "Branch pattern"). Exactly two shapes, per the human's own KISS ask (ticket br-29f9):
/// trunk (`release` unset — work merges into `integration`, releases are tags on it, bridle's
/// own pattern) or dev+release (`release` set — work merges into `integration`, and `release`
/// only moves when `integration` is merged into it for a release, done by whoever cuts
/// releases, not by workers or the manager's normal merge). A project trial
/// (docs/tickets/open/trial-adoption-*.md, 63rv) sets `integration` to its trial branch
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

/// `[integration]`: how `bridle land` verifies a merge before moving the integration branch.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntegrationConfig {
    /// Shell command run in the integration worktree; `None` skips the check.
    pub check: Option<String>,
    /// Shell command run in the integration worktree, in the background, after each land, to
    /// keep its `target/` fresh for `[worktrees] warm_target`; `None` runs nothing.
    pub warm_build: Option<String>,
    /// Globs of paths that can't affect the check. A landing whose integration branch gained only
    /// commits touching these during the check merges them in and lands without re-checking.
    pub check_skip_paths: Vec<String>,
}

/// `[ci]`: opt-in watching of the integration branch's GitHub Actions runs (`crate::ci`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CiConfig {
    pub github: bool,
}

/// `[messages]`: who may answer for the human.
#[derive(Debug, Clone, PartialEq)]
pub struct MessagesConfig {
    /// A reply from one of these principals to a message addressed to the human closes it
    /// (`AgentManager::send`). Nothing else can.
    pub answer_for_human: Vec<String>,
}

impl Default for MessagesConfig {
    fn default() -> Self {
        MessagesConfig {
            answer_for_human: vec!["external:orchestrator".to_string()],
        }
    }
}

/// `[ports]`: what `bridle port alloc` may hand out (docs/design/worktrees-and-ports.md).
#[derive(Debug, Clone, PartialEq)]
pub struct PortsConfig {
    /// Inclusive.
    pub range: (u16, u16),
    /// The human's own ports: never allocated, even when they're free right now.
    pub reserved: Vec<u16>,
}

impl Default for PortsConfig {
    fn default() -> Self {
        PortsConfig {
            range: (4000, 4999),
            reserved: Vec::new(),
        }
    }
}

/// `[review]`: documents under review (docs/design/agent-host/daemon.md, Document review).
#[derive(Debug, Clone)]
pub struct ReviewConfig {
    /// How long a document's pending comments must be still before its agent is woken.
    pub quiet: Duration,
    /// Document agents running at once; a further document waits.
    pub max_agents: u32,
    /// An idle document agent stops after this long.
    pub idle: Duration,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            quiet: Duration::from_secs(7 * 60),
            max_agents: 3,
            idle: Duration::from_secs(4 * 3600),
        }
    }
}

/// `[disk]`: the periodic disk usage check (`crate::disk`).
#[derive(Debug, Clone, PartialEq)]
pub struct DiskConfig {
    /// Zero turns the check off.
    pub check_interval: Duration,
    /// Free space under this many GiB on the workspace volume messages the human.
    pub min_free_gb: u64,
}

impl Default for DiskConfig {
    fn default() -> Self {
        DiskConfig {
            check_interval: crate::disk::DEFAULT_INTERVAL,
            min_free_gb: crate::disk::DEFAULT_MIN_FREE_GB,
        }
    }
}

/// `[machine]`: the load watch (`crate::load`).
#[derive(Debug, Clone, PartialEq)]
pub struct MachineConfig {
    /// Zero turns the watch off.
    pub check_interval: Duration,
    /// 1-minute load average per core above which new spawns are held; zero or less never holds.
    pub load_per_core: f64,
}

impl Default for MachineConfig {
    fn default() -> Self {
        MachineConfig {
            check_interval: crate::load::DEFAULT_INTERVAL,
            load_per_core: crate::load::DEFAULT_LOAD_PER_CORE,
        }
    }
}

/// `[orchestrator]`: the supervisor that keeps the human's interactive orchestrator running
/// (docs/design/agent-host/orchestrator-supervision.md, sections 2 to 4).
#[derive(Debug, Clone, PartialEq)]
pub struct OrchestratorConfig {
    /// Off by default: the supervisor doesn't run.
    pub enabled: bool,
    /// The command typed into the pane to relaunch, verbatim. None: `bridle session orchestrator
    /// --project <project>`.
    pub launcher: Option<String>,
    /// Wait before relaunch 2, 3, ...; the first relaunch goes at once. After them all fail
    /// the supervisor gives up.
    pub relaunch_backoff: Vec<Duration>,
    /// A session that stays up this long resets the relaunch count.
    pub stable_after: Duration,
    /// No `wait-for-wake` connected for this long while the session is up is an incident.
    pub waiter_grace: Duration,
    /// Context tokens at which the orchestrator is told the context size, asked to plan a
    /// handover, and told to hand over now (which starts the deadline).
    pub note_tokens: u64,
    pub plan_tokens: u64,
    pub handover_tokens: u64,
    /// After the "hand over now" or uptime message, the session is stopped this long after.
    pub handover_deadline: Duration,
    /// A session this old is asked to plan a handover; the deadline follows.
    pub max_uptime: Duration,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        OrchestratorConfig {
            enabled: false,
            launcher: None,
            relaunch_backoff: vec![
                Duration::from_secs(30),
                Duration::from_secs(2 * 60),
                Duration::from_secs(10 * 60),
            ],
            stable_after: Duration::from_secs(10 * 60),
            waiter_grace: Duration::from_secs(15 * 60),
            note_tokens: 150_000,
            plan_tokens: 180_000,
            handover_tokens: 200_000,
            handover_deadline: Duration::from_secs(30 * 60),
            max_uptime: Duration::from_secs(12 * 60 * 60),
        }
    }
}

impl OrchestratorConfig {
    fn merge(mut self, raw: RawOrchestrator) -> Result<Self, ConfigError> {
        if let Some(v) = raw.enabled {
            self.enabled = v;
        }
        if let Some(v) = raw.launcher {
            if v.trim().is_empty() {
                return Err(ConfigError::BadOrchestrator("launcher is empty".into()));
            }
            // The old default: a repo-relative script path the pane's cwd may not resolve.
            // Any config value other than the old script path is kept as an override.
            self.launcher = (v != "scripts/claude-orchestrator").then_some(v);
        }
        if let Some(v) = raw.relaunch_backoff {
            if v.is_empty() {
                return Err(ConfigError::BadOrchestrator(
                    "relaunch_backoff needs at least one duration".into(),
                ));
            }
            self.relaunch_backoff = v
                .iter()
                .map(|s| parse_duration(s))
                .collect::<Result<_, _>>()?;
        }
        if let Some(s) = raw.stable_after {
            self.stable_after = parse_duration(&s)?;
        }
        if let Some(s) = raw.waiter_grace {
            self.waiter_grace = parse_duration(&s)?;
        }
        if let Some(v) = raw.note_tokens {
            self.note_tokens = v.0;
        }
        if let Some(v) = raw.plan_tokens {
            self.plan_tokens = v.0;
        }
        if let Some(v) = raw.handover_tokens {
            self.handover_tokens = v.0;
        }
        if let Some(s) = raw.handover_deadline {
            self.handover_deadline = parse_duration(&s)?;
        }
        if let Some(s) = raw.max_uptime {
            self.max_uptime = parse_duration(&s)?;
        }
        if self.note_tokens == 0
            || self.note_tokens > self.plan_tokens
            || self.plan_tokens > self.handover_tokens
        {
            return Err(ConfigError::BadOrchestrator(
                "need 0 < note_tokens <= plan_tokens <= handover_tokens".into(),
            ));
        }
        if self.handover_deadline.is_zero() || self.max_uptime.is_zero() {
            return Err(ConfigError::BadOrchestrator(
                "handover_deadline and max_uptime must be positive".into(),
            ));
        }
        Ok(self)
    }
}

/// The context steps of an interactive session (docs/design/agent-host/orchestrator-supervision.md,
/// "Every interactive session"): warn, plan a handover, the normal ceiling, the hard limit.
pub type SessionSteps = [u64; 4];

const DEFAULT_SESSION_STEPS: SessionSteps = [150_000, 200_000, 250_000, 300_000];

/// `[sessions] warn` and the per-role `[sessions.advisor] warn` / `[sessions.aide] warn`.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionsConfig {
    pub warn: SessionSteps,
    pub advisor: Option<SessionSteps>,
    pub aide: Option<SessionSteps>,
}

impl Default for SessionsConfig {
    fn default() -> Self {
        SessionsConfig {
            warn: DEFAULT_SESSION_STEPS,
            advisor: None,
            aide: None,
        }
    }
}

impl SessionsConfig {
    /// The steps for a session identity (`advisor`, `advisor/<name>`, `aide`).
    pub fn steps_for(&self, identity: &str) -> SessionSteps {
        let role = identity.split('/').next().unwrap_or(identity);
        match role {
            "advisor" => self.advisor,
            "aide" => self.aide,
            _ => None,
        }
        .unwrap_or(self.warn)
    }

    fn merge(mut self, raw: RawSessions) -> Result<Self, ConfigError> {
        if let Some(w) = raw.warn {
            self.warn = session_steps(&w)?;
        }
        if let Some(w) = raw.advisor.and_then(|r| r.warn) {
            self.advisor = Some(session_steps(&w)?);
        }
        if let Some(w) = raw.aide.and_then(|r| r.warn) {
            self.aide = Some(session_steps(&w)?);
        }
        Ok(self)
    }
}

fn session_steps(raw: &[TokenCount]) -> Result<SessionSteps, ConfigError> {
    let steps: SessionSteps = raw
        .iter()
        .map(|t| t.0)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| ConfigError::BadSessions("warn needs exactly four token counts".into()))?;
    if steps[0] == 0 || steps.windows(2).any(|w| w[0] >= w[1]) {
        return Err(ConfigError::BadSessions(
            "warn must be four increasing, nonzero token counts".into(),
        ));
    }
    Ok(steps)
}

#[derive(Debug, Default, Deserialize)]
struct RawSessions {
    #[serde(default)]
    warn: Option<Vec<TokenCount>>,
    #[serde(default)]
    advisor: Option<RawSessionRole>,
    #[serde(default)]
    aide: Option<RawSessionRole>,
}

#[derive(Debug, Default, Deserialize)]
struct RawSessionRole {
    #[serde(default)]
    warn: Option<Vec<TokenCount>>,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        CommandsConfig {
            check: "just check".to_string(),
            check_worker: None,
        }
    }
}

impl CommandsConfig {
    fn merge(mut self, raw: RawCommands) -> Self {
        if let Some(v) = raw.check {
            self.check = v;
        }
        if let Some(v) = raw.check_worker {
            self.check_worker = Some(v);
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: SocketAddr,
    /// `[daemon] listen` was set, so it beats the port from `[projects]` (k7mw).
    pub listen_set: bool,
    pub stall_after: Duration,
    /// How long a claimed task's lease survives without the claiming
    /// agent's own activity (`last_event_at`/`turn_started_at`) before it's
    /// released back to `planned` (docs/design/storage.md, claims).
    pub claim_lease_after: Duration,
    /// `[daemon] self_upgrade`: when a newer green commit is on the integration
    /// branch, run what `bridle restart --upgrade` runs (docs/design/agent-host/daemon.md, Upgrade).
    pub self_upgrade: bool,
    pub stop_grace: Duration,
    pub roles: BTreeMap<String, Role>,
    pub budget: BudgetConfig,
    pub models: ModelsConfig,
    pub context: ContextConfig,
    pub commands: CommandsConfig,
    /// `[worktrees] warm_target`: clone the clone's `target/` into each new worktree
    /// (macOS only; ticket b7cz). On by default.
    pub warm_target: bool,
    /// `[worktrees] setup`: shell command run in each new worker worktree (e.g. an install step).
    pub setup: Option<String>,
    /// `[worktrees] setup_timeout_secs` (default 600).
    pub setup_timeout: Duration,
    /// `[worktrees] copy`: repo-relative files copied into each new worktree before setup.
    pub copy: Vec<String>,
    /// `[worktrees] layout`/`root`: where new worker worktrees live.
    pub worktree_layout: WorktreeLayout,
    /// `[worktrees.pair.<name>]`: sibling repos of a paired layout.
    pub worktree_pairs: BTreeMap<String, PairMember>,
    pub branches: BranchesConfig,
    pub ci: CiConfig,
    pub disk: DiskConfig,
    pub machine: MachineConfig,
    pub review: ReviewConfig,
    /// `[state] push`: push `bridle/state` to origin after a flush that committed. On by
    /// default: set to false to opt-out (rule existing-projects, human-approved 2026-09-29).
    pub state_push: bool,
    /// `[migrations] auto`: `bridle serve` applies the project's pending migrations at start-up
    /// (docs/design/migrations.md). On by default.
    pub migrations_auto: bool,
    pub orchestrator: OrchestratorConfig,
    pub sessions: SessionsConfig,
    pub ports: PortsConfig,
    pub integration: IntegrationConfig,
    pub messages: MessagesConfig,
    /// The prefix new task ids get (storage.md: `<prefix>-<4 hex chars>`,
    /// e.g. `tw-7fa2`). `None` means derive one from the project name
    /// ([`default_task_prefix`]).
    pub task_prefix: Option<String>,
    /// `[tasks] settle`: how long a task waits after creation or a human
    /// edit/comment before anyone can start it (ny9u). Zero turns it off.
    pub tasks_settle: Duration,
    /// Set when `[tasks] settle` didn't parse and `tasks_settle` fell back to
    /// the default; `bridle doctor` reports it. Never fatal: a typo here must
    /// not stop the daemon starting.
    pub tasks_settle_problem: Option<String>,
    /// `[tasks] open_stale`: an `open` task nobody planned for this long goes back to `pending`
    /// (xz4f). Zero turns it off.
    pub tasks_open_stale: Duration,
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
        roles.insert("researcher".to_string(), Role::researcher_default());
        roles.insert("prototyper".to_string(), Role::worker_default());
        roles.insert("document-reviewer".to_string(), Role::worker_default());
        Config {
            listen: "127.0.0.1:0".parse().expect("valid default listen addr"),
            listen_set: false,
            stall_after: Duration::from_secs(10 * 60),
            claim_lease_after: Duration::from_secs(10 * 60),
            self_upgrade: false,
            stop_grace: Duration::from_secs(30),
            roles,
            budget: BudgetConfig::default(),
            models: ModelsConfig::default(),
            context: ContextConfig::default(),
            commands: CommandsConfig::default(),
            warm_target: true,
            setup: None,
            setup_timeout: Duration::from_secs(10 * 60),
            copy: Vec::new(),
            worktree_layout: WorktreeLayout::Default,
            worktree_pairs: BTreeMap::new(),
            branches: BranchesConfig::default(),
            ci: CiConfig::default(),
            disk: DiskConfig::default(),
            machine: MachineConfig::default(),
            review: ReviewConfig::default(),
            state_push: true,
            migrations_auto: true,
            orchestrator: OrchestratorConfig::default(),
            sessions: SessionsConfig::default(),
            ports: PortsConfig::default(),
            integration: IntegrationConfig::default(),
            messages: MessagesConfig::default(),
            task_prefix: None,
            tasks_settle: DEFAULT_SETTLE,
            tasks_settle_problem: None,
            tasks_open_stale: DEFAULT_OPEN_STALE,
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
                    bridle_api::config_warn::parse(&text, &path.display().to_string()).map_err(
                        |source| ConfigError::Parse {
                            path: path.clone(),
                            source: Box::new(source),
                        },
                    )?;
                BudgetConfig::default().merge(raw.budget.unwrap_or_default())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BudgetConfig::default()),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// The machine's `workflow` override from `<home>/config.toml`, if any. A machine
    /// beats the project because the project's path is written for one machine.
    fn load_machine_workflow(home_override: Option<&Path>) -> Result<Option<String>, ConfigError> {
        let home = home_override
            .map(Path::to_path_buf)
            .unwrap_or_else(bridle_api::discovery::bridle_home);
        let path = home.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let raw: RawConfig =
                    bridle_api::config_warn::parse(&text, &path.display().to_string()).map_err(
                        |source| ConfigError::Parse {
                            path: path.clone(),
                            source: Box::new(source),
                        },
                    )?;
                Ok(raw.workflow)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// The machine's `workflow_url` from `<home>/config.toml`, if any.
    pub fn machine_workflow_url(
        home_override: Option<&Path>,
    ) -> Result<Option<String>, ConfigError> {
        let home = home_override
            .map(Path::to_path_buf)
            .unwrap_or_else(bridle_api::discovery::bridle_home);
        let path = home.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let raw: RawConfig =
                    bridle_api::config_warn::parse(&text, &path.display().to_string()).map_err(
                        |source| ConfigError::Parse {
                            path: path.clone(),
                            source: Box::new(source),
                        },
                    )?;
                Ok(raw.workflow_url)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// The workflow checkout's directory: `workflow` with `~`/`$VAR` expanded, relative
    /// paths taken against `repo`. `None` if unset; a git url is an error (nothing clones
    /// those yet). A directory that isn't there is an error: silently dropping the base layer
    /// leaves agents without rules and `bridle sync` without skills.
    pub fn workflow_root(&self, repo: &Path) -> Result<Option<PathBuf>, ConfigError> {
        let Some(w) = self.workflow.as_deref() else {
            return Ok(None);
        };
        if w.contains("://") || w.starts_with("git@") {
            return Err(ConfigError::WorkflowGitUrl { url: w.to_string() });
        }
        let root = repo.join(w);
        match std::fs::read_dir(&root) {
            Ok(_) => Ok(Some(root)),
            Err(e) => Err(ConfigError::WorkflowDirMissing {
                path: root,
                reason: e.to_string(),
            }),
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
        let mut config = match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse_with_budget(&text, machine_budget, &path)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut config = Config {
                    budget: machine_budget,
                    ..Config::default()
                };
                apply_branches(&mut config);
                config
            }
            Err(source) => return Err(ConfigError::Read { path, source }),
        };
        if let Some(w) = Self::load_machine_workflow(home_override)? {
            config.workflow = Some(w);
        }
        // A project with no `workflow` uses the copy `bridle init` vendored, if there is one.
        if config.workflow.is_none() && repo.join(VENDORED_WORKFLOW).join("base").is_dir() {
            config.workflow = Some(VENDORED_WORKFLOW.to_string());
        }
        config.workflow = config.workflow.as_deref().map(expand_path).transpose()?;
        config.default_role_prompts(repo);
        Ok(config)
    }

    /// A role with no `system_prompt` uses `<workflow>/base/roles/<role>.md` when the
    /// project sets `workflow` and that file exists, so onboarding onto the base layer
    /// needs no per-role line (docs/design/agent-host/roles-and-config.md). An explicit
    /// `system_prompt` always wins; no file means no prompt, as before.
    fn default_role_prompts(&mut self, repo: &Path) {
        let Some(workflow) = self.workflow.as_deref() else {
            return;
        };
        for (name, role) in &mut self.roles {
            if role.system_prompt.is_some() {
                continue;
            }
            // The researcher is a worker with web tools: it reuses the worker's prompt.
            let candidates = [name.as_str(), "worker"];
            let candidates = if name == "researcher" {
                &candidates[..]
            } else {
                &candidates[..1]
            };
            for c in candidates {
                let rel = Path::new(workflow)
                    .join("base/roles")
                    .join(format!("{c}.md"));
                if repo.join(&rel).is_file() {
                    role.system_prompt = Some(rel);
                    break;
                }
            }
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
        let raw: RawConfig = bridle_api::config_warn::parse(text, &path.display().to_string())
            .map_err(|source| ConfigError::Parse {
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
                config.listen_set = true;
            }
            if let Some(s) = d.stall_after {
                config.stall_after = parse_duration(&s)?;
            }
            if let Some(s) = d.claim_lease_after {
                config.claim_lease_after = parse_duration(&s)?;
            }
            if let Some(v) = d.self_upgrade {
                config.self_upgrade = v;
            }
            if let Some(s) = d.stop_grace {
                config.stop_grace = parse_duration(&s)?;
            }
        }

        if let Some(raw_branches) = raw.branches {
            config.branches = config.branches.merge(raw_branches);
        }
        if let Some(v) = raw.migrations.and_then(|m| m.auto) {
            config.migrations_auto = v;
        }
        if let Some(github) = raw.ci.and_then(|c| c.github) {
            config.ci.github = github;
        }

        if let Some(v) = raw.messages.and_then(|m| m.answer_for_human) {
            config.messages.answer_for_human = v;
        }

        if let Some(i) = raw.integration {
            if let Some(c) = i.check {
                config.integration.check = Some(c);
            }
            if let Some(c) = i.warm_build {
                config.integration.warm_build = Some(c);
            }
            if let Some(v) = i.check_skip_paths {
                config.integration.check_skip_paths = v;
            }
        }

        if let Some(p) = raw.ports {
            if let Some([lo, hi]) = p.range {
                if lo > hi {
                    return Err(ConfigError::BadPortRange(lo, hi));
                }
                config.ports.range = (lo, hi);
            }
            config.ports.reserved = p.reserved;
        }

        if let Some(o) = raw.orchestrator {
            config.orchestrator = config.orchestrator.merge(o)?;
        }
        if let Some(r) = raw.sessions {
            config.sessions = config.sessions.merge(r)?;
        }

        if let Some(v) = raw.state.and_then(|s| s.push) {
            config.state_push = v;
        }

        if let Some(d) = raw.disk {
            if let Some(s) = d.check_interval {
                config.disk.check_interval = parse_duration(&s)?;
            }
            if let Some(v) = d.min_free_gb {
                config.disk.min_free_gb = v;
            }
        }

        if let Some(m) = raw.machine {
            if let Some(s) = m.check_interval {
                config.machine.check_interval = parse_duration(&s)?;
            }
            if let Some(v) = m.load_per_core {
                config.machine.load_per_core = v;
            }
        }

        if let Some(r) = raw.review {
            if let Some(m) = r.quiet_minutes {
                config.review.quiet = Duration::from_secs(m * 60);
            }
            if let Some(n) = r.max_agents {
                config.review.max_agents = n;
            }
            if let Some(h) = r.idle_hours {
                config.review.idle = Duration::from_secs(h * 3600);
            }
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
            let name = canonical_role(&name);
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

        if let Some(w) = raw.worktrees {
            if let Some(v) = w.warm_target {
                config.warm_target = v;
            }
            config.setup = w.setup.filter(|c| !c.trim().is_empty());
            if let Some(secs) = w.setup_timeout_secs {
                config.setup_timeout = Duration::from_secs(secs);
            }
            if let Some(bad) = w.copy.iter().find(|p| !is_safe_relative(p)) {
                return Err(ConfigError::BadCopyPath(bad.clone()));
            }
            config.copy = w.copy;
            config.worktree_layout = WorktreeLayout::parse(w.layout.as_deref(), w.root)?;
            config.worktree_pairs = parse_pairs(w.pair, &config.worktree_layout)?;
        }

        if let Some(t) = raw.tasks {
            config.task_prefix = t.prefix;
            if let Some(v) = t.settle {
                match parse_settle(&v) {
                    Ok(d) => config.tasks_settle = d,
                    Err(problem) => {
                        tracing::warn!("{problem}; using 5m (`bridle doctor` reports this)");
                        config.tasks_settle_problem = Some(problem);
                    }
                }
            }
            if let Some(v) = t.open_stale {
                match parse_open_stale(&v) {
                    Ok(d) => config.tasks_open_stale = d,
                    Err(problem) => tracing::warn!("{problem}; using 4h"),
                }
            }
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

/// `HH:MM`, or `HH:MM+1d` for an end on the day after the start day. The suffix doesn't change
/// the parsed time: an end before its start already means the next day when matching, and the
/// suffix is what `overnight_problem` asks for.
fn parse_time_of_day(s: &str, period_name: &str) -> Result<NaiveTime, ConfigError> {
    let s = s.strip_suffix("+1d").unwrap_or(s);
    NaiveTime::parse_from_str(s, "%H:%M").map_err(|_| ConfigError::BadSchedule {
        name: period_name.to_string(),
        reason: format!("invalid time {s:?}: expected HH:MM"),
    })
}

/// What is wrong with a period's `start`/`end` as written, with the fix, or `None`. An end
/// before its start must say `+1d` (`00:00` is the midnight ending the start day, so it needs
/// none); `+1d` on an end that isn't before the start can't mean the next day's earlier time.
/// Unparseable times are left to `parse_time_of_day`. Called by `bridle doctor` as an error and
/// by the loaders as a warning only: a surprising block must never stop the daemon.
fn overnight_problem(name: &str, start: &str, end: &str) -> Option<String> {
    let plus = end.ends_with("+1d");
    let end_t = parse_time_of_day(end, name).ok()?;
    let start_t = parse_time_of_day(start, name).ok()?;
    let end_s = end.strip_suffix("+1d").unwrap_or(end);
    if plus && end_t >= start_t {
        Some(format!(
            "{name}: end {end_s}+1d is not before start {start}; drop the \"+1d\" (or make the end earlier than the start)"
        ))
    } else if !plus && end_t < start_t && end_t != NaiveTime::MIN {
        Some(format!(
            "{name}: end {end} is before start {start}; write \"{end}+1d\""
        ))
    } else {
        None
    }
}

fn warn_overnight(name: &str, start: &str, end: &str) {
    if let Some(problem) = overnight_problem(name, start, end) {
        tracing::warn!("{problem} (read as the next day; `bridle doctor` reports this)");
    }
}

/// Every overnight problem in `<home>/config.toml`'s `[[focus]]` and `[[budget.schedule]]`
/// blocks, for `bridle doctor`. Empty when the file is absent.
pub fn machine_config_problems(home: &Path) -> Result<Vec<String>, ConfigError> {
    let path = home.join("config.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(ConfigError::Read { path, source }),
    };
    let raw: RawConfig = bridle_api::config_warn::parse(&text, &path.display().to_string())
        .map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source: Box::new(source),
        })?;
    let mut out = Vec::new();
    for f in raw.focus.iter().flatten() {
        out.extend(overnight_problem(&f.name, &f.start, &f.end));
    }
    for p in raw.budget.iter().flat_map(|b| b.schedule.iter().flatten()) {
        if let (Some(s), Some(e)) = (&p.start, &p.end) {
            out.extend(overnight_problem(&p.name, s, e));
        }
    }
    Ok(out)
}

/// The settle period when `[tasks] settle` is unset or invalid.
const DEFAULT_SETTLE: Duration = Duration::from_secs(5 * 60);

/// `[tasks] settle`: a duration string ("5m", "0s") or the integer 0.
fn parse_settle(v: &toml::Value) -> Result<Duration, String> {
    match v {
        toml::Value::Integer(0) => Ok(Duration::ZERO),
        toml::Value::String(s) if s.trim() == "0" => Ok(Duration::ZERO),
        toml::Value::String(s) => parse_duration(s)
            .map_err(|_| format!("[tasks] settle = {s:?} is not a duration like \"5m\" (or 0)")),
        other => Err(format!(
            "[tasks] settle = {other} is not a duration like \"5m\" (or 0)"
        )),
    }
}

/// The `open_stale` period when `[tasks] open_stale` is unset or invalid.
const DEFAULT_OPEN_STALE: Duration = Duration::from_secs(4 * 3600);

/// `[tasks] open_stale`: a duration string ("4h", "0s") or the integer 0.
fn parse_open_stale(v: &toml::Value) -> Result<Duration, String> {
    match v {
        toml::Value::Integer(0) => Ok(Duration::ZERO),
        toml::Value::String(s) if s.trim() == "0" => Ok(Duration::ZERO),
        toml::Value::String(s) => parse_duration(s).map_err(|_| {
            format!("[tasks] open_stale = {s:?} is not a duration like \"4h\" (or 0)")
        }),
        other => Err(format!(
            "[tasks] open_stale = {other} is not a duration like \"4h\" (or 0)"
        )),
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

/// Parses token counts with optional suffix: "150000", "150k", "1.5M" (case-insensitive).
/// `k` = 1,000, `M` = 1,000,000.
fn parse_token_count(s: &str) -> Result<u64, ConfigError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ConfigError::BadOrchestrator(
            "token count must not be empty".into(),
        ));
    }
    let (num_part, unit) = if let Some(c) = s.chars().last() {
        match c.to_ascii_lowercase() {
            'k' => (&s[..s.len() - 1], 1_000_u64),
            'm' => (&s[..s.len() - 1], 1_000_000_u64),
            _ => (s, 1_u64),
        }
    } else {
        (s, 1_u64)
    };
    let bad = || ConfigError::BadOrchestrator(format!("invalid token count {s:?}"));
    // Plain digits with an optional fraction: no sign, exponent, "inf" or "nan", which
    // f64's parser would let through.
    let (whole, frac) = num_part.split_once('.').unwrap_or((num_part, ""));
    let digits = |p: &str| p.bytes().all(|b| b.is_ascii_digit());
    if (whole.is_empty() && frac.is_empty()) || !digits(whole) || !digits(frac) {
        return Err(bad());
    }
    let n: f64 = num_part.parse().map_err(|_| bad())?;
    let result = n * unit as f64;
    if result >= u64::MAX as f64 {
        return Err(bad());
    }
    Ok(result.round() as u64)
}

#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    /// `[machines]` and `[projects]` (k7mw) are read through
    /// `bridle_api::machines::MachineMap`; here they only have to parse.
    #[serde(default)]
    #[allow(dead_code)]
    machines: Option<BTreeMap<String, String>>,
    #[serde(default)]
    #[allow(dead_code)]
    projects: Option<BTreeMap<String, bridle_api::machines::ProjectPlace>>,
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
    worktrees: Option<RawWorktrees>,
    #[serde(default)]
    branches: Option<RawBranches>,
    #[serde(default)]
    ci: Option<RawCi>,
    #[serde(default)]
    disk: Option<RawDisk>,
    #[serde(default)]
    review: Option<RawReview>,
    #[serde(default)]
    state: Option<RawState>,
    #[serde(default)]
    migrations: Option<RawMigrations>,
    #[serde(default)]
    orchestrator: Option<RawOrchestrator>,
    #[serde(default)]
    sessions: Option<RawSessions>,
    #[serde(default)]
    ports: Option<RawPorts>,
    #[serde(default)]
    integration: Option<RawIntegration>,
    #[serde(default)]
    messages: Option<RawMessages>,
    #[serde(default)]
    tasks: Option<RawTasks>,
    #[serde(default)]
    workflow: Option<String>,
    /// Machine scope only: the git url `bridle init` / `bridle workflow update` fetch the
    /// base workflow from when there's no local clone.
    #[serde(default)]
    workflow_url: Option<String>,
    #[serde(default)]
    packs: Option<Vec<String>>,
    #[serde(default)]
    components: Option<BTreeMap<String, Component>>,
    /// `[mail]` belongs to `bridle mail run` (the `bridle-mail` crate parses it); the daemon
    /// only has to accept it.
    #[serde(default)]
    #[allow(dead_code)]
    mail: Option<toml::Value>,
    /// `[gateway]` and `[interactions]` belong to `bridle gateway` (the `bridle-gateway` crate
    /// parses them); the daemon only has to accept them, or adding either stops every daemon
    /// starting (br-jmpf).
    #[serde(default)]
    #[allow(dead_code)]
    gateway: Option<toml::Value>,
    #[serde(default)]
    #[allow(dead_code)]
    interactions: Option<toml::Value>,
    /// Machine scope only; a project's config may carry it but nothing reads it there.
    #[serde(default)]
    machine: Option<RawMachine>,
    /// Read by `bridle advisor start` from the machine config only (`advisor_pane`).
    #[serde(default)]
    tmux: Option<RawTmux>,
    /// Machine scope: the `[[focus]]` periods (ticket cvaq).
    #[serde(default)]
    focus: Option<Vec<RawFocusPeriod>>,
    /// Machine scope: minutes before a focus override file takes effect.
    #[serde(default)]
    focus_override_delay_minutes: Option<u32>,
    /// Project scope: `focus_hours = false` opts the project out of focus hours.
    #[serde(default)]
    focus_hours: Option<bool>,
    /// `[auto_mode]`: lines for Claude Code's auto-mode classifier in `bridle session`'s
    /// `--settings` (br-fc9a). Machine file: trust lines. Project file: tightening lines only.
    #[serde(default)]
    auto_mode: Option<RawAutoMode>,
}

#[derive(Debug, Default, Deserialize)]
struct RawAutoMode {
    #[serde(default)]
    environment: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawTmux {
    #[serde(default)]
    advisor_pane: Option<String>,
}

/// Where `bridle advisor start` puts the advisor: `[tmux] advisor_pane` of `<home>/config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdvisorPane {
    /// Split the orchestrator's window when its pane is found, else a new window.
    #[default]
    Split,
    Window,
}

pub fn advisor_pane(home: &Path) -> Result<AdvisorPane, ConfigError> {
    let path = home.join("config.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(AdvisorPane::Split),
        Err(source) => return Err(ConfigError::Read { path, source }),
    };
    let raw: RawConfig = bridle_api::config_warn::parse(&text, &path.display().to_string())
        .map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source: Box::new(source),
        })?;
    match raw.tmux.unwrap_or_default().advisor_pane.as_deref() {
        None | Some("split") => Ok(AdvisorPane::Split),
        Some("window") => Ok(AdvisorPane::Window),
        Some(other) => Err(ConfigError::BadTmux(format!(
            "advisor_pane = \"{other}\" in {}: expected \"split\" or \"window\"",
            path.display()
        ))),
    }
}

/// `[gateway] public_url` of one config file: absent file, section or key is `None`.
fn gateway_public_url(path: &Path) -> Result<Option<String>, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ConfigError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let raw: RawConfig = bridle_api::config_warn::parse(&text, &path.display().to_string())
        .map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })?;
    Ok(raw
        .gateway
        .as_ref()
        .and_then(|g| g.get("public_url"))
        .and_then(|v| v.as_str())
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty()))
}

/// The bridle UI's base URL for `bridle link`: `[gateway] public_url` of `<repo>/.bridle/config.toml`
/// over the same key in `<home>/config.toml`. `None` when neither sets it.
pub fn ui_base_url(home: &Path, repo: Option<&Path>) -> Result<Option<String>, ConfigError> {
    if let Some(repo) = repo
        && let Some(url) = gateway_public_url(&repo.join(".bridle/config.toml"))?
    {
        return Ok(Some(url));
    }
    gateway_public_url(&home.join("config.toml"))
}

#[derive(Debug, Default, Deserialize)]
struct RawMachine {
    #[serde(default)]
    tools_only: Vec<String>,
    /// This machine's name; read by `bridle_api::machines::MachineMap`, not the daemon.
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    /// The load watch (`crate::load`).
    #[serde(default)]
    check_interval: Option<String>,
    #[serde(default)]
    load_per_core: Option<f64>,
}

/// Whether `repo` is listed in `[machine] tools_only` of `<home>/config.toml` (hw6c): a clone
/// kept for its tools, where the daemon must not serve and nothing should commit. Entries
/// expand `~`/`$VAR` and are compared canonicalized, so symlinks and trailing slashes match.
/// A missing config file or section means no.
pub fn is_tools_only(repo: &Path, home: &Path) -> Result<bool, ConfigError> {
    let path = home.join("config.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(source) => return Err(ConfigError::Read { path, source }),
    };
    let raw: RawConfig = bridle_api::config_warn::parse(&text, &path.display().to_string())
        .map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source: Box::new(source),
        })?;
    let repo = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    for entry in raw.machine.unwrap_or_default().tools_only {
        let p = PathBuf::from(expand_path(&entry)?);
        if p.canonicalize().unwrap_or(p) == repo {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug, Default, Deserialize)]
struct RawContext {
    #[serde(default)]
    wind_down_at: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    wind_down_grace: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCommands {
    #[serde(default)]
    check: Option<String>,
    #[serde(default)]
    check_worker: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawWorktrees {
    #[serde(default)]
    warm_target: Option<bool>,
    #[serde(default)]
    setup: Option<String>,
    #[serde(default)]
    setup_timeout_secs: Option<u64>,
    #[serde(default)]
    copy: Vec<String>,
    #[serde(default)]
    layout: Option<String>,
    #[serde(default)]
    root: Option<String>,
    #[serde(default)]
    pair: BTreeMap<String, RawPair>,
}

#[derive(Debug, Deserialize)]
struct RawPair {
    path: String,
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawBranches {
    #[serde(default)]
    integration: Option<String>,
    #[serde(default)]
    release: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawMigrations {
    #[serde(default)]
    auto: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCi {
    #[serde(default)]
    github: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct RawMessages {
    #[serde(default)]
    answer_for_human: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
struct RawIntegration {
    #[serde(default)]
    check: Option<String>,
    #[serde(default)]
    warm_build: Option<String>,
    #[serde(default)]
    check_skip_paths: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
struct RawPorts {
    #[serde(default)]
    range: Option<[u16; 2]>,
    #[serde(default)]
    reserved: Vec<u16>,
}

#[derive(Debug, Default, Deserialize)]
struct RawState {
    #[serde(default)]
    push: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct RawDisk {
    #[serde(default)]
    check_interval: Option<String>,
    #[serde(default)]
    min_free_gb: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawReview {
    #[serde(default)]
    quiet_minutes: Option<u64>,
    #[serde(default)]
    max_agents: Option<u32>,
    #[serde(default)]
    idle_hours: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct TokenCount(u64);

impl<'de> Deserialize<'de> for TokenCount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct TokenCountVisitor;

        impl<'de> Visitor<'de> for TokenCountVisitor {
            type Value = TokenCount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer or a string like \"150k\" or \"1.5M\"")
            }

            fn visit_u64<E>(self, value: u64) -> Result<TokenCount, E>
            where
                E: de::Error,
            {
                Ok(TokenCount(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<TokenCount, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    return Err(E::custom("token count must be non-negative"));
                }
                Ok(TokenCount(value as u64))
            }

            fn visit_str<E>(self, value: &str) -> Result<TokenCount, E>
            where
                E: de::Error,
            {
                parse_token_count(value)
                    .map(TokenCount)
                    .map_err(|e| E::custom(e.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<TokenCount, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(TokenCountVisitor)
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawOrchestrator {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    launcher: Option<String>,
    #[serde(default)]
    relaunch_backoff: Option<Vec<String>>,
    #[serde(default)]
    stable_after: Option<String>,
    #[serde(default)]
    waiter_grace: Option<String>,
    #[serde(default)]
    note_tokens: Option<TokenCount>,
    #[serde(default)]
    plan_tokens: Option<TokenCount>,
    #[serde(default)]
    handover_tokens: Option<TokenCount>,
    #[serde(default)]
    handover_deadline: Option<String>,
    #[serde(default)]
    max_uptime: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawTasks {
    #[serde(default)]
    prefix: Option<String>,
    /// A duration like "5m", or the integer 0.
    #[serde(default)]
    settle: Option<toml::Value>,
    /// A duration like "4h", or the integer 0.
    #[serde(default)]
    open_stale: Option<toml::Value>,
}

#[derive(Debug, Default, Deserialize)]
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
struct RawSchedulePeriod {
    name: String,
    /// `days`, `start` and `end` are all given or all omitted (a preset).
    #[serde(default)]
    days: Option<RawDays>,
    #[serde(default)]
    start: Option<String>,
    #[serde(default)]
    end: Option<String>,
    hold_at: f64,
    wind_down_at: f64,
    stop_at: f64,
    #[serde(default)]
    max_workers: Option<u32>,
}

impl RawSchedulePeriod {
    fn into_period(self) -> Result<SchedulePeriod, ConfigError> {
        let (days, start, end) = match (self.days, self.start, self.end) {
            (Some(d), Some(s), Some(e)) => (d, Some(s), Some(e)),
            (None, None, None) => (RawDays::List(Vec::new()), None, None),
            _ => {
                return Err(ConfigError::BadSchedule {
                    name: self.name,
                    reason: "days, start and end must all be given or all omitted \
                             (a preset used only via `bridle budget override`)"
                        .into(),
                });
            }
        };
        let days = match days {
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
        if let (Some(s), Some(e)) = (&start, &end) {
            warn_overnight(&self.name, s, e);
        }
        let start = start
            .map(|t| parse_time_of_day(&t, &self.name))
            .transpose()?;
        let end = end.map(|t| parse_time_of_day(&t, &self.name)).transpose()?;
        Ok(SchedulePeriod {
            start,
            end,
            name: self.name,
            days,
            hold_at: self.hold_at,
            wind_down_at: self.wind_down_at,
            stop_at: self.stop_at,
            max_workers: self.max_workers,
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
struct RawDaemon {
    #[serde(default)]
    listen: Option<String>,
    #[serde(default)]
    stall_after: Option<String>,
    #[serde(default)]
    claim_lease_after: Option<String>,
    #[serde(default)]
    self_upgrade: Option<bool>,
    #[serde(default)]
    stop_grace: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
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
    tools: Option<Vec<String>>,
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
        "researcher" => Some(
            "\nYou are a researcher: a worker with WebSearch and WebFetch. Cite every source you rely on with its URL, and record each failed fetch or missing tool (what, URL, exact error) in the task thread and your summary instead of working around it silently.\n",
        ),
        "prototyper" => Some(
            "\nYou are a prototyper: build the prototype in your own worktree and branch, from the prompt's constraints only, and report back to whoever gave you the task.\n",
        ),
        "document-reviewer" => Some(
            "\nYou are a document-reviewer: answer the human's comments in the one document you were given, revise it when a comment asks, and commit each round.\n",
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
    rules: &str,
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
    // The project's own prototype conventions (where prototypes live) are appended to the
    // shared role; a missing file is the normal case. Other roles don't read this dir here.
    if matches!(role_name, "prototyper" | "document-reviewer")
        && let Ok(text) =
            std::fs::read_to_string(repo.join(format!(".bridle/roles/{role_name}.md")))
    {
        out.push('\n');
        out.push_str(&substitute_role_text(&text, branches, commands));
    }
    if !rules.is_empty() {
        out.push_str("\n## Workflow rules\n\n");
        out.push_str(rules);
    }
    out
}

impl Config {
    /// The layers' `hooks/<event>.json` (base, packs, project: the overlay
    /// `bridle sync` uses) for the spawn's `--settings`. Entries already in
    /// the project's committed `.claude/settings.json` (where `bridle sync`
    /// puts the same hooks) are dropped so they don't run twice. Never
    /// fails: a problem yields fewer hooks and a warning.
    pub fn layer_hooks(&self, repo: &Path) -> BTreeMap<String, serde_json::Value> {
        let root = match self.workflow_root(repo) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "workflow root not resolvable; no layer hooks");
                return BTreeMap::new();
            }
        };
        let layers = crate::rules::discover_layers(repo, root.as_deref(), &self.packs);
        let mut hooks = crate::sync::discover_hooks_lossy(&layers);
        let committed: Option<serde_json::Value> =
            std::fs::read_to_string(repo.join(".claude").join("settings.json"))
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok());
        hooks.retain(|event, value| {
            let existing = committed
                .as_ref()
                .and_then(|c| c["hooks"][event.as_str()].as_array());
            if let (Some(existing), Some(entries)) = (existing, value.as_array_mut()) {
                entries.retain(|e| !existing.contains(e));
            }
            value.as_array().is_some_and(|a| !a.is_empty())
        });
        hooks
    }

    /// The role's resolved workflow rules (L1 base, packs, project; no components) as
    /// `bridle prime <role>` prints them, for [`stable_system_prompt`]. Never fails: rules
    /// that can't be resolved (missing workflow dir or pack, a bad rule file) log a warning
    /// and give an empty string, so a spawn or restart never depends on them.
    pub fn role_rules_text(&self, repo: &Path, role_name: &str) -> String {
        let resolved = self
            .workflow_root(repo)
            .map_err(|e| e.to_string())
            .and_then(|root| {
                let layers = crate::rules::discover_layers(repo, root.as_deref(), &self.packs);
                crate::rules::load_and_resolve(&layers).map_err(|e| e.to_string())
            });
        match resolved {
            Ok(res) => {
                let text = crate::rules::rules_section(&res, role_name, None);
                // prime's "(none)" placeholder would be noise in a prompt.
                if text == crate::rules::NO_RULES {
                    String::new()
                } else {
                    text
                }
            }
            Err(e) => {
                tracing::warn!(role = role_name, error = %e, "workflow rules not resolvable; system prompt omits them");
                String::new()
            }
        }
    }
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
    let out = out.replace("{{commands.check_worker}}", commands.worker_check());
    let out = out.replace("{{branches.integration}}", &branches.integration);
    match &branches.release {
        Some(release) => out.replace("{{branches.release}}", release),
        None => out,
    }
}

/// The rendered `--append-system-prompt-file` contents for one agent:
/// [`stable_system_prompt`], then a short identity sentence (name, role,
/// cwd, branch) appended last, so an agent always knows these facts even
/// with no first message (docs/tickets/open/v1-follow-ups-from-the-build-9c6e.md).
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
    rules: &str,
    agent_name: &str,
    cwd: &Path,
    branch: Option<&str>,
    siblings: &[(String, PathBuf)],
) -> String {
    let mut out = stable_system_prompt(role_name, role, repo, branches, commands, rules);
    let branch_clause = branch
        .map(|b| format!(" on branch {b}"))
        .unwrap_or_default();
    out.push_str(&format!(
        "\nYou are {agent_name} (role {role_name}) in {}{branch_clause}.\n",
        cwd.display()
    ));
    for (name, path) in siblings {
        out.push_str(&format!(
            "Paired sibling repo {name} is at {}.\n",
            path.display()
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn auto_mode_environment_orders_and_limits_project_lines() {
        let repo = Path::new("/w/proj");
        let machine = vec!["Trusted repo: github.com:me/".to_string()];
        let project = vec![
            "Sensitive: the prod database".to_string(),
            "Trusted repo: github.com:evil/x".to_string(),
            "Prod host: deploy.example.com".to_string(),
        ];
        let env = auto_mode_environment(&machine, &project, repo);
        assert_eq!(env[0], "$defaults");
        assert!(env[1].contains("/w/proj/wt") && env[1].contains("/w/proj/.bridle/state"));
        assert_eq!(env[2], machine[0]);
        assert_eq!(env[3], project[0]);
        assert_eq!(env[4], project[2]);
        assert_eq!(env.len(), 5);
        assert_eq!(auto_mode_environment(&[], &[], repo).len(), 2);
    }

    #[test]
    fn auto_mode_sections_parse_in_both_files() {
        let dir = tempfile::tempdir().expect("tmp");
        let (home, repo) = (dir.path().join("home"), dir.path().join("repo"));
        std::fs::create_dir_all(&home).expect("mk");
        std::fs::create_dir_all(repo.join(".bridle")).expect("mk");
        std::fs::write(
            home.join("config.toml"),
            "[auto_mode]\nenvironment = [\"Trusted repo: github.com:me/\"]\n",
        )
        .expect("w");
        std::fs::write(
            repo.join(".bridle/config.toml"),
            "[auto_mode]\nenvironment = [\"Sensitive: prod\"]\n",
        )
        .expect("w");
        let env = auto_mode_environment_for(&home, &repo);
        assert_eq!(env[2], "Trusted repo: github.com:me/");
        assert_eq!(env[3], "Sensitive: prod");
        Config::parse("[auto_mode]\nenvironment = []\n").expect("accepted");
    }

    use super::*;

    #[test]
    fn accepts_sections_owned_by_other_crates() {
        for text in [
            "[gateway]\nport = 1\n",
            "[interactions]\ngap = \"5m\"\n",
            "[mail]\nx = 1\n[gateway]\nport = 1\n[interactions]\ngap = \"5m\"\n",
        ] {
            Config::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        }
    }

    #[test]
    fn every_builtin_role_denies_pattern_kills() {
        let cfg = Config::default();
        for (name, role) in &cfg.roles {
            for rule in DENY_PATTERN_KILLS {
                assert!(
                    role.disallowed_tools.contains(&rule.to_string()),
                    "{name} should deny {rule}"
                );
            }
        }
        assert!(!cfg.roles.is_empty());
    }

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

    fn tools_of(cfg: &Config, role: &str) -> Option<String> {
        cfg.roles[role].tools.as_ref().map(|t| t.join(","))
    }

    #[test]
    fn default_roles_get_the_spike_toolsets() {
        let cfg = Config::default();
        assert_eq!(
            tools_of(&cfg, "worker").as_deref(),
            Some("Bash,Read,Edit,Write,Glob,Grep")
        );
        assert_eq!(
            tools_of(&cfg, "manager").as_deref(),
            Some("Bash,Read,Glob,Grep,Agent")
        );
        assert!(
            cfg.roles["manager"]
                .allowed_tools
                .contains(&"Agent".to_string())
        );
        // Not measured yet (ct8m step 4): full toolset.
        assert_eq!(tools_of(&cfg, "orchestrator"), None);
    }

    #[test]
    fn researcher_is_a_built_in_worker_with_web_tools() {
        // No [roles.researcher] in the project's config: the default still exists.
        let cfg = Config::parse("[roles.worker]\nmodel = \"sonnet\"\n").expect("parse");
        let researcher = &cfg.roles["researcher"];
        for tool in ["WebSearch", "WebFetch"] {
            assert!(
                researcher.allowed_tools.contains(&tool.to_string()),
                "{tool}"
            );
            assert!(
                researcher
                    .tools
                    .as_ref()
                    .expect("tools")
                    .contains(&tool.to_string())
            );
            assert!(
                !cfg.roles["worker"]
                    .allowed_tools
                    .contains(&tool.to_string())
            );
            assert!(
                !cfg.roles["worker"]
                    .tools
                    .as_ref()
                    .expect("tools")
                    .contains(&tool.to_string())
            );
        }
        assert!(researcher.allowed_tools.contains(&"Edit".to_string()));
    }

    #[test]
    fn a_roles_tools_key_replaces_the_default() {
        let cfg = Config::parse(
            r#"
            [roles.worker]
            tools = ["Bash", "Read", "Skill"]

            [roles.reviewer]
            tools = ["Read"]
        "#,
        )
        .expect("parse");
        assert_eq!(tools_of(&cfg, "worker").as_deref(), Some("Bash,Read,Skill"));
        assert_eq!(tools_of(&cfg, "reviewer").as_deref(), Some("Read"));
        // Untouched roles keep theirs.
        assert!(tools_of(&cfg, "manager").is_some());
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
            self_upgrade = true
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
        assert!(cfg.self_upgrade);
        assert!(!Config::default().self_upgrade);
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
        // project-manager is a custom role, so it falls back to
        // Role::worker_default() as its merge base (stop_check: true); config
        // must turn it back off explicitly, since it isn't the worker role.
        assert!(!config.roles["project-manager"].stop_check);
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
            check_worker: None,
        };
        for name in ["worker", "manager", "project-manager"] {
            let role = Role {
                system_prompt: Some(format!("workflow/base/roles/{name}.md").into()),
                ..Role::worker_default()
            };
            let rendered = stable_system_prompt(name, &role, &repo, &branches, &commands, "");
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
            "",
        );
        assert!(
            !rendered.contains("{{branches."),
            "manager.md should have every {{{{branches.*}}}} placeholder substituted"
        );
        // Bridle's own project defaults to trunk on "main".
        assert!(rendered.contains("integration branch is `main`"));
        assert!(rendered.contains("bridle task land <task-id>"));
    }

    #[test]
    fn custom_role_falling_back_to_worker_default_can_turn_stop_check_back_off() {
        let toml = r#"
            [roles.project-manager]
            model = "sonnet"
            stop_check = false
        "#;
        let cfg = Config::parse(toml).expect("parse");
        let pm = &cfg.roles["project-manager"];
        // Confirms the fallback base really is worker_default (stop_check: true)
        // and that the project config can override it.
        assert!(Role::worker_default().stop_check);
        assert!(!pm.stop_check);
    }

    #[test]
    fn old_product_manager_role_name_resolves_as_project_manager() {
        let cfg = Config::parse("[roles.product-manager]\nstop_check = false\n").expect("parse");
        assert!(!cfg.roles["project-manager"].stop_check);
        assert!(!cfg.roles.contains_key("product-manager"));
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
    fn budget_schedule_preset_omits_days_start_end_and_never_matches() {
        let toml = r#"
            [[budget.schedule]]
            name = "burst"
            hold_at = 95
            wind_down_at = 97
            stop_at = 99
            max_workers = 4
        "#;
        let budget = parse_machine_budget(toml);
        let burst = &budget.schedule[0];
        assert_eq!((burst.start, burst.end), (None, None));
        assert_eq!(burst.max_workers, Some(4));
        let now = Local::now();
        for d in 0..7 {
            assert!(!burst.matches(now + chrono::Duration::days(d)));
        }
    }

    #[test]
    fn budget_schedule_rejects_a_partial_schedule() {
        let toml = r#"
            [[budget.schedule]]
            name = "half"
            start = "09:00"
            hold_at = 85
            wind_down_at = 92
            stop_at = 95
        "#;
        let raw: RawConfig = toml::from_str(toml).expect("toml");
        let err = BudgetConfig::default()
            .merge(raw.budget.unwrap_or_default())
            .unwrap_err();
        assert!(matches!(err, ConfigError::BadSchedule { .. }), "{err}");
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

    /// 2024-01-01 is a Monday; `d` is the day of January.
    fn jan(d: u32, h: u32, m: u32) -> DateTime<Local> {
        use chrono::{NaiveDate, TimeZone};
        Local
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2024, 1, d)
                    .expect("date")
                    .and_hms_opt(h, m, 0)
                    .expect("time"),
            )
            .single()
            .expect("unambiguous")
    }

    fn hm(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).expect("time")
    }

    fn focus(days: Vec<Weekday>, start: NaiveTime, end: NaiveTime, mode: FocusMode) -> FocusPeriod {
        FocusPeriod {
            name: "f".to_string(),
            days,
            start,
            end,
            mode,
        }
    }

    fn sched(days: Vec<Weekday>, start: NaiveTime, end: NaiveTime) -> SchedulePeriod {
        SchedulePeriod {
            name: "s".to_string(),
            days,
            start: Some(start),
            end: Some(end),
            hold_at: 85.0,
            wind_down_at: 92.0,
            stop_at: 95.0,
            max_workers: None,
        }
    }

    const NIGHT: &str = r#"
        [[budget.schedule]]
        name = "night"
        days = "all"
        start = "23:00"
        end = "END"
        hold_at = 70
        wind_down_at = 80
        stop_at = 90
    "#;

    #[test]
    fn plus_1d_end_parses_and_matches_the_morning_after() {
        let cfg = Config::parse(&NIGHT.replace("END", "08:00+1d")).expect("parses");
        let p = &cfg.budget.schedule[0];
        assert_eq!(p.end, Some(hm(8, 0)));
        assert!(p.matches(jan(2, 3, 0)) && !p.matches(jan(2, 9, 0)));
        let raw = RawFocusPeriod {
            name: "f".into(),
            days: RawDays::All("all".into()),
            start: "23:00".into(),
            end: "08:00+1d".into(),
            mode: None,
        };
        let f = raw.into_period().expect("focus parses");
        assert!(f.matches(jan(2, 3, 0)) && !f.matches(jan(2, 9, 0)));
    }

    #[test]
    fn overnight_end_without_plus_1d_is_a_validator_error_but_still_loads() {
        let msg = overnight_problem("night", "23:00", "08:00").expect("problem");
        assert_eq!(
            msg,
            "night: end 08:00 is before start 23:00; write \"08:00+1d\""
        );
        // Daemon load stays lenient: the 3xr4 next-day meaning, only a warning.
        let cfg = Config::parse(&NIGHT.replace("END", "08:00")).expect("still loads");
        assert!(cfg.budget.schedule[0].matches(jan(2, 3, 0)));
    }

    #[test]
    fn midnight_end_is_valid_and_start_equals_end_is_empty() {
        assert_eq!(overnight_problem("e", "21:30", "00:00"), None);
        assert_eq!(overnight_problem("e", "09:00", "09:00"), None);
        assert!(overnight_problem("e", "09:00", "17:00+1d").is_some());
        let s = sched(vec![Weekday::Mon], hm(9, 0), hm(9, 0));
        assert!(!s.matches(jan(1, 9, 0)));
    }

    #[test]
    fn overnight_period_belongs_to_the_day_it_starts() {
        // Sunday 23:00 to 07:00; Monday is not listed. Sun is Dec 31 2023, Mon Jan 1.
        let sun = vec![Weekday::Sun];
        let f = focus(sun.clone(), hm(23, 0), hm(7, 0), FocusMode::Quiet);
        let s = sched(sun, hm(23, 0), hm(7, 0));
        let sun_23 = jan(1, 23, 0) - chrono::Duration::days(1);
        let mon_03 = jan(1, 3, 0);
        for (name, got) in [
            ("focus sun 23:00", f.matches(sun_23)),
            ("focus mon 03:00", f.matches(mon_03)),
            ("sched sun 23:00", s.matches(sun_23)),
            ("sched mon 03:00", s.matches(mon_03)),
        ] {
            assert!(got, "{name}");
        }
        // Monday 23:00 and Tuesday 03:00: Monday isn't a listed start day.
        assert!(!f.matches(jan(1, 23, 0)) && !s.matches(jan(1, 23, 0)));
        assert!(!f.matches(jan(2, 3, 0)) && !s.matches(jan(2, 3, 0)));
        // Listing Monday too as a start day does not stop Sunday's night covering Monday morning.
        let both = focus(
            vec![Weekday::Mon, Weekday::Sun],
            hm(23, 0),
            hm(7, 0),
            FocusMode::Quiet,
        );
        assert!(both.matches(mon_03) && both.matches(jan(2, 3, 0)));
        // The end is exclusive.
        assert!(!f.matches(jan(1, 7, 0)));
    }

    #[test]
    fn same_day_and_empty_periods_are_unchanged() {
        let mon = vec![Weekday::Mon];
        let f = focus(mon.clone(), hm(9, 0), hm(17, 0), FocusMode::Quiet);
        let s = sched(mon.clone(), hm(9, 0), hm(17, 0));
        assert!(f.matches(jan(1, 9, 0)) && s.matches(jan(1, 16, 59)));
        assert!(!f.matches(jan(1, 17, 0)) && !s.matches(jan(1, 8, 59)));
        assert!(!f.matches(jan(2, 10, 0)) && !s.matches(jan(2, 10, 0)));
        // start == end never matched, and still doesn't.
        let f = focus(mon.clone(), hm(9, 0), hm(9, 0), FocusMode::Quiet);
        let s = sched(mon, hm(9, 0), hm(9, 0));
        assert!(!f.matches(jan(1, 9, 0)) && !s.matches(jan(1, 12, 0)));
    }

    #[test]
    fn focus_end_follows_touching_and_overlapping_periods() {
        let all = vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
        let evening = focus(all.clone(), hm(21, 30), hm(0, 0), FocusMode::Quiet);
        let night = focus(all.clone(), hm(0, 0), hm(6, 0), FocusMode::Quiet);
        let periods = [evening.clone(), night.clone()];
        let (end, capped) = focus_end(&periods, &evening, jan(1, 22, 0));
        assert_eq!((end, capped), (jan(2, 6, 0), false));
        // Overlapping: the second starts before the first ends and ends later.
        let a = focus(all.clone(), hm(22, 0), hm(1, 0), FocusMode::Quiet);
        let b = focus(all.clone(), hm(23, 0), hm(5, 0), FocusMode::Quiet);
        let (end, _) = focus_end(&[a.clone(), b], &a, jan(1, 22, 30));
        assert_eq!(end, jan(2, 5, 0));
        // A lone period, and a period of another mode, aren't followed.
        let locked = focus(all, hm(0, 0), hm(6, 0), FocusMode::Locked);
        let (end, _) = focus_end(&[evening.clone(), locked], &evening, jan(1, 22, 0));
        assert_eq!(end, jan(2, 0, 0));
    }

    #[test]
    fn focus_end_terminates_when_periods_cover_the_whole_week() {
        let all = vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
        let day = focus(all.clone(), hm(0, 0), hm(12, 0), FocusMode::Quiet);
        let pm = focus(all, hm(12, 0), hm(0, 0), FocusMode::Quiet);
        let now = jan(1, 1, 0);
        let (end, capped) = focus_end(&[day.clone(), pm], &day, now);
        assert!(capped);
        assert_eq!(end, now + chrono::Duration::days(7));
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
            start: NaiveTime::from_hms_opt(9, 0, 0),
            end: NaiveTime::from_hms_opt(17, 0, 0),
            hold_at: 85.0,
            wind_down_at: 92.0,
            stop_at: 95.0,
            max_workers: None,
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
            start: NaiveTime::from_hms_opt(23, 0, 0),
            end: NaiveTime::from_hms_opt(7, 0, 0),
            hold_at: 90.0,
            wind_down_at: 93.0,
            stop_at: 95.0,
            max_workers: None,
        };
        // Both sides of midnight match.
        assert!(night.matches(at(23, 30)));
        assert!(night.matches(at(6, 0)));
        assert!(!night.matches(at(12, 0)));
    }

    #[test]
    fn integration_warm_build_parses() {
        assert_eq!(Config::default().integration.warm_build, None);
        let cfg = Config::parse("[integration]\nwarm_build = \"cargo build\"\n").unwrap();
        assert_eq!(cfg.integration.warm_build.as_deref(), Some("cargo build"));
    }

    #[test]
    fn integration_check_skip_paths_parses() {
        assert!(Config::default().integration.check_skip_paths.is_empty());
        let cfg = Config::parse("[integration]\ncheck_skip_paths = [\"docs/**\"]\n").unwrap();
        assert_eq!(
            cfg.integration.check_skip_paths,
            vec!["docs/**".to_string()]
        );
    }

    #[test]
    fn integration_check_parses() {
        assert_eq!(Config::default().integration.check, None);
        let cfg = Config::parse("[integration]\ncheck = \"just check\"\n").unwrap();
        assert_eq!(cfg.integration.check.as_deref(), Some("just check"));
    }

    #[test]
    fn ports_config_parses() {
        assert_eq!(Config::default().ports.range, (4000, 4999));
        let cfg =
            Config::parse("[ports]\nrange = [5000, 5010]\nreserved = [5001, 5002]\n").unwrap();
        assert_eq!(cfg.ports.range, (5000, 5010));
        assert_eq!(cfg.ports.reserved, vec![5001, 5002]);
        assert!(Config::parse("[ports]\nrange = [5010, 5000]\n").is_err());
    }

    #[test]
    fn migrations_auto_defaults_on_and_parses() {
        assert!(Config::default().migrations_auto);
        assert!(
            !Config::parse("[migrations]\nauto = false\n")
                .unwrap()
                .migrations_auto
        );
    }

    #[test]
    fn state_push_defaults_on_and_parses() {
        assert!(Config::default().state_push);
        assert!(!Config::parse("[state]\npush = false\n").unwrap().state_push);
    }

    #[test]
    fn review_config_parses() {
        let d = Config::default().review;
        assert_eq!(d.quiet, Duration::from_secs(7 * 60));
        assert_eq!((d.max_agents, d.idle), (3, Duration::from_secs(4 * 3600)));
        let cfg =
            Config::parse("[review]\nquiet_minutes = 5\nmax_agents = 1\nidle_hours = 2\n").unwrap();
        assert_eq!(cfg.review.quiet, Duration::from_secs(300));
        assert_eq!(cfg.review.max_agents, 1);
        assert_eq!(cfg.review.idle, Duration::from_secs(7200));
    }

    #[test]
    fn machine_config_parses() {
        let d = Config::default().machine;
        assert_eq!(d.load_per_core, crate::load::DEFAULT_LOAD_PER_CORE);
        let cfg =
            Config::parse("[machine]\ncheck_interval = \"0s\"\nload_per_core = 3.5\n").unwrap();
        assert!(cfg.machine.check_interval.is_zero());
        assert_eq!(cfg.machine.load_per_core, 3.5);
    }

    #[test]
    fn disk_config_parses() {
        let d = Config::default().disk;
        assert_eq!(d.check_interval, Duration::from_secs(3600));
        assert_eq!(d.min_free_gb, 20);
        let cfg = Config::parse("[disk]\ncheck_interval = \"0s\"\nmin_free_gb = 5\n").unwrap();
        assert!(cfg.disk.check_interval.is_zero());
        assert_eq!(cfg.disk.min_free_gb, 5);
    }

    #[test]
    fn orchestrator_config_parses_and_validates() {
        let d = Config::default().orchestrator;
        assert!(!d.enabled);
        assert_eq!(d.relaunch_backoff.len(), 3);
        let cfg = Config::parse(
            "[orchestrator]\nenabled = true\nlauncher = \"/x/launch\"\n\
             relaunch_backoff = [\"5s\", \"1m\"]\nstable_after = \"2m\"\nwaiter_grace = \"30s\"\n",
        )
        .unwrap();
        let o = cfg.orchestrator;
        assert!(o.enabled);
        assert_eq!(o.launcher.as_deref(), Some("/x/launch"));
        assert_eq!(d.launcher, None);
        let old =
            Config::parse("[orchestrator]\nlauncher = \"scripts/claude-orchestrator\"\n").unwrap();
        assert_eq!(old.orchestrator.launcher, None);
        assert_eq!(
            o.relaunch_backoff,
            vec![Duration::from_secs(5), Duration::from_secs(60)]
        );
        assert_eq!(o.stable_after, Duration::from_secs(120));
        assert_eq!(o.waiter_grace, Duration::from_secs(30));
        assert_eq!(d.waiter_grace, Duration::from_secs(900));
        assert!(Config::parse("[orchestrator]\nrelaunch_backoff = []\n").is_err());
        assert!(Config::parse("[orchestrator]\nrelaunch_backoff = [\"5\"]\n").is_err());
        assert!(Config::parse("[orchestrator]\nlauncher = \" \"\n").is_err());
        // `pane` is gone: an unknown key is a warning now (6hx4), not a refusal to start.
        assert!(Config::parse("[orchestrator]\npane = \"%3\"\n").is_ok());
        assert_eq!(d.plan_tokens, 180_000);
        assert_eq!(d.handover_tokens, 200_000);
        assert_eq!(d.handover_deadline, Duration::from_secs(1800));
        let o = Config::parse(
            "[orchestrator]\nnote_tokens = 1\nplan_tokens = 2\nhandover_tokens = 3\n\
             handover_deadline = \"5m\"\nmax_uptime = \"1h\"\n",
        )
        .unwrap()
        .orchestrator;
        assert_eq!((o.note_tokens, o.plan_tokens, o.handover_tokens), (1, 2, 3));
        assert_eq!(o.max_uptime, Duration::from_secs(3600));
        // Out of order, zero, or a zero duration.
        assert!(Config::parse("[orchestrator]\nplan_tokens = 100\n").is_err());
        assert!(Config::parse("[orchestrator]\nnote_tokens = 0\n").is_err());
        assert!(Config::parse("[orchestrator]\nhandover_tokens = 170000\n").is_err());
        assert!(Config::parse("[orchestrator]\nmax_uptime = \"0s\"\n").is_err());
    }

    #[test]
    fn session_steps_default_override_per_role_and_validate() {
        let d = Config::parse("").unwrap().sessions;
        assert_eq!(
            d.steps_for("advisor/x"),
            [150_000, 200_000, 250_000, 300_000]
        );
        let c = Config::parse(
            "[sessions]\nwarn = [\"100k\", \"150k\", 200000, \"1M\"]\n\
             [sessions.aide]\nwarn = [1, 2, 3, 4]\n",
        )
        .unwrap()
        .sessions;
        assert_eq!(
            c.steps_for("advisor"),
            [100_000, 150_000, 200_000, 1_000_000]
        );
        assert_eq!(c.steps_for("aide"), [1, 2, 3, 4]);
        assert!(Config::parse("[sessions]\nwarn = [1, 2, 3]\n").is_err());
        assert!(Config::parse("[sessions]\nwarn = [1, 3, 2, 4]\n").is_err());
        assert!(Config::parse("[sessions]\nwarn = [0, 1, 2, 3]\n").is_err());
    }

    #[test]
    fn token_count_parser() {
        assert_eq!(parse_token_count("150000").unwrap(), 150_000);
        assert_eq!(parse_token_count("150k").unwrap(), 150_000);
        assert_eq!(parse_token_count("150K").unwrap(), 150_000);
        assert_eq!(parse_token_count("1.5M").unwrap(), 1_500_000);
        assert_eq!(parse_token_count("1.5m").unwrap(), 1_500_000);
        assert_eq!(parse_token_count("2M").unwrap(), 2_000_000);
        assert_eq!(parse_token_count("2m").unwrap(), 2_000_000);
        assert_eq!(parse_token_count("1k").unwrap(), 1_000);
        assert_eq!(parse_token_count(" 100k ").unwrap(), 100_000);
        // Invalid formats
        assert!(parse_token_count("").is_err());
        assert!(parse_token_count("abc").is_err());
        assert!(parse_token_count("150x").is_err());
        assert_eq!(parse_token_count("0.5k").unwrap(), 500);
        assert_eq!(parse_token_count("1.").unwrap(), 1);
        for bad in [
            "k",
            "M",
            ".",
            ".k",
            "-5k",
            "+5k",
            "1e3k",
            "inf",
            "nan",
            "1kk",
            "1 k",
            "1_000",
            "1.2.3k",
            "99999999999999999999999M",
        ] {
            assert!(
                parse_token_count(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn bad_token_counts_name_the_key() {
        for key in ["note_tokens", "plan_tokens", "handover_tokens"] {
            for bad in ["\"lots\"", "\"150x\"", "\"\"", "-5", "1.5", "true"] {
                let err = Config::parse(&format!("[orchestrator]\n{key} = {bad}\n"))
                    .expect_err(bad)
                    .to_string();
                assert!(err.contains(key), "{key} = {bad}: {err}");
            }
        }
    }

    #[test]
    fn old_integer_only_orchestrator_config_still_loads() {
        let o = Config::parse(
            "[orchestrator]\nenabled = true\nnote_tokens = 150000\nplan_tokens = 210000\n\
             handover_tokens = 255000\n",
        )
        .unwrap()
        .orchestrator;
        assert_eq!(
            (o.note_tokens, o.plan_tokens, o.handover_tokens),
            (150_000, 210_000, 255_000)
        );
        // Mixed integer and string forms are fine, and order is still checked.
        let o = Config::parse("[orchestrator]\nnote_tokens = 100000\nplan_tokens = \"1.5M\"\nhandover_tokens = \"2M\"\n")
            .unwrap()
            .orchestrator;
        assert_eq!(o.plan_tokens, 1_500_000);
        assert!(
            Config::parse("[orchestrator]\nnote_tokens = \"200k\"\nplan_tokens = \"180k\"\n")
                .is_err()
        );
    }

    #[test]
    fn orchestrator_token_counts_accept_k_and_m_suffix() {
        let cfg = Config::parse(
            "[orchestrator]\nnote_tokens = \"150k\"\nplan_tokens = \"180k\"\nhandover_tokens = \"200k\"\n",
        )
        .unwrap();
        let o = cfg.orchestrator;
        assert_eq!(o.note_tokens, 150_000);
        assert_eq!(o.plan_tokens, 180_000);
        assert_eq!(o.handover_tokens, 200_000);

        let cfg = Config::parse(
            "[orchestrator]\nnote_tokens = \"0.15M\"\nplan_tokens = \"0.18M\"\nhandover_tokens = \"0.2M\"\n",
        )
        .unwrap();
        let o = cfg.orchestrator;
        assert_eq!(o.note_tokens, 150_000);
        assert_eq!(o.plan_tokens, 180_000);
        assert_eq!(o.handover_tokens, 200_000);
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
    fn tasks_open_stale_parses_and_defaults() {
        let secs = |t: &str| Config::parse(t).expect("parse").tasks_open_stale.as_secs();
        assert_eq!(secs(""), 4 * 3600);
        assert_eq!(secs("[tasks]\nopen_stale = \"2h\"\n"), 7200);
        assert_eq!(secs("[tasks]\nopen_stale = 0\n"), 0);
        assert_eq!(secs("[tasks]\nopen_stale = \"soon\"\n"), 4 * 3600);
    }

    #[test]
    fn tasks_settle_parses_defaults_and_falls_back_on_a_bad_value() {
        let secs = |t: &str| Config::parse(t).expect("parse").tasks_settle.as_secs();
        assert_eq!(secs(""), 300);
        assert_eq!(secs("[tasks]\nsettle = \"10m\"\n"), 600);
        assert_eq!(secs("[tasks]\nsettle = 0\n"), 0);
        assert_eq!(secs("[tasks]\nsettle = \"0\"\n"), 0);
        for bad in ["\"soon\"", "7", "true"] {
            let cfg = Config::parse(&format!("[tasks]\nsettle = {bad}\n")).expect("still loads");
            assert_eq!(cfg.tasks_settle.as_secs(), 300);
            let problem = cfg.tasks_settle_problem.expect("problem recorded");
            assert!(problem.contains("[tasks] settle"), "{problem}");
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
        let branches = BranchesConfig::default();
        for (name, role) in Config::default().roles {
            let rendered = stable_system_prompt(
                &name,
                &role,
                repo,
                &branches,
                &CommandsConfig::default(),
                "",
            );
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
        let a = stable_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "",
        );
        let b = stable_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "",
        );
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
            "",
        );
        let manager = stable_system_prompt(
            "manager",
            &Role::manager_default(),
            repo,
            &branches,
            &CommandsConfig::default(),
            "",
        );
        assert_ne!(worker, manager);
    }

    #[test]
    fn rendered_prompt_carries_identity_after_the_shared_prefix() {
        let repo = Path::new("/repo/a");
        let branches = BranchesConfig::default();
        let role = Role::worker_default();
        let stable = stable_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "",
        );
        let a = render_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "",
            "worker-1",
            Path::new("/repo/a/wt/worker-1"),
            Some("bridle/worker-1"),
            &[],
        );
        let b = render_system_prompt(
            "worker",
            &role,
            repo,
            &branches,
            &CommandsConfig::default(),
            "",
            "worker-2",
            Path::new("/repo/a/wt/worker-2"),
            Some("bridle/worker-2"),
            &[],
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
        assert_eq!(cfg.commands.worker_check(), "just check");
        assert!(cfg.warm_target);
        let cfg = Config::parse("[commands]\ncheck_worker = \"just check-affected\"\n[worktrees]\nwarm_target = false\n")
            .expect("parse");
        assert_eq!(cfg.commands.check, "just check");
        assert_eq!(cfg.commands.worker_check(), "just check-affected");
        assert!(!cfg.warm_target);
        assert!(cfg.setup.is_none());
        let cfg = Config::parse("[worktrees]\nsetup = \"npm ci\"\nsetup_timeout_secs = 5\n")
            .expect("parse");
        assert_eq!(cfg.setup.as_deref(), Some("npm ci"));
        assert_eq!(cfg.setup_timeout, Duration::from_secs(5));
        assert!(cfg.copy.is_empty());
        let cfg = Config::parse("[worktrees]\ncopy = [\".env\", \"a/b.json\"]\n").expect("parses");
        assert_eq!(cfg.copy, [".env", "a/b.json"]);
        for bad in ["/etc/passwd", "../x", "a/../../x", ""] {
            let toml = format!("[worktrees]\ncopy = [{bad:?}]\n");
            assert!(
                matches!(Config::parse(&toml), Err(ConfigError::BadCopyPath(_))),
                "{bad}"
            );
        }

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
    fn workflow_path_expands_tilde_and_vars() {
        let home = std::env::var("HOME").expect("HOME");
        assert_eq!(expand_path("~/wf").expect("tilde"), format!("{home}/wf"));
        assert_eq!(
            expand_path("$HOME/a/${HOME}").expect("vars"),
            format!("{home}/a/{home}")
        );
        assert_eq!(expand_path("rel/~x/$").expect("plain"), "rel/~x/$");
        let err = expand_path("$BRIDLE_SURELY_UNSET_VAR/wf").expect_err("unset");
        assert!(err.to_string().contains("BRIDLE_SURELY_UNSET_VAR"), "{err}");
    }

    #[test]
    fn machine_workflow_overrides_the_project_and_is_expanded() {
        let repo = tempfile::tempdir().expect("repo");
        let home = tempfile::tempdir().expect("home");
        std::fs::create_dir_all(repo.path().join(".bridle")).expect("mkdir");
        std::fs::write(
            repo.path().join(".bridle/config.toml"),
            "workflow = \"/elsewhere/workflow\"\n",
        )
        .expect("write");
        let cfg = Config::load_with_home(repo.path(), Some(home.path())).expect("load");
        assert_eq!(cfg.workflow.as_deref(), Some("/elsewhere/workflow"));

        std::fs::write(
            home.path().join("config.toml"),
            "workflow = \"$HOME/machine-wf\"\n",
        )
        .expect("write");
        let cfg = Config::load_with_home(repo.path(), Some(home.path())).expect("load");
        let h = std::env::var("HOME").expect("HOME");
        assert_eq!(
            cfg.workflow.as_deref(),
            Some(format!("{h}/machine-wf").as_str())
        );
    }

    #[test]
    fn workflow_root_errors_on_a_missing_dir_or_git_url_and_resolves_an_existing_one() {
        let repo = tempfile::tempdir().expect("repo");
        let mut cfg = Config::parse("").expect("parse");
        assert_eq!(cfg.workflow_root(repo.path()).expect("unset"), None);

        cfg.workflow = Some("wf".to_string());
        let err = cfg.workflow_root(repo.path()).expect_err("missing");
        assert!(
            matches!(err, ConfigError::WorkflowDirMissing { .. }),
            "{err}"
        );

        std::fs::create_dir(repo.path().join("wf")).expect("mkdir");
        assert_eq!(
            cfg.workflow_root(repo.path()).expect("present"),
            Some(repo.path().join("wf"))
        );

        cfg.workflow = Some("https://example.com/wf.git".to_string());
        let err = cfg.workflow_root(repo.path()).expect_err("url");
        assert!(matches!(err, ConfigError::WorkflowGitUrl { .. }), "{err}");
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
            "",
            "manager-1",
            repo,
            None,
            &[],
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
        // An unknown key is a warning now (6hx4); a wrong type is still a parse error.
        assert!(Config::parse("[components.a]\nbogus = 1").is_ok());
        let err = Config::parse("[components.a]\nparent = 1").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn unset_system_prompt_defaults_to_the_base_role_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        std::fs::create_dir_all(repo.join(".bridle")).expect("mkdir");
        std::fs::create_dir_all(repo.join("wf/base/roles")).expect("mkdir");
        std::fs::write(repo.join("wf/base/roles/worker.md"), "w").expect("write");
        std::fs::write(repo.join("wf/base/roles/manager.md"), "m").expect("write");
        std::fs::write(
            repo.join(".bridle/config.toml"),
            "workflow = \"wf\"\n[roles.manager]\nsystem_prompt = \"mine.md\"\n",
        )
        .expect("write");
        let cfg = Config::load_with_home(repo, Some(repo)).expect("load");
        assert_eq!(
            cfg.roles["worker"].system_prompt.as_deref(),
            Some(Path::new("wf/base/roles/worker.md"))
        );
        assert_eq!(
            cfg.roles["manager"].system_prompt.as_deref(),
            Some(Path::new("mine.md"))
        );
        // No base file for the role: still no prompt.
        assert_eq!(cfg.roles["orchestrator"].system_prompt, None);
    }

    #[test]
    fn worktree_pairs_parse_and_refuse_bad_config() {
        let cfg = Config::parse(
            "[worktrees]\nlayout = \"paired\"\nroot = \"/w/{task}\"\n\
             [worktrees.pair.web]\npath = \"/r/web\"\nmode = \"symlink\"\n\
             [worktrees.pair.api]\npath = \"/r/api\"\n",
        )
        .expect("parses");
        assert!(matches!(cfg.worktree_layout, WorktreeLayout::Paired(_)));
        assert_eq!(cfg.worktree_pairs["web"].mode, PairMode::Symlink);
        assert_eq!(cfg.worktree_pairs["api"].mode, PairMode::Worktree);
        let r = cfg.worktree_layout.resolve(Path::new("/x/proj"), "t", "a");
        assert_eq!(r, Some(PathBuf::from("/w/t/proj")));
        for bad in [
            "layout = \"paired\"\nroot = \"/w/{task}\"",
            "layout = \"root\"\nroot = \"/w/{task}\"\n[worktrees.pair.a]\npath = \"/r\"",
            "layout = \"paired\"\nroot = \"/w/{task}\"\n[worktrees.pair.a]\npath = \"r\"",
            "layout = \"paired\"\nroot = \"/w/{task}\"\n[worktrees.pair.a]\npath = \"/r\"\nmode = \"x\"",
            "layout = \"paired\"\nroot = \"/w/{task}\"\n[worktrees.pair.A_b]\npath = \"/r\"",
        ] {
            let err = Config::parse(&format!("[worktrees]\n{bad}\n")).expect_err(bad);
            assert!(
                matches!(err, ConfigError::BadWorktreeLayout(_)),
                "{bad}: {err}"
            );
        }
    }

    #[test]
    fn worktree_layout_parses_and_refuses_bad_roots() {
        let cfg = Config::parse("").expect("parses");
        assert_eq!(cfg.worktree_layout, WorktreeLayout::Default);
        let cfg = Config::parse("[worktrees]\nlayout = \"root\"\nroot = \"/w/{project}/{task}\"\n")
            .expect("parses");
        let path = cfg
            .worktree_layout
            .resolve(Path::new("/x/proj"), "br-1", "w1")
            .expect("root");
        assert_eq!(path, PathBuf::from("/w/proj/br-1"));
        for bad in [
            "layout = \"root\"",
            "root = \"/w/{task}\"",
            "layout = \"root\"\nroot = \"w/{task}\"",
            "layout = \"root\"\nroot = \"/w/../{task}\"",
            "layout = \"root\"\nroot = \"/w/{nope}\"",
            "layout = \"root\"\nroot = \"/w/fixed\"",
            "layout = \"bogus\"",
        ] {
            let err = Config::parse(&format!("[worktrees]\n{bad}\n")).expect_err(bad);
            assert!(
                matches!(err, ConfigError::BadWorktreeLayout(_)),
                "{bad}: {err}"
            );
        }
    }

    #[test]
    fn deny_focus_files_has_only_edit_rules() {
        for rule in DENY_FOCUS_FILES {
            assert!(
                rule.starts_with("Edit("),
                "DENY_FOCUS_FILES should only contain Edit rules, not {rule:?}"
            );
        }
        assert!(
            DENY_FOCUS_FILES.contains(&"Edit(~/.bridle/focus*)"),
            "Edit(~/.bridle/focus*) should be in DENY_FOCUS_FILES"
        );
        assert!(
            DENY_FOCUS_FILES.contains(&"Edit(~/.bridle/config.toml)"),
            "Edit(~/.bridle/config.toml) should be in DENY_FOCUS_FILES"
        );
    }

    fn write_rule(root: &Path, rel: &str, id: &str, roles: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            p,
            format!("---\nid: {id}\nroles: {roles}\n---\nbody of {id}\n"),
        )
        .expect("write");
    }

    fn rules_repo() -> (tempfile::TempDir, Config) {
        let dir = tempfile::tempdir().expect("tempdir");
        let r = dir.path();
        write_rule(r, "wf/base/rules/base-rule.md", "base-rule", "[]");
        write_rule(r, "wf/packs/p/rules/pack-rule.md", "pack-rule", "[worker]");
        write_rule(r, ".bridle/rules/proj-rule.md", "proj-rule", "[worker]");
        write_rule(r, ".bridle/rules/mgr-rule.md", "mgr-rule", "[manager]");
        let mut cfg = Config::parse("").expect("parse");
        cfg.workflow = Some("wf".to_string());
        cfg.packs = vec!["p".to_string()];
        (dir, cfg)
    }

    #[test]
    fn role_rules_reach_the_stable_prompt_and_differ_only_by_role_tags() {
        let (dir, cfg) = rules_repo();
        let repo = dir.path();
        let prompt = |role: &str| {
            let rules = cfg.role_rules_text(repo, role);
            stable_system_prompt(
                role,
                &Role::worker_default(),
                repo,
                &cfg.branches,
                &cfg.commands,
                &rules,
            )
        };
        let worker = prompt("worker");
        assert_eq!(
            worker,
            prompt("worker"),
            "same bytes for every agent of a role"
        );
        for id in ["base-rule", "pack-rule", "proj-rule"] {
            assert!(worker.contains(&format!("- {id} [")), "{id}");
        }
        assert!(!worker.contains("mgr-rule"));
        let manager = prompt("manager");
        assert!(manager.contains("base-rule") && manager.contains("mgr-rule"));
        assert!(!manager.contains("pack-rule") && !manager.contains("proj-rule"));
    }

    #[test]
    fn unresolvable_rules_give_no_rules_text() {
        let (dir, mut cfg) = rules_repo();
        cfg.workflow = Some("missing-dir".to_string());
        assert_eq!(cfg.role_rules_text(dir.path(), "worker"), "");
        let (dir, cfg) = rules_repo();
        write_rule(dir.path(), ".bridle/rules/dup.md", "base-rule", "[]");
        assert_eq!(cfg.role_rules_text(dir.path(), "worker"), "");
    }

    fn write_hook(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, text).expect("write");
    }

    const GUARD: &str =
        r#"[{"matcher":"Edit","hooks":[{"type":"command","command":"bridle arch-guard"}]}]"#;

    #[test]
    fn layer_hooks_include_base_and_project_events() {
        let (dir, cfg) = rules_repo();
        let r = dir.path();
        write_hook(r, "wf/base/hooks/PreToolUse.json", GUARD);
        write_hook(
            r,
            ".bridle/hooks/PostToolUse.json",
            r#"[{"hooks":[{"type":"command","command":"proj-post"}]}]"#,
        );
        let hooks = cfg.layer_hooks(r);
        assert_eq!(
            hooks.keys().map(String::as_str).collect::<Vec<_>>(),
            ["PostToolUse", "PreToolUse"]
        );
        assert_eq!(
            hooks["PreToolUse"][0]["hooks"][0]["command"],
            "bridle arch-guard"
        );
    }

    #[test]
    fn layer_hooks_skip_malformed_files_and_keep_the_rest() {
        let (dir, cfg) = rules_repo();
        let r = dir.path();
        write_hook(r, "wf/base/hooks/PreToolUse.json", GUARD);
        write_hook(r, ".bridle/hooks/Stop.json", "{not json");
        write_hook(r, ".bridle/hooks/PostToolUse.json", r#"{"not":"an array"}"#);
        let hooks = cfg.layer_hooks(r);
        assert_eq!(hooks.len(), 1);
        assert!(hooks.contains_key("PreToolUse"));
    }

    #[test]
    fn no_hooks_dir_means_no_layer_hooks() {
        let (dir, cfg) = rules_repo();
        assert!(cfg.layer_hooks(dir.path()).is_empty());
    }

    #[test]
    fn layer_hooks_already_in_committed_settings_are_not_doubled() {
        let (dir, cfg) = rules_repo();
        let r = dir.path();
        write_hook(r, "wf/base/hooks/PreToolUse.json", GUARD);
        write_hook(
            r,
            ".claude/settings.json",
            &format!(r#"{{"hooks":{{"PreToolUse":{GUARD}}}}}"#),
        );
        assert!(cfg.layer_hooks(r).is_empty());
    }
}
