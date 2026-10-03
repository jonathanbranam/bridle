//! `bridle session orchestrator|advisor|triage`: starts the role's `claude` session (ticket mrhe). Ports
//! `scripts/claude-orchestrator` and `scripts/claude-advisor`, which are now wrappers around it.
//! No `exec`: bridle stays as the parent so the pid file can be removed and the exit recorded.

use std::io::Write as _;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;

use anyhow::Context;
use bridle_api::discovery::bridle_home;

use crate::cli::{Cli, SessionRole};
use crate::error::CliError;

/// Lean start (ticket ct8m, docs/spikes/08-lean-context-findings.md): drops bundled skills,
/// workflows and the claude.ai connectors, and denies tools these roles never use. Not
/// disableRemoteControl: the human reaches the session through Remote Control.
const LEAN: &str = r#""disableBundledSkills":true,"disableWorkflows":true,"disableClaudeAiConnectors":true,"disableArtifact":true,"permissions":{"deny":["EnterPlanMode","ExitPlanMode","DesignSync","NotebookEdit","PushNotification","ReportFindings","RemoteTrigger","Artifact","Workflow","Edit(~/.bridle/focus*)","Edit(~/.bridle/config.toml)"]}"#;

/// Focus hours (ticket cvaq): a no-op unless `~/.bridle/config.toml` has `[[focus]]`.
const FOCUS_GATE: &str =
    r#""UserPromptSubmit":[{"hooks":[{"type":"command","command":"bridle focus gate"}]}]"#;

/// The Stop hook records when the agent finishes replying, the other end of the human's reading
/// time (ticket u6w9).
const REPLY_HOOK: &str =
    r#""Stop":[{"hooks":[{"type":"command","command":"bridle focus reply"}]}]"#;

/// The SessionStart hook records the session id (it changes on /clear). One --settings object:
/// a second would replace it.
fn orchestrator_settings() -> String {
    format!(
        r#"{{"hooks":{{"SessionStart":[{{"hooks":[{{"type":"command","command":"bridle orchestrator note-session"}}]}}],{FOCUS_GATE},{REPLY_HOOK}}},{LEAN}}}"#
    )
}

/// The advisor's SessionStart hook tells the daemon the Claude session id (it changes on /clear)
/// so it can read the session's context (jttf).
const ADVISOR_NOTE: &str =
    r#""SessionStart":[{"hooks":[{"type":"command","command":"bridle session note"}]}]"#;

fn advisor_settings() -> String {
    format!(r#"{{"hooks":{{{ADVISOR_NOTE},{FOCUS_GATE},{REPLY_HOOK}}},{LEAN}}}"#)
}

// The opening prompts only point at `bridle prime`: a long prompt in argv is matched by any
// `pkill -f <pattern>` a worker runs (fx7x).
const ORCHESTRATOR_PROMPT: &str = "Run `bridle prime orchestrator` and follow what it prints.";

const TRIAGE_PROMPT: &str = "Run `bridle prime triage` and follow what it prints.";

fn advisor_prompt(name: Option<&str>) -> String {
    let base = "Run `bridle prime advisor` and follow what it prints. Say hello to the human in one line, then wait.";
    match name {
        Some(n) => format!(
            "You are advisor {n}: first read your unread inbox messages (bridle inbox --json) starting \"For advisor {n}:\" and start from that brief. Then run `bridle prime advisor` and follow what it prints. Say hello to the human in one line, then wait."
        ),
        None => base.to_string(),
    }
}

/// 'orch-<project>' or 'advisor[-<name>]-<project>', plus '-<suffix>' when `suffix` is non-empty.
fn session_name(role: &str, name: Option<&str>, project: &str, suffix: &str) -> String {
    let mut s = role.to_string();
    if let Some(n) = name {
        s.push('-');
        s.push_str(n);
    }
    s.push('-');
    s.push_str(project);
    if !suffix.is_empty() {
        s.push('-');
        s.push_str(suffix);
    }
    s
}

fn claude_args(settings: &str, name: &str, extra: &[String], prompt: &str) -> Vec<String> {
    let mut a: Vec<String> = [
        "--settings",
        settings,
        "--strict-mcp-config",
        "--name",
        name,
    ]
    .into_iter()
    .map(String::from)
    .collect();
    a.extend(["--remote-control".into(), name.into()]);
    a.extend(extra.iter().cloned());
    a.push(prompt.into());
    a
}

pub async fn run(cli: &Cli, role: &SessionRole) -> Result<(), CliError> {
    if matches!(role, SessionRole::Note) {
        note(cli).await;
        return Ok(());
    }
    // Never under a bridle agent (k6b3): a worker's test run once overwrote the live
    // orchestrator's pid/session files and used its Remote Control name.
    if std::env::var_os("BRIDLE_AGENT_ID").is_some()
        && std::env::var_os("BRIDLE_LAUNCHER_TEST").is_none()
    {
        return Err(anyhow::anyhow!(
            "bridle session: refusing to start under a bridle agent (BRIDLE_AGENT_ID is set). Test \
             with a stub claude on PATH, a temporary BRIDLE_HOME and BRIDLE_LAUNCHER_TEST=1."
        )
        .into());
    }
    // A tools-only clone (hw6c) is not the project's home.
    crate::tools_only::check_here()?;
    let project = cli.project.clone().unwrap_or_else(|| "bridle".into());
    let suffix = std::env::var("BRIDLE_SESSION_SUFFIX").unwrap_or_default();
    let home = bridle_home();
    let code = match role {
        SessionRole::Orchestrator { claude_args: extra } => {
            let name = session_name("orch", None, &project, &suffix);
            let args = claude_args(&orchestrator_settings(), &name, extra, ORCHESTRATOR_PROMPT);
            orchestrator(&home, &project, &args).await?
        }
        SessionRole::Advisor { args } => {
            let (adv, extra) = match args.split_first() {
                Some((n, rest)) => (Some(n.as_str()), rest),
                None => (None, &[][..]),
            };
            crate::focus::refuse_advisor_if_locked(&home, chrono::Local::now())?;
            let name = session_name("advisor", adv, &project, &suffix);
            let prompt = advisor_prompt(adv);
            let args = claude_args(&advisor_settings(), &name, extra, &prompt);
            advisor(cli, &home, &project, adv, &args).await?
        }
        SessionRole::Triage { claude_args: extra } => {
            crate::focus::refuse_advisor_if_locked(&home, chrono::Local::now())?;
            let name = session_name("triage", None, &project, &suffix);
            let args = claude_args(&advisor_settings(), &name, extra, TRIAGE_PROMPT);
            triage(&project, &args).await?
        }
        SessionRole::Note => unreachable!("handled above"),
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// Tags the pane so the daemon's supervisor (or the human) can find it. Best effort.
fn tag_pane(tag: &str) {
    let Ok(pane) = std::env::var("TMUX_PANE") else {
        return;
    };
    let _ = Command::new("tmux")
        .args(["set-option", "-p", "-t", &pane, "@bridle", tag])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// "<pid> <process start time> <launch epoch>", which the daemon's supervisor and `bridle mail
/// run` read. The pid is bridle's, which lives exactly as long as claude. The start time is `ps`
/// lstart, the same identity check the daemon uses for agents.
fn write_pid_file(path: &Path) -> anyhow::Result<()> {
    let pid = std::process::id();
    let out = Command::new("ps")
        .args(["-o", "lstart=", "-p", &pid.to_string()])
        .output()
        .context("running ps")?;
    let lstart = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let now = chrono::Utc::now().timestamp();
    std::fs::write(path, format!("{pid} {lstart} {now}\n"))
        .with_context(|| format!("writing {}", path.display()))
}

/// Runs claude in the foreground and returns its exit code (128 + signal when killed). SIGINT is
/// left to claude: the terminal sends it to us too, and we outlive it to record the exit.
async fn run_claude(env: &[(&str, &str)], args: &[String]) -> anyhow::Result<i32> {
    let _int = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .context("installing SIGINT handler")?;
    let mut cmd = tokio::process::Command::new("claude");
    cmd.args(args).envs(env.iter().copied());
    let status = cmd.status().await.context("running claude")?;
    Ok(status
        .code()
        .or_else(|| status.signal().map(|s| 128 + s))
        .unwrap_or(1))
}

async fn orchestrator(home: &Path, project: &str, args: &[String]) -> anyhow::Result<i32> {
    std::fs::create_dir_all(home)?;
    write_pid_file(&home.join("orchestrator.pid"))?;
    tag_pane("orchestrator");
    let env = [("BRIDLE_AS", "orchestrator"), ("BRIDLE_PROJECT", project)];
    let rc = run_claude(&env, args).await?;
    // A killed claude can't restore the pane's terminal modes; left on they make the next
    // session's input fill with focus reports (csfe). Off: focus reporting, bracketed paste,
    // mouse; pop kitty keys.
    print!("\x1b[?1004l\x1b[?2004l\x1b[?1000l\x1b[?1006l\x1b[>4;0m\x1b[<u");
    // Sessions have ended unattended with no trace (fx7x): record when, and how.
    let now = chrono::Utc::now();
    let line = format!(
        "{} {} orchestrator claude ended: {}",
        now.format("%FT%TZ"),
        chrono::Local::now().format("%-I:%M:%S %p %Z"),
        how_ended(rc)
    );
    println!("{line}");
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.join("orchestrator.exits"))?;
    writeln!(f, "{line}")?;
    Ok(rc)
}

fn how_ended(rc: i32) -> String {
    if rc <= 128 {
        return format!("exit {rc}");
    }
    let sig = rc - 128;
    let name = match sig {
        1 => "HUP".into(),
        2 => "INT".into(),
        3 => "QUIT".into(),
        6 => "ABRT".into(),
        9 => "KILL".into(),
        15 => "TERM".into(),
        n => n.to_string(),
    };
    format!("exit {rc} (SIG{name})")
}

/// Triage is one session per project, signed `external:triage` through `BRIDLE_AS`. It isn't
/// registered with the daemon (that is the advisors' session list) and keeps no pid file.
async fn triage(project: &str, args: &[String]) -> anyhow::Result<i32> {
    crate::pane::tag_pane("triage");
    run_claude(
        &[("BRIDLE_AS", "triage"), ("BRIDLE_PROJECT", project)],
        args,
    )
    .await
}

async fn advisor(
    cli: &Cli,
    home: &Path,
    project: &str,
    name: Option<&str>,
    args: &[String],
) -> anyhow::Result<i32> {
    let pid = std::process::id().to_string();
    let mut env = vec![
        ("BRIDLE_AS", "advisor"),
        ("BRIDLE_PROJECT", project),
        ("BRIDLE_SESSION_PID", pid.as_str()),
    ];
    if let Some(n) = name {
        env.push(("BRIDLE_ADVISOR_NAME", n));
    }
    // Only the unnamed advisor records its pid, so `bridle mail run` sends mail to it only while
    // it is alive (rs7p); a stale file is harmless, the start time is checked.
    let pid_file = home.join(format!("advisor-{project}.pid"));
    if name.is_none() {
        std::fs::create_dir_all(home)?;
        write_pid_file(&pid_file)?;
    }
    crate::pane::tag_pane(&match name {
        Some(n) => format!("advisor-{n}"),
        None => "advisor".into(),
    });
    register(cli, std::process::id() as i32, name, None).await;
    let rc = run_claude(&env, args).await;
    end(cli).await;
    if name.is_none() {
        let _ = std::fs::remove_file(&pid_file);
    }
    rc
}

/// The daemon's wait for a session call: a daemon that is down or slow never holds a session up.
const DAEMON_WAIT: std::time::Duration = std::time::Duration::from_secs(3);

/// Registers this launcher's session with the daemon, or adds the Claude session id to it.
/// Best effort: every failure is dropped.
async fn register(cli: &Cli, pid: i32, name: Option<&str>, claude_session_id: Option<String>) {
    let Some(pid_start) = process_start(pid) else {
        return;
    };
    let req = bridle_api::types::SessionRegister {
        identity: match name {
            Some(n) => format!("advisor/{n}"),
            None => "advisor".into(),
        },
        pid,
        pid_start,
        pane: std::env::var("TMUX_PANE").ok(),
        claude_session_id,
    };
    let _ = tokio::time::timeout(DAEMON_WAIT, async {
        crate::commands::client_for(cli)
            .await?
            .session_register(&req)
            .await
            .map_err(CliError::from)
    })
    .await;
}

async fn end(cli: &Cli) {
    let pid = std::process::id() as i32;
    let _ = tokio::time::timeout(DAEMON_WAIT, async {
        crate::commands::client_for(cli)
            .await?
            .session_end(pid)
            .await
            .map_err(CliError::from)
    })
    .await;
}

/// `bridle session note`, run by claude's SessionStart hook inside an advisor session: the
/// launcher's pid comes in `BRIDLE_SESSION_PID`.
async fn note(cli: &Cli) {
    let Some(pid) = std::env::var("BRIDLE_SESSION_PID")
        .ok()
        .and_then(|p| p.parse::<i32>().ok())
    else {
        return;
    };
    let id = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("session_id")?.as_str().map(String::from));
    let Some(id) = id.filter(|s| !s.is_empty()) else {
        return;
    };
    let name = std::env::var("BRIDLE_ADVISOR_NAME").ok();
    register(cli, pid, name.as_deref(), Some(id)).await;
}

/// `ps` lstart of `pid`, the identity check the daemon uses for processes.
fn process_start(pid: i32) -> Option<String> {
    let out = Command::new("ps")
        .args(["-o", "lstart=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!s.is_empty()).then_some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_project_and_suffix() {
        assert_eq!(session_name("orch", None, "bridle", ""), "orch-bridle");
        assert_eq!(
            session_name("orch", None, "meta-notes", "nuc"),
            "orch-meta-notes-nuc"
        );
        assert_eq!(
            session_name("advisor", Some("alice"), "meta-notes", "nuc"),
            "advisor-alice-meta-notes-nuc"
        );
        assert_eq!(session_name("advisor", None, "p", ""), "advisor-p");
    }

    #[test]
    fn settings_are_valid_json() {
        for s in [orchestrator_settings(), advisor_settings()] {
            serde_json::from_str::<serde_json::Value>(&s).expect("json");
        }
    }

    #[test]
    fn session_settings_have_the_reply_hook() {
        for s in [orchestrator_settings(), advisor_settings()] {
            assert!(s.contains("bridle focus reply"));
        }
    }

    #[test]
    fn lean_settings_deny_edit_focus_files_not_write() {
        for s in [orchestrator_settings(), advisor_settings()] {
            assert!(
                s.contains("\"Edit(~/.bridle/focus*)\""),
                "Edit(~/.bridle/focus*) should be denied"
            );
            assert!(
                s.contains("\"Edit(~/.bridle/config.toml)\""),
                "Edit(~/.bridle/config.toml) should be denied"
            );
            assert!(
                !s.contains("\"Write(~/.bridle/focus*)\""),
                "Write(~/.bridle/focus*) should not be denied"
            );
            assert!(
                !s.contains("\"Write(~/.bridle/config.toml)\""),
                "Write(~/.bridle/config.toml) should not be denied"
            );
        }
    }

    #[test]
    fn signals_are_named() {
        assert_eq!(how_ended(0), "exit 0");
        assert_eq!(how_ended(143), "exit 143 (SIGTERM)");
    }

    #[test]
    fn advisor_prompt_with_name_includes_name_and_inbox_instruction() {
        let prompt = advisor_prompt(Some("alice"));
        assert!(
            prompt.contains("You are advisor alice"),
            "prompt should contain advisor name"
        );
        assert!(
            prompt.contains("bridle inbox --json"),
            "prompt should instruct reading inbox"
        );
        assert!(
            prompt.contains("For advisor alice:"),
            "prompt should filter for messages to this advisor"
        );
    }

    #[test]
    fn advisor_prompt_without_name_is_unchanged() {
        let prompt = advisor_prompt(None);
        let expected = "Run `bridle prime advisor` and follow what it prints. Say hello to the human in one line, then wait.";
        assert_eq!(prompt, expected);
    }
}
