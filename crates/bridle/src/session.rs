//! `bridle session orchestrator|advisor|aide`: starts the role's `claude` session (ticket mrhe). Ports
//! `scripts/claude-orchestrator` and `scripts/claude-advisor`, which are now wrappers around it.
//! No `exec`: bridle stays as the parent so the pid file can be removed and the exit recorded.

use std::io::Write as _;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
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

const AIDE_PROMPT: &str = "Run `bridle prime aide` and follow what it prints.";

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

/// Merges the workflow layers' hooks (the spawn path's overlay, br-01a4) into a session's
/// `--settings` JSON: per event the arrays concatenate, bridle's own first, so a layer can't
/// drop the focus gate or the reply hook. This is also how the base `UserPromptSubmit` time
/// stamp reaches the sessions where the human types.
fn with_layer_hooks(
    settings: &str,
    layer_hooks: &std::collections::BTreeMap<String, serde_json::Value>,
) -> String {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(settings) else {
        return settings.to_string();
    };
    for (event, entries) in layer_hooks {
        let Some(entries) = entries.as_array() else {
            continue;
        };
        if let Some(own) = v["hooks"][event.as_str()].as_array_mut() {
            own.extend(entries.iter().cloned());
        } else {
            v["hooks"][event.as_str()] = serde_json::Value::Array(entries.clone());
        }
    }
    v.to_string()
}

/// The layer hooks for the project in the current directory; none (with a warning) when its
/// config can't be loaded, so a session never fails to start over them.
fn session_layer_hooks() -> std::collections::BTreeMap<String, serde_json::Value> {
    let load = || -> anyhow::Result<_> {
        let repo = std::env::current_dir()?;
        let config = bridle_daemon::config::Config::load(&repo)?;
        Ok(config.layer_hooks(&repo))
    };
    load().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "no layer hooks for this session");
        Default::default()
    })
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
    if let SessionRole::Keep { identifier } = role {
        let client = crate::commands::client_for(cli).await?;
        client.session_keep(&session_identity(identifier)).await?;
        println!("{identifier} will carry on; it is asked again at the next step");
        return Ok(());
    }
    if let SessionRole::Restart {
        identifier, fresh, ..
    } = role
    {
        return restart(cli, identifier, *fresh).await;
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
    let layer_hooks = session_layer_hooks();
    let code = match role {
        SessionRole::Orchestrator { claude_args: extra } => {
            let name = session_name("orch", None, &project, &suffix);
            let args = claude_args(
                &with_layer_hooks(&orchestrator_settings(), &layer_hooks),
                &name,
                extra,
                ORCHESTRATOR_PROMPT,
            );
            orchestrator(&home, &project, &args).await?
        }
        SessionRole::Advisor { args } => {
            let (adv, extra) = match args.split_first() {
                Some((n, rest)) => (Some(n.as_str()), rest),
                None => (None, &[][..]),
            };
            crate::focus::refuse_advisor_if_locked(&home, chrono::Local::now())?;
            let name = session_name("advisor", adv, &project, &suffix);
            let mut prompt = advisor_prompt(adv);
            if let Some(path) = take_handover(&home, &advisor_identity(adv)) {
                prompt = format!(
                    "{prompt} Your previous session left a handover note at {}: read it first.",
                    path.display()
                );
            }
            let args = claude_args(
                &with_layer_hooks(&advisor_settings(), &layer_hooks),
                &name,
                extra,
                &prompt,
            );
            advisor(cli, &home, &project, adv, &args).await?
        }
        SessionRole::Aide { claude_args: extra } => {
            crate::focus::refuse_advisor_if_locked(&home, chrono::Local::now())?;
            let name = session_name("aide", None, &project, &suffix);
            let mut prompt = AIDE_PROMPT.to_string();
            if let Some(path) = take_handover(&home, "aide") {
                prompt = format!(
                    "{prompt} Your previous session left a handover note at {}: read it first.",
                    path.display()
                );
            }
            let args = claude_args(
                &with_layer_hooks(&advisor_settings(), &layer_hooks),
                &name,
                extra,
                &prompt,
            );
            aide(cli, &project, &args).await?
        }
        SessionRole::Note | SessionRole::Restart { .. } | SessionRole::Keep { .. } => {
            unreachable!("handled above")
        }
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

/// Aide is one session per project, signed `external:aide` through `BRIDLE_AS`. It is
/// registered with the daemon like an advisor (context warnings, restart) but keeps no pid file.
async fn aide(cli: &Cli, project: &str, args: &[String]) -> anyhow::Result<i32> {
    let pid = std::process::id().to_string();
    crate::pane::tag_pane("aide");
    register(cli, std::process::id() as i32, "aide", None).await;
    let rc = run_claude(
        &[
            ("BRIDLE_AS", "aide"),
            ("BRIDLE_PROJECT", project),
            ("BRIDLE_SESSION_PID", pid.as_str()),
        ],
        args,
    )
    .await;
    end(cli).await;
    rc
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
    register(
        cli,
        std::process::id() as i32,
        &advisor_identity(name),
        None,
    )
    .await;
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
async fn register(cli: &Cli, pid: i32, identity: &str, claude_session_id: Option<String>) {
    let Some(pid_start) = process_start(pid) else {
        return;
    };
    let req = bridle_api::types::SessionRegister {
        identity: identity.to_string(),
        pid,
        pid_start,
        pane: std::env::var("TMUX_PANE").ok(),
        claude_session_id,
        project: std::env::var("BRIDLE_PROJECT").ok(),
        machine: hostname(),
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
    let identity = if std::env::var("BRIDLE_AS").is_ok_and(|a| a == "aide") {
        "aide".to_string()
    } else {
        advisor_identity(std::env::var("BRIDLE_ADVISOR_NAME").ok().as_deref())
    };
    register(cli, pid, &identity, Some(id)).await;
}

/// A session as the human names it: `alice` is `advisor/alice`; `advisor`, `aide` and anything
/// with a `/` stand as they are.
fn session_identity(identifier: &str) -> String {
    if identifier.contains('/') || matches!(identifier, "advisor" | "aide") {
        identifier.to_string()
    } else {
        format!("advisor/{identifier}")
    }
}

fn advisor_identity(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("advisor/{n}"),
        None => "advisor".into(),
    }
}

fn hostname() -> Option<String> {
    let out = Command::new("hostname").output().ok()?;
    let h = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!h.is_empty()).then_some(h)
}

/// Where a restart's handover note goes: `$BRIDLE_HOME/handover/<identity, '/' as '-'>.md`.
fn handover_path(home: &Path, identity: &str) -> PathBuf {
    home.join("handover")
        .join(format!("{}.md", identity.replace('/', "-")))
}

/// The handover note a restart left for this session, renamed `.read` so a later plain launch
/// doesn't read it again. `None` when there isn't one.
fn take_handover(home: &Path, identity: &str) -> Option<PathBuf> {
    let path = handover_path(home, identity);
    if !path.is_file() {
        return None;
    }
    let read = path.with_extension("md.read");
    std::fs::rename(&path, &read).ok()?;
    Some(read)
}

/// How long `--handover` waits for the session to write its note.
fn handover_wait() -> std::time::Duration {
    let secs = std::env::var("BRIDLE_HANDOVER_WAIT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(600);
    std::time::Duration::from_secs(secs)
}

/// The command that starts the session again.
fn relaunch_command(identity: &str, project: Option<&str>) -> String {
    let mut c = String::from("bridle");
    if let Some(p) = project {
        c.push_str(&format!(" --project {p}"));
    }
    if identity == "aide" {
        c.push_str(" session aide");
        return c;
    }
    c.push_str(" session advisor");
    if let Some(n) = identity.strip_prefix("advisor/") {
        c.push(' ');
        c.push_str(n);
    }
    c
}

/// `bridle session restart`: stop the session's claude and start the launcher again in its tmux
/// pane. A handover asks the session to write a note first. Restarting with no handover is
/// the human's choice: refused for a session or agent (BRIDLE_AS / BRIDLE_AGENT_ID).
async fn restart(cli: &Cli, identifier: &str, fresh: bool) -> Result<(), CliError> {
    let test = std::env::var_os("BRIDLE_LAUNCHER_TEST").is_some();
    if std::env::var_os("BRIDLE_AGENT_ID").is_some() && !test {
        return Err(anyhow::anyhow!(
            "bridle session restart: refusing under a bridle agent (BRIDLE_AGENT_ID is set)"
        )
        .into());
    }
    if fresh && std::env::var_os("BRIDLE_AS").is_some() {
        return Err(anyhow::anyhow!(
            "bridle session restart --fresh is the human's choice: run it from your own terminal"
        )
        .into());
    }
    let client = crate::commands::client_for(cli).await?;
    let sessions = client.sessions().await?;
    let wanted = session_identity(identifier);
    let info = sessions
        .iter()
        .find(|s| s.identity == wanted)
        .ok_or_else(|| anyhow::anyhow!("no running session {wanted:?} (see `bridle status`)"))?;
    let home = bridle_home();
    let note = handover_path(&home, &info.identity);
    let _ = std::fs::remove_file(&note); // a stale note is not this restart's
    if !fresh {
        std::fs::create_dir_all(note.parent().expect("handover dir"))
            .map_err(anyhow::Error::from)?;
        let to = format!("external:{}", info.identity);
        let body = format!(
            "The human is restarting this session. Write a handover note (what you were doing,              open threads, what the next session needs) to {} and say nothing more; the              restart follows when the file appears.",
            note.display()
        );
        client
            .send(&bridle_api::SendRequest {
                to: Some(to),
                body,
                kind: bridle_api::MessageKind::Note,
                when: bridle_api::When::Now,
                reply_to: None,
                task: None,
            })
            .await?;
        println!(
            "asked {} to write {}; waiting",
            info.identity,
            note.display()
        );
        let deadline = std::time::Instant::now() + handover_wait();
        while !note.metadata().is_ok_and(|m| m.len() > 0) {
            if std::time::Instant::now() >= deadline {
                return Err(anyhow::anyhow!(
                    "no handover note after {}s; nothing restarted. Use --fresh to restart with                      no context",
                    handover_wait().as_secs()
                )
                .into());
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
    stop_session(info.pid).await;
    let cmd = relaunch_command(&info.identity, info.project.as_deref());
    match &info.pane {
        Some(pane) => {
            let ok = Command::new("tmux")
                .args(["send-keys", "-t", pane, &cmd, "Enter"])
                .status()
                .is_ok_and(|s| s.success());
            if ok {
                println!("restarted {} in pane {pane}", info.identity);
            } else {
                println!("pane {pane} is gone; run in a terminal: {cmd}");
            }
        }
        None => println!(
            "{} had no tmux pane; run in a terminal: {cmd}",
            info.identity
        ),
    }
    Ok(())
}

/// SIGTERM to the launcher's children (claude), then wait for the launcher to exit so the pane
/// is back at its shell. Only this pid's own children, never by name (no-kill-by-name).
async fn stop_session(pid: i32) {
    let kids = Command::new("pgrep")
        .args(["-P", &pid.to_string()])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    for kid in kids.split_whitespace() {
        let _ = Command::new("kill").args(["-TERM", kid]).status();
    }
    for _ in 0..40 {
        let alive = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !alive {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
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
    fn relaunch_names_the_advisor_and_project() {
        assert_eq!(
            relaunch_command("advisor/alice", Some("meta")),
            "bridle --project meta session advisor alice"
        );
        assert_eq!(relaunch_command("advisor", None), "bridle session advisor");
    }

    #[test]
    fn a_handover_note_is_read_once() {
        let home = tempfile::tempdir().unwrap();
        let p = handover_path(home.path(), "advisor/alice");
        assert!(p.ends_with("handover/advisor-alice.md"));
        assert!(take_handover(home.path(), "advisor/alice").is_none());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "state").unwrap();
        let read = take_handover(home.path(), "advisor/alice").unwrap();
        assert!(read.is_file() && !p.exists());
        assert!(take_handover(home.path(), "advisor/alice").is_none());
    }

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
    fn layer_hooks_merge_after_bridles_own() {
        let mut layer = std::collections::BTreeMap::new();
        layer.insert(
            "UserPromptSubmit".to_string(),
            serde_json::json!([{"hooks":[{"type":"command","command":"date"}]}]),
        );
        layer.insert(
            "PreToolUse".to_string(),
            serde_json::json!([{"hooks":[{"type":"command","command":"guard"}]}]),
        );
        let v: serde_json::Value =
            serde_json::from_str(&with_layer_hooks(&advisor_settings(), &layer)).expect("json");
        let ups = v["hooks"]["UserPromptSubmit"].as_array().expect("array");
        assert_eq!(ups.len(), 2);
        assert_eq!(ups[0]["hooks"][0]["command"], "bridle focus gate");
        assert_eq!(ups[1]["hooks"][0]["command"], "date");
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "guard");
        assert!(v["hooks"]["Stop"].is_array() && v["hooks"]["SessionStart"].is_array());
        assert!(v["permissions"]["deny"].is_array());
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
