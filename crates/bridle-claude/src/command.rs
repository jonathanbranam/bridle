//! `ClaudeCommand`: builds the argv/env for one headless `claude` process,
//! per docs/design/agent-host/agents.md. Kept as a pure builder (`args()` returns a
//! `Vec<String>`) so the flag set is unit-testable without spawning
//! anything.

use std::path::PathBuf;

use uuid::Uuid;

/// Whether the process starts a new conversation or resumes one.
/// `--session-id` and `--resume` are never both passed (claude rejects that
/// unless `--fork-session` is also given; see S7 in
/// docs/spikes/01-stream-json-findings.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    New(Uuid),
    Resume(Uuid),
}

#[derive(Debug, Clone)]
pub struct ClaudeCommand {
    /// The executable to run. Defaults to `"claude"`; tests point this at
    /// `tests/fake-claude.py`.
    pub program: String,
    pub cwd: PathBuf,
    pub session: Session,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub append_system_prompt_file: Option<PathBuf>,
    pub permission_mode: Option<String>,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub name: Option<String>,
    /// `--max-budget-usd`: per process, checked after each model call.
    pub max_budget_usd: Option<f64>,
    /// Applied on top of the stripped inherited environment; see
    /// [`env_removal_keys`].
    pub env: Vec<(String, String)>,
    /// Registers `bridle stop-check` as a `Stop` hook (docs/design/
    /// coordination.md, docs/spikes/05-stop-hook-findings.md): worker role
    /// only, set from `Role::stop_check`.
    pub stop_check: bool,
    pub extra_args: Vec<String>,
}

/// The `Stop` hook command name: resolved on the agent's own `PATH`, which
/// the supervisor already points at the running daemon's own binary
/// (`agent_path` in supervisor.rs), so the hook always calls the same
/// version of `bridle` the agent itself does.
const STOP_CHECK_HOOK_COMMAND: &str = "bridle stop-check";

/// The `--settings` JSON passed on every spawn. Agents never use Claude
/// Code's auto memory: everything durable goes in the repo, where the human
/// and other agents can read it (docs/proposal/decisions.md, no assistant
/// memory). `stop_check` adds the `Stop` hook shape confirmed by spike 05
/// (flat `decision`/`reason`, not `hookSpecificOutput`).
fn settings_json(stop_check: bool) -> String {
    let mut settings = serde_json::json!({
        "autoMemoryEnabled": false,
        "autoDreamEnabled": false,
    });
    if stop_check {
        settings["hooks"] = serde_json::json!({
            "Stop": [
                {
                    "matcher": "",
                    "hooks": [
                        { "type": "command", "command": STOP_CHECK_HOOK_COMMAND }
                    ]
                }
            ]
        });
    }
    settings.to_string()
}

impl ClaudeCommand {
    pub fn new(cwd: impl Into<PathBuf>, session: Session) -> Self {
        Self {
            program: "claude".to_string(),
            cwd: cwd.into(),
            session,
            model: None,
            effort: None,
            append_system_prompt_file: None,
            permission_mode: None,
            allowed_tools: Vec::new(),
            disallowed_tools: Vec::new(),
            name: None,
            max_budget_usd: None,
            env: Vec::new(),
            stop_check: false,
            extra_args: Vec::new(),
        }
    }

    /// The argv (excluding the program name itself), in the order given by
    /// docs/design/agent-host/agents.md.
    pub fn args(&self) -> Vec<String> {
        let mut args: Vec<String> = [
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--replay-user-messages",
            "--exclude-dynamic-system-prompt-sections",
            "--strict-mcp-config",
            "--permission-prompts",
            "none",
            // `project` only: excludes the human's `~/.claude/settings.json`
            // and the clone's untracked `.claude/settings.local.json` from
            // ever loading into a spawned agent (docs/questions/open/
            // per-role-claude-settings-for-spawned-agents-4eep.md).
            "--setting-sources",
            "project",
            "--settings",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        args.push(settings_json(self.stop_check));

        match &self.session {
            Session::New(id) => {
                args.push("--session-id".into());
                args.push(id.to_string());
            }
            Session::Resume(id) => {
                args.push("--resume".into());
                args.push(id.to_string());
            }
        }

        if let Some(model) = &self.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        if let Some(effort) = &self.effort {
            args.push("--effort".into());
            args.push(effort.clone());
        }
        if let Some(path) = &self.append_system_prompt_file {
            args.push("--append-system-prompt-file".into());
            args.push(path.to_string_lossy().into_owned());
        }
        if let Some(mode) = &self.permission_mode {
            args.push("--permission-mode".into());
            args.push(mode.clone());
        }
        // `--allowedTools <tools...>` is variadic (verified via `claude
        // --help`): one flag followed by each tool as its own argv element.
        if !self.allowed_tools.is_empty() {
            args.push("--allowedTools".into());
            args.extend(self.allowed_tools.iter().cloned());
        }
        if !self.disallowed_tools.is_empty() {
            args.push("--disallowedTools".into());
            args.extend(self.disallowed_tools.iter().cloned());
        }
        if let Some(name) = &self.name {
            args.push("--name".into());
            args.push(name.clone());
        }
        if let Some(usd) = self.max_budget_usd {
            args.push("--max-budget-usd".into());
            args.push(usd.to_string());
        }

        args.extend(self.extra_args.iter().cloned());
        args
    }
}

/// Keys to remove from the inherited environment before spawning: every
/// `CLAUDE*` variable (including `CLAUDECODE`) and `BRIDLE_TOKEN` (spike
/// surprise 12 / agents.md). Bridle itself may run inside a
/// Claude Code session, and the child must not inherit that session's
/// identity or the parent daemon's own token.
pub fn env_removal_keys(vars: impl IntoIterator<Item = (String, String)>) -> Vec<String> {
    vars.into_iter()
        .map(|(k, _)| k)
        .filter(|k| k.starts_with("CLAUDE") || k == "BRIDLE_TOKEN")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    #[test]
    fn new_session_passes_session_id_not_resume() {
        let cmd = ClaudeCommand::new("/tmp", Session::New(uuid(1)));
        let args = cmd.args();
        assert!(args.contains(&"--session-id".to_string()));
        assert!(!args.contains(&"--resume".to_string()));
    }

    #[test]
    fn resume_session_passes_resume_not_session_id() {
        let cmd = ClaudeCommand::new("/tmp", Session::Resume(uuid(2)));
        let args = cmd.args();
        assert!(args.contains(&"--resume".to_string()));
        assert!(!args.contains(&"--session-id".to_string()));
    }

    #[test]
    fn never_both_session_id_and_resume() {
        for session in [Session::New(uuid(3)), Session::Resume(uuid(4))] {
            let cmd = ClaudeCommand::new("/tmp", session);
            let args = cmd.args();
            let has_session_id = args.contains(&"--session-id".to_string());
            let has_resume = args.contains(&"--resume".to_string());
            assert!(!(has_session_id && has_resume), "{args:?}");
            assert!(has_session_id || has_resume, "{args:?}");
        }
    }

    #[test]
    fn always_includes_fixed_flags() {
        let cmd = ClaudeCommand::new("/tmp", Session::New(uuid(5)));
        let args = cmd.args();
        for flag in [
            "-p",
            "--input-format",
            "--output-format",
            "--verbose",
            "--replay-user-messages",
            "--exclude-dynamic-system-prompt-sections",
            "--strict-mcp-config",
            "--permission-prompts",
            "--setting-sources",
            "--settings",
        ] {
            assert!(
                args.contains(&flag.to_string()),
                "missing {flag} in {args:?}"
            );
        }
        let i = args
            .iter()
            .position(|a| a == "--settings")
            .expect("--settings");
        let settings: serde_json::Value =
            serde_json::from_str(&args[i + 1]).expect("--settings is JSON");
        assert_eq!(settings["autoMemoryEnabled"], false);
        assert_eq!(settings["autoDreamEnabled"], false);
    }

    #[test]
    fn stop_check_false_omits_hooks_from_settings() {
        let cmd = ClaudeCommand::new("/tmp", Session::New(uuid(9)));
        let args = cmd.args();
        let i = args
            .iter()
            .position(|a| a == "--settings")
            .expect("--settings");
        let settings: serde_json::Value =
            serde_json::from_str(&args[i + 1]).expect("--settings is JSON");
        assert!(settings.get("hooks").is_none());
    }

    #[test]
    fn stop_check_true_adds_stop_hook_to_settings() {
        let mut cmd = ClaudeCommand::new("/tmp", Session::New(uuid(10)));
        cmd.stop_check = true;
        let args = cmd.args();
        let i = args
            .iter()
            .position(|a| a == "--settings")
            .expect("--settings");
        let settings: serde_json::Value =
            serde_json::from_str(&args[i + 1]).expect("--settings is JSON");
        let command = settings["hooks"]["Stop"][0]["hooks"][0]["command"]
            .as_str()
            .expect("hooks.Stop[0].hooks[0].command");
        assert_eq!(command, "bridle stop-check");
    }

    #[test]
    fn always_restricts_setting_sources_to_project() {
        let cmd = ClaudeCommand::new("/tmp", Session::New(uuid(8)));
        let args = cmd.args();
        let i = args
            .iter()
            .position(|a| a == "--setting-sources")
            .expect("--setting-sources");
        assert_eq!(args[i + 1], "project");
    }

    #[test]
    fn allowed_tools_are_separate_argv_elements_after_one_flag() {
        let mut cmd = ClaudeCommand::new("/tmp", Session::New(uuid(6)));
        cmd.allowed_tools = vec!["Bash(git *)".to_string(), "Edit".to_string()];
        let args = cmd.args();
        let idx = args
            .iter()
            .position(|a| a == "--allowedTools")
            .expect("flag present");
        assert_eq!(args[idx + 1], "Bash(git *)");
        assert_eq!(args[idx + 2], "Edit");
        // Only one occurrence of the flag itself.
        assert_eq!(args.iter().filter(|a| *a == "--allowedTools").count(), 1);
    }

    #[test]
    fn omits_optional_flags_when_unset() {
        let cmd = ClaudeCommand::new("/tmp", Session::New(uuid(7)));
        let args = cmd.args();
        for flag in [
            "--model",
            "--effort",
            "--append-system-prompt-file",
            "--permission-mode",
            "--allowedTools",
            "--disallowedTools",
            "--name",
        ] {
            assert!(
                !args.contains(&flag.to_string()),
                "unexpected {flag} in {args:?}"
            );
        }
    }

    #[test]
    fn env_removal_strips_claude_and_bridle_token() {
        let vars = vec![
            ("CLAUDECODE".to_string(), "1".to_string()),
            ("CLAUDE_CODE_SESSION_ID".to_string(), "abc".to_string()),
            ("BRIDLE_TOKEN".to_string(), "secret".to_string()),
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("HOME".to_string(), "/home/x".to_string()),
        ];
        let removed = env_removal_keys(vars);
        assert!(removed.contains(&"CLAUDECODE".to_string()));
        assert!(removed.contains(&"CLAUDE_CODE_SESSION_ID".to_string()));
        assert!(removed.contains(&"BRIDLE_TOKEN".to_string()));
        assert!(!removed.contains(&"PATH".to_string()));
        assert!(!removed.contains(&"HOME".to_string()));
        assert_eq!(removed.len(), 3);
    }
}
