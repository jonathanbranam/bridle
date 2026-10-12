//! `bridle project drain` (kx5u K4): the step before `project stop`. Holds new claims and spawns
//! for this project's daemon only, tells every principal to finish, then waits with no timeout
//! until the stable-project check (K3) passes. The orchestrator is messaged at the start and on
//! each change, so it works the drain with the human; a drain that stops changing is named to
//! the orchestrator and to the human through aide. `--undo` lifts the hold.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Context;
use bridle_api::Client;
use bridle_api::discovery::list_registry;
use bridle_api::types::{MessageKind, SendRequest, When};
use bridle_daemon::config::Config;
use bridle_daemon::paths::Workspace;

use crate::cli::{Cli, ProjectDrainArgs};
use crate::commands::client_for;
use crate::error::CliError;

const POLL: Duration = Duration::from_secs(5);
/// How long the list may stay the same before it is called stuck (and again each time after).
const STUCK_AFTER: Duration = Duration::from_secs(30 * 60);

pub async fn run(cli: &Cli, args: &ProjectDrainArgs) -> Result<(), CliError> {
    let running = cli
        .project
        .as_deref()
        .and_then(|p| list_registry().into_iter().find(|d| d.project == p));
    let repo = match (&args.repo, &running) {
        (Some(r), _) => r.clone(),
        (None, Some(d)) => PathBuf::from(&d.repo),
        (None, None) => std::env::current_dir().context("current directory")?,
    }
    .canonicalize()
    .context("repo path")?;
    let client = client_for(cli).await?;
    if args.undo {
        client.hold_project(false).await?;
        println!("hold lifted: the project takes new work again");
        return Ok(());
    }
    let integration = Config::load(&repo)
        .map(|c| c.branches.integration)
        .unwrap_or_else(|_| "main".into());
    let db = Workspace::new(repo.clone(), None).db();

    client.hold_project(true).await?;
    let status = client.status().await?;
    let project = status.daemon.project.clone();
    println!("{project}: held; no new claims or spawns. Announcing, then waiting (no timeout).");
    announce(&client, &project, &recipients(&client).await?).await;

    let mut last: Vec<String> = Vec::new();
    let mut since = Instant::now();
    let mut told_stuck = false;
    let mut first = true;
    loop {
        // A daemon restart loses the hold; take it again rather than let new work in.
        if !client.status().await?.held {
            client.hold_project(true).await?;
        }
        let (bad, _left) = crate::stable::unstable(&repo, &db, &integration, &args.leave, true);
        if bad.is_empty() {
            println!("{project}: stable. Run `bridle project stop`.");
            tell(
                &client,
                "orchestrator",
                format!("drain of {project}: stable; the project can be stopped."),
            )
            .await;
            return Ok(());
        }
        if first || bad != last {
            println!("waiting on:");
            for b in &bad {
                println!("  {b}");
            }
            tell(
                &client,
                "orchestrator",
                format!(
                    "drain of {project}: {} item(s) left:\n{}",
                    bad.len(),
                    bad.join("\n")
                ),
            )
            .await;
            last = bad;
            since = Instant::now();
            told_stuck = false;
            first = false;
        } else if since.elapsed() >= STUCK_AFTER && !told_stuck {
            told_stuck = true;
            let msg = format!(
                "drain of {project} looks stuck: unchanged for {} min:\n{}",
                since.elapsed().as_secs() / 60,
                last.join("\n")
            );
            println!("{msg}");
            tell(&client, "orchestrator", msg.clone()).await;
            tell(&client, "aide", msg).await;
        }
        tokio::time::sleep(POLL).await;
    }
}

/// Everyone the move concerns, once each: running agents by name, registered sessions by identity.
async fn recipients(client: &Client) -> Result<Vec<String>, CliError> {
    let agents = client.list_agents().await?;
    let status = client.status().await?;
    Ok(recipient_names(
        agents
            .iter()
            .filter(|a| a.state.is_running())
            .map(|a| a.name.as_str()),
        status.sessions.iter().map(|s| s.identity.as_str()),
    ))
}

fn recipient_names<'a>(
    agents: impl Iterator<Item = &'a str>,
    sessions: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    agents
        .chain(sessions)
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

async fn announce(client: &Client, project: &str, to: &[String]) {
    let body = format!(
        "{project} is moving to another machine. Finish your current work and start nothing new; \
         workers: report and stop. Interactive roles: write your handover note in the repo. \
         The manager lands everything (full checks, merged to main) or drops it."
    );
    for who in to {
        tell(client, who, body.clone()).await;
    }
}

/// Best effort: a recipient that is not there must not stop the drain.
async fn tell(client: &Client, to: &str, body: String) {
    let req = SendRequest {
        to: Some(to.to_string()),
        body,
        kind: MessageKind::Note,
        when: When::Idle,
        reply_to: None,
        task: None,
    };
    if let Err(e) = client.send(&req).await {
        eprintln!("(could not tell {to}: {e})");
    }
}

#[cfg(test)]
mod tests {
    use super::recipient_names;

    #[test]
    fn each_principal_is_named_once() {
        let got = recipient_names(
            ["w1", "mgr", "w1"].into_iter(),
            ["external:orchestrator", "mgr"].into_iter(),
        );
        assert_eq!(got, ["external:orchestrator", "mgr", "w1"]);
    }
}
