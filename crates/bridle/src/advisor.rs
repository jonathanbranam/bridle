//! `bridle advisor start <name> [--brief]`: an advisor in a new tmux pane (ticket ervd). The brief
//! goes first, as a message to the shared advisor inbox naming the advisor; the session it
//! starts is `bridle session advisor <name>`, which signs the advisor's name and tags the pane.

use std::process::Command;

use bridle_api::{MessageKind, SendRequest, When};
use bridle_daemon::config::{AdvisorPane, advisor_pane};

use crate::cli::{AdvisorAction, Cli};
use crate::commands::client_for;
use crate::error::CliError;

pub async fn run(cli: &Cli, action: &AdvisorAction) -> Result<(), CliError> {
    let AdvisorAction::Start { name, brief } = action;
    // Orchestrator and human only: a worker has no business starting sessions. The orchestrator
    // and the human run without BRIDLE_AGENT_ID.
    if std::env::var_os("BRIDLE_AGENT_ID").is_some()
        && std::env::var_os("BRIDLE_LAUNCHER_TEST").is_none()
    {
        return Err(anyhow::anyhow!(
            "bridle advisor start: only the orchestrator and the human may start an advisor \
             (BRIDLE_AGENT_ID is set)"
        )
        .into());
    }
    crate::focus::refuse_advisor_if_locked(
        &bridle_api::discovery::bridle_home(),
        chrono::Local::now(),
    )?;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow::anyhow!(
            "advisor name {name:?}: use letters, digits, '-' and '_' only"
        )
        .into());
    }
    let mode = advisor_pane(&bridle_api::discovery::bridle_home()).map_err(anyhow::Error::from)?;
    if let Some(brief) = brief {
        let text = match brief.strip_prefix('@') {
            Some(path) => std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("reading the brief from {path}: {e}"))?,
            None => brief.clone(),
        };
        let client = client_for(cli).await?;
        client
            .send(&SendRequest {
                to: Some("external:advisor".into()),
                body: brief_body(name, &text),
                kind: MessageKind::Note,
                when: When::Now,
                reply_to: None,
                task: None,
            })
            .await?;
        println!("sent the brief to external:advisor for advisor {name}");
    }
    let command = session_command(crate::project::resolve(cli).ok().as_deref(), name);
    if std::env::var_os("TMUX").is_none() {
        println!("Not inside tmux. In a terminal, run:\n  {command}");
        return Ok(());
    }
    let panes = tmux_output(&["list-panes", "-a", "-F", "#{pane_id} #{@bridle}"]);
    let args = tmux_args(mode, panes.as_deref().unwrap_or(""), &command);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    tmux_output(&refs).map_err(|e| anyhow::anyhow!("tmux {}: {e}", args[0]))?;
    println!("started advisor {name} in a new tmux pane");
    Ok(())
}

fn brief_body(name: &str, text: &str) -> String {
    format!("For advisor {name}: {}", text.trim())
}

fn session_command(project: Option<&str>, name: &str) -> String {
    match project {
        Some(p) => format!("bridle session advisor --project {p} {name}"),
        None => format!("bridle session advisor {name}"),
    }
}

/// The tmux call that starts `command`: a split beside the orchestrator's pane (tagged
/// `@bridle=orchestrator`) or, when there is none or `window` is configured, a new window. `-d`
/// keeps the caller's focus where it is.
fn tmux_args(mode: AdvisorPane, panes: &str, command: &str) -> Vec<String> {
    let orch = panes.lines().find_map(|l| {
        let (id, tag) = l.split_once(' ')?;
        (tag.trim() == "orchestrator").then_some(id)
    });
    let v = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut args = match (mode, orch) {
        (AdvisorPane::Split, Some(pane)) => v(&["split-window", "-d", "-t", pane]),
        _ => v(&["new-window", "-d"]),
    };
    args.push(command.to_string());
    args
}

fn tmux_output(args: &[&str]) -> Result<String, String> {
    let out = Command::new("tmux")
        .args(args)
        .output()
        .map_err(|e| format!("running tmux: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brief_names_the_advisor() {
        assert_eq!(
            brief_body("fred", " pooling\n"),
            "For advisor fred: pooling"
        );
    }

    #[test]
    fn split_targets_the_orchestrator_pane() {
        let panes = "%1 \n%4 orchestrator\n%5 advisor-x\n";
        assert_eq!(
            tmux_args(AdvisorPane::Split, panes, "c"),
            ["split-window", "-d", "-t", "%4", "c"]
        );
        assert_eq!(
            tmux_args(AdvisorPane::Window, panes, "c"),
            ["new-window", "-d", "c"]
        );
        assert_eq!(
            tmux_args(AdvisorPane::Split, "%1 \n", "c"),
            ["new-window", "-d", "c"]
        );
    }
}
