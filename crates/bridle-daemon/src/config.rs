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

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: SocketAddr,
    pub stall_after: Duration,
    pub stop_grace: Duration,
    pub roles: BTreeMap<String, Role>,
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
        }
    }
}

impl Config {
    /// Loads `<repo>/.bridle/config.toml` over the built-in defaults. Missing
    /// file is not an error: the file is optional (docs/design/agent-host/operating-model.md).
    pub fn load(repo: &Path) -> Result<Self, ConfigError> {
        let path = repo.join(".bridle").join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// Parses config TOML text over the built-in defaults. Split out from
    /// [`Config::load`] so tests don't need real files.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let raw: RawConfig = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: PathBuf::from("config.toml"),
            source: Box::new(source),
        })?;

        let mut config = Config::default();

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
/// agent names, paths or timestamps: those go in the first user message.
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

/// The rendered `--append-system-prompt-file` contents for an agent of
/// `role_name`: the fixed preamble, a role-specific sentence for the three
/// built-in roles, then the role's own prompt file if it has one.
pub fn render_system_prompt(role_name: &str, role: &Role, repo: &Path) -> String {
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
    fn preamble_has_no_per_agent_data() {
        let repo = Path::new("/does/not/matter");
        for (name, role) in Config::default().roles {
            let rendered = render_system_prompt(&name, &role, repo);
            assert!(
                !rendered.contains("BRIDLE_AGENT_ID="),
                "preamble must not embed an id"
            );
            assert!(
                !rendered.contains(repo.to_str().expect("utf8 path")),
                "preamble must not embed a path"
            );
            assert!(
                !rendered.contains("2026"),
                "preamble must not embed a timestamp"
            );
        }
    }

    #[test]
    fn preamble_identical_for_two_agents_of_the_same_role() {
        let repo = Path::new("/repo/a");
        let role = Role::worker_default();
        let a = render_system_prompt("worker", &role, repo);
        let b = render_system_prompt("worker", &role, repo);
        assert_eq!(a, b);
    }

    #[test]
    fn preamble_differs_by_role() {
        let repo = Path::new("/repo/a");
        let worker = render_system_prompt("worker", &Role::worker_default(), repo);
        let manager = render_system_prompt("manager", &Role::manager_default(), repo);
        assert_ne!(worker, manager);
    }
}
