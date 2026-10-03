//! `ClaudeCommand`: builds the argv/env for one headless `claude` process,
//! per docs/design/agent-host/agents.md. Kept as a pure builder (`args()` returns a
//! `Vec<String>`) so the flag set is unit-testable without spawning
//! anything.

use std::collections::BTreeMap;
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
    /// `--tools`: the built-in tools that exist for the agent at all. Unlike
    /// `allowed_tools`, unlisted tools' definitions leave its context
    /// (docs/spikes/08-lean-context-findings.md). `None` passes no flag.
    pub tools: Option<Vec<String>>,
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
    /// Layer hooks (`hooks/<event>.json`, event -> array of hook entries),
    /// merged into the `--settings` hooks. Bridle's own `Stop` entry stays
    /// first in its event's array; layer entries follow it.
    pub layer_hooks: BTreeMap<String, serde_json::Value>,
    pub extra_args: Vec<String>,
}

/// The `Stop` hook command name: resolved on the agent's own `PATH`, which
/// the supervisor already points at the running daemon's own binary
/// (`agent_path` in supervisor.rs), so the hook always calls the same
/// version of `bridle` the agent itself does.
const STOP_CHECK_HOOK_COMMAND: &str = "bridle stop-check";

/// The hook may run the project's `check_worker` command itself, which can
/// outlast Claude Code's default hook timeout.
const STOP_CHECK_TIMEOUT_SECS: u64 = 1800;

/// The `--settings` JSON passed on every spawn. Agents never use Claude
/// Code's auto memory: everything durable goes in the repo, where the human
/// and other agents can read it (docs/proposal/decisions.md, no assistant
/// memory). `stop_check` adds the `Stop` hook shape confirmed by spike 05
/// (flat `decision`/`reason`, not `hookSpecificOutput`).
fn settings_json(
    stop_check: bool,
    keeps_skill: bool,
    layer_hooks: &BTreeMap<String, serde_json::Value>,
) -> String {
    let mut settings = serde_json::json!({
        "autoMemoryEnabled": false,
        "autoDreamEnabled": false,
    });
    // Without `Skill` in `--tools` the skill listing is already gone; a role
    // that keeps it drops Claude Code's bundled skills instead (spike 08).
    if keeps_skill {
        settings["disableBundledSkills"] = true.into();
    }
    if stop_check {
        settings["hooks"] = serde_json::json!({
            "Stop": [
                {
                    "matcher": "",
                    "hooks": [
                        {
                            "type": "command",
                            "command": STOP_CHECK_HOOK_COMMAND,
                            "timeout": STOP_CHECK_TIMEOUT_SECS
                        }
                    ]
                }
            ]
        });
    }
    for (event, entries) in layer_hooks {
        let Some(entries) = entries.as_array().filter(|a| !a.is_empty()) else {
            continue;
        };
        let slot = &mut settings["hooks"];
        if slot.is_null() {
            *slot = serde_json::json!({});
        }
        match slot[event.as_str()].as_array_mut() {
            Some(existing) => existing.extend(entries.iter().cloned()),
            None => slot[event.as_str()] = entries.clone().into(),
        }
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
            tools: None,
            disallowed_tools: Vec::new(),
            name: None,
            max_budget_usd: None,
            env: Vec::new(),
            stop_check: false,
            layer_hooks: BTreeMap::new(),
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
            // ever loading into a spawned agent (docs/tickets/open/
            // per-role-claude-settings-for-spawned-agents-4eep.md).
            "--setting-sources",
            "project",
            "--settings",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        args.push(settings_json(
            self.stop_check,
            self.tools
                .as_ref()
                .is_some_and(|t| t.iter().any(|x| x == "Skill")),
            &self.layer_hooks,
        ));

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
        if let Some(tools) = &self.tools {
            args.push("--tools".into());
            args.push(tools.join(","));
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
    fn layer_hooks_follow_the_stop_hook_and_add_other_events() {
        let mut cmd = ClaudeCommand::new("/tmp", Session::New(uuid(11)));
        cmd.stop_check = true;
        cmd.layer_hooks.insert(
            "Stop".into(),
            serde_json::json!([{"hooks": [{"type": "command", "command": "layer-stop"}]}]),
        );
        cmd.layer_hooks.insert(
            "PreToolUse".into(),
            serde_json::json!([{"matcher": "Edit", "hooks": [{"type": "command", "command": "bridle arch-guard"}]}]),
        );
        let settings = settings_of(&cmd.args());
        let stop = settings["hooks"]["Stop"].as_array().expect("Stop array");
        assert_eq!(stop.len(), 2);
        assert_eq!(stop[0]["hooks"][0]["command"], STOP_CHECK_HOOK_COMMAND);
        assert_eq!(stop[1]["hooks"][0]["command"], "layer-stop");
        assert_eq!(
            settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "bridle arch-guard"
        );
    }

    #[test]
    fn layer_hooks_alone_create_the_hooks_object() {
        let mut cmd = ClaudeCommand::new("/tmp", Session::New(uuid(12)));
        cmd.layer_hooks
            .insert("PreToolUse".into(), serde_json::json!([{"hooks": []}]));
        assert!(settings_of(&cmd.args())["hooks"]["PreToolUse"].is_array());
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

    fn settings_of(args: &[String]) -> serde_json::Value {
        let i = args
            .iter()
            .position(|a| a == "--settings")
            .expect("--settings");
        serde_json::from_str(&args[i + 1]).expect("--settings is JSON")
    }

    #[test]
    fn tools_are_one_comma_joined_argument() {
        let mut cmd = ClaudeCommand::new("/tmp", Session::New(uuid(11)));
        cmd.tools = Some(vec!["Bash".into(), "Read".into(), "Agent".into()]);
        let args = cmd.args();
        let i = args.iter().position(|a| a == "--tools").expect("--tools");
        assert_eq!(args[i + 1], "Bash,Read,Agent");
        assert!(settings_of(&args).get("disableBundledSkills").is_none());
    }

    #[test]
    fn skill_in_tools_disables_bundled_skills() {
        let mut cmd = ClaudeCommand::new("/tmp", Session::New(uuid(12)));
        cmd.tools = Some(vec!["Bash".into(), "Skill".into()]);
        assert_eq!(settings_of(&cmd.args())["disableBundledSkills"], true);
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
            "--tools",
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
