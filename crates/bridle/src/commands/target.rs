//! Which daemon a command talks to, and with which token: discovery's order
//! (docs/design/agent-host/daemon.md) and principals.md's, plus the one case they can't settle
//! without asking a daemon: a typed `--project` under `$BRIDLE_URL` (br-5akm).

use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use bridle_api::Client;
use bridle_api::discovery::{self, Endpoint, Env, ProcessEnv};

use crate::cli::Cli;
use crate::error::CliError;

pub(super) struct Target {
    pub endpoint: Endpoint,
    /// `--project` took the command away from `$BRIDLE_URL`'s daemon, so `$BRIDLE_TOKEN`
    /// (that daemon's token) must not go with it.
    redirected: bool,
}

pub(super) async fn resolve(cli: &Cli) -> Result<Target, CliError> {
    let cwd = std::env::current_dir().context("current directory")?;
    if cli.url.is_none()
        && cli.project_flag
        && let Some(project) = cli.project.as_deref()
        && let Some(url) = ProcessEnv.var("BRIDLE_URL")
    {
        return named_under_url(project, url, &cwd).await;
    }
    let endpoint = discovery::resolve_endpoint(
        cli.url.as_deref(),
        cli.project.as_deref(),
        &cwd,
        &ProcessEnv,
    )
    .map_err(|e| CliError::Unreachable(e.to_string()))?;
    Ok(Target {
        endpoint,
        redirected: false,
    })
}

/// `--project <project>` with `$BRIDLE_URL` set: keep the URL only if it is that project's daemon,
/// else go to the project's own daemon, else refuse. Never the URL's daemon by default.
async fn named_under_url(project: &str, url: String, cwd: &Path) -> Result<Target, CliError> {
    let own = Target {
        endpoint: Endpoint {
            url,
            workspace: None,
            project: None,
            machine: None,
        },
        redirected: false,
    };
    let without_url = |key: &str| ProcessEnv.var(key).filter(|_| key != "BRIDLE_URL");
    let named = discovery::resolve_endpoint(None, Some(project), cwd, &without_url);
    if let Ok(named) = &named
        && named.url.trim_end_matches('/') == own.endpoint.url.trim_end_matches('/')
    {
        return Ok(own);
    }
    // Ask it: an agent's own home has no registry. Only $BRIDLE_TOKEN goes with the question;
    // --token may be meant for the named project.
    let client = Client::new_with_timeout(
        own.endpoint.url.clone(),
        ProcessEnv.var("BRIDLE_TOKEN"),
        Duration::from_secs(5),
    );
    let answered = client.status().await.map(|s| s.daemon.project);
    if answered.as_deref().is_ok_and(|p| p == project) {
        return Ok(own);
    }
    match named {
        Ok(endpoint) => Ok(Target {
            endpoint,
            redirected: true,
        }),
        Err(e) => {
            let theirs = match answered {
                Ok(p) => format!("is project '{p}'"),
                Err(why) => format!("did not say which project it is ({why})"),
            };
            Err(CliError::Unreachable(format!(
                "--project {project}, but the daemon at $BRIDLE_URL ({}) {theirs}, and {e}. \
                 Nothing was sent to it. `bridle send --project {project} <to> <text>` is \
                 forwarded by your own daemon",
                own.endpoint.url
            )))
        }
    }
}

/// The URL and token for a command. `resolve_endpoint` failing to find any daemon at all is
/// exactly the "daemon unreachable" case (exit 3). `allow_anonymous_read` mirrors the daemon's own
/// tolerance for token-less GET/HEAD requests (server.rs's `auth_middleware`): pass it for a
/// read-only command so a caller running inside Claude Code with no `$BRIDLE_TOKEN` gets a working
/// request instead of a client-side error ahead of one that would have succeeded anyway.
pub(super) async fn resolve_endpoint_and_token(
    cli: &Cli,
    allow_anonymous_read: bool,
) -> Result<(String, Option<String>), CliError> {
    let Target {
        endpoint,
        redirected,
    } = resolve(cli).await?;
    let env = |key: &str| {
        ProcessEnv
            .var(key)
            .filter(|_| !(redirected && key == "BRIDLE_TOKEN"))
    };
    let token = discovery::resolve_token(
        cli.token.as_deref(),
        endpoint.workspace.as_deref(),
        endpoint.project.as_deref(),
        endpoint.machine.as_deref(),
        &env,
        allow_anonymous_read,
    );
    let token = match token {
        Err(e) if redirected => {
            return Err(CliError::from(anyhow::anyhow!(
                "--project {} is not $BRIDLE_URL's daemon, and $BRIDLE_TOKEN is that daemon's, \
                 so it is not sent: {e}",
                endpoint.project.as_deref().unwrap_or_default()
            )));
        }
        other => other?,
    };
    Ok((endpoint.url, token))
}
