//! The one place a command learns which project it acts on (br-3397): `--project`, then
//! `$BRIDLE_PROJECT` (clap folds both into `Cli::project`), then the project of the workspace
//! containing the cwd. No hard-coded fallback: with none of them, refuse and name `--project`.
//! Commands that reach a daemon get the same order from `discovery::resolve_endpoint`.

use std::path::Path;

use bridle_api::discovery::{find_workspace, list_registry, read_daemon_json};

use crate::cli::Cli;
use crate::error::CliError;

/// The project of the workspace containing `cwd`: its `.bridle/daemon.json` (walking up), else a
/// registered daemon whose workspace is an ancestor of `cwd`.
pub fn from_cwd(cwd: &Path) -> Option<String> {
    if let Some(info) = find_workspace(cwd).and_then(|w| read_daemon_json(&w).ok()) {
        return Some(info.project);
    }
    list_registry()
        .into_iter()
        .find(|d| cwd.starts_with(&d.workspace))
        .map(|d| d.project)
}

pub fn resolve_in(explicit: Option<&str>, cwd: &Path) -> Result<String, CliError> {
    explicit
        .map(str::to_string)
        .or_else(|| from_cwd(cwd))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no project: {} is not inside a bridle workspace. Pass --project <name> or set \
                 $BRIDLE_PROJECT (see `bridle daemons`).",
                cwd.display()
            )
            .into()
        })
}

pub fn resolve(cli: &Cli) -> Result<String, CliError> {
    let cwd = std::env::current_dir().map_err(anyhow::Error::from)?;
    resolve_in(cli.project.as_deref(), &cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// How each top-level command learns its project (br-3397, docs/design/cli.md "Project
    /// resolution"). A new command must be added here, which makes its author choose.
    enum Scope {
        /// Reaches a daemon: `discovery::resolve_endpoint` (`--url`, `$BRIDLE_URL`, then
        /// `--project`/`$BRIDLE_PROJECT`, then the cwd's workspace).
        Daemon,
        /// Needs the project's name itself: `project::resolve` or `launchd::project_name`.
        Resolver,
        /// Acts on no project, or on the cwd's repo files only; says why.
        NoProject(&'static str),
    }
    use Scope::*;

    const SCOPES: &[(&str, Scope)] = &[
        ("daemon", Daemon),
        ("agent", Daemon),
        (
            "hook",
            NoProject("Claude Code hooks: read the agent's own env and cwd"),
        ),
        (
            "serve",
            NoProject("creates the project: its name is the repo's"),
        ),
        (
            "docs",
            NoProject("static text embedded in the binary; no daemon"),
        ),
        ("gateway", NoProject("serves every project")),
        (
            "link",
            NoProject("config lookup; a ticket link takes --project or the cwd's workspace"),
        ),
        ("stop-daemon", Daemon),
        ("restart", Daemon),
        ("doctor", NoProject("checks the machine and the cwd's repo")),
        ("init", NoProject("creates a project in the cwd")),
        ("launchd", Resolver),
        ("systemd", Resolver),
        (
            "sign",
            NoProject("signs a binary with the machine's keychain identity"),
        ),
        ("rebuild", Daemon),
        ("daemons", NoProject("lists every project's daemon")),
        ("status", Daemon),
        ("ci", Daemon),
        ("spawn", Daemon),
        ("agents", Daemon),
        ("show", Daemon),
        ("send", Daemon),
        ("schedule", Daemon),
        ("inbox", Daemon),
        ("messages", Daemon),
        ("interrupt", Daemon),
        ("stop", Daemon),
        ("resume", Daemon),
        ("renew", Daemon),
        ("rm", Daemon),
        ("logs", Daemon),
        ("wake", Daemon),
        ("events", Daemon),
        ("wait", Daemon),
        ("usage", Daemon),
        ("cost", Daemon),
        ("review", Daemon),
        ("tui", Daemon),
        ("budget", Daemon),
        ("token", Daemon),
        ("task", Daemon),
        ("impact", Daemon),
        ("probe", Daemon),
        ("land", Daemon),
        ("push", Daemon),
        ("conflict", Daemon),
        ("port", Daemon),
        ("dep", Daemon),
        ("ask", Daemon),
        ("answer", Daemon),
        ("claim", Daemon),
        ("release", Daemon),
        ("ready", Daemon),
        ("queue", Daemon),
        ("report", Daemon),
        (
            "statusline",
            NoProject("display text from the statusline token"),
        ),
        ("stop-check", Daemon),
        ("arch-guard", NoProject("reads the cwd's repo")),
        ("kill-guard", NoProject("reads only the hook's stdin")),
        ("orchestrator", Daemon),
        ("focus", NoProject("the human's own focus gate")),
        ("wait-for-wake", Daemon),
        ("mail", Daemon),
        ("handover", Daemon),
        ("prime", Resolver),
        ("rules", NoProject("reads the cwd's repo config")),
        ("sync", NoProject("writes the cwd's repo files")),
        (
            "workflow",
            NoProject("vendors workflow files into the cwd's repo"),
        ),
        ("spec", NoProject("reads the cwd's repo specs")),
        ("goals", Daemon),
        ("ticket", Resolver),
        ("arch", NoProject("reads the cwd's repo")),
        ("explore", NoProject("scratch directories")),
        ("pane", NoProject("tmux")),
        ("machine", NoProject("machine setup")),
        ("session", Resolver),
        ("advisor", Resolver),
        ("trace", NoProject("reads the cwd's repo")),
        ("completions", NoProject("prints a shell script")),
        ("migrate", Daemon),
    ];

    #[test]
    fn every_command_declares_how_it_finds_its_project() {
        let cmd = Cli::command();
        let names: Vec<String> = cmd
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        for n in &names {
            assert!(
                SCOPES.iter().any(|(s, _)| s == n),
                "`bridle {n}` is new: add it to SCOPES in project.rs, resolving its project \
                 through project::resolve or discovery (--project, $BRIDLE_PROJECT, cwd; no default)"
            );
        }
        for (s, scope) in SCOPES {
            assert!(names.iter().any(|n| n == s), "SCOPES lists unknown `{s}`");
            if let NoProject(why) = scope {
                assert!(!why.is_empty());
            }
        }
        // The project flag is global, so every command accepts it.
        assert!(
            cmd.get_arguments()
                .any(|a| a.get_id() == "project" && a.is_global_set())
        );
    }

    #[test]
    fn no_command_defaults_to_a_hard_coded_project() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![src];
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let text = std::fs::read_to_string(&p).unwrap();
                let prod = text.split("#[cfg(test)]").next().unwrap();
                for pat in [
                    r#"unwrap_or("bridle")"#,
                    r#"|| "bridle".into()"#,
                    r#"|| "bridle".to_string()"#,
                    r#"|| String::from("bridle")"#,
                ] {
                    assert!(
                        !prod.contains(pat),
                        "{}: hard-coded project `{pat}`",
                        p.display()
                    );
                }
            }
        }
    }

    #[test]
    fn explicit_wins_then_cwd_workspace_then_error() {
        let ws = tempfile::tempdir().unwrap();
        let sub = ws.path().join("clone/deep");
        std::fs::create_dir_all(&sub).unwrap();
        assert!(resolve_in(None, &sub).is_err());
        bridle_api::discovery::write_daemon_json(
            ws.path(),
            &bridle_api::DaemonInfo {
                project: "x".into(),
                workspace: ws.path().to_string_lossy().into_owned(),
                repo: String::new(),
                url: "http://127.0.0.1:1".into(),
                pid: 1,
                started_at: chrono::Utc::now(),
                version: String::new(),
            },
        )
        .unwrap();
        assert_eq!(resolve_in(None, &sub).unwrap(), "x");
        assert_eq!(resolve_in(Some("y"), &sub).unwrap(), "y");
    }
}
