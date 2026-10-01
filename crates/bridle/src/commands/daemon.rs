//! Daemon lifecycle commands: restart, stop-daemon, rebuild, daemons.

use super::*;

/// The daemon answers once it has decided to go (a busy daemon answers 409 and stays up), then
/// execs itself with the new binary, so wait for it to go and return (possibly on a new port).
pub(super) async fn restart(cli: &Cli, wait: Option<u64>, upgrade: bool) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let reply = client
        .restart(&bridle_api::types::RestartRequest {
            wait_secs: wait,
            upgrade,
        })
        .await?;
    if !reply.restarting {
        // Nothing newer, or the build runs on in the daemon and it restarts itself after.
        println!("{}", reply.message.unwrap_or_default());
        return Ok(());
    }
    println!(
        "restarting{}: {} agent{} to resume (stopping takes up to {}s)",
        reply.commit.map(|c| format!(" at {c}")).unwrap_or_default(),
        reply.agents.len(),
        if reply.agents.len() == 1 { "" } else { "s" },
        reply.stop_limit_secs
    );
    let limit = std::time::Duration::from_secs(reply.stop_limit_secs + 60);
    // An explicit --url is used as given; otherwise the daemon may come back on another port
    // (a new [projects] port or [daemon] listen), so look it up again through discovery.
    let explicit = cli.url.is_some();
    let log = std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            discovery::resolve_endpoint(None, cli.project.as_deref(), &cwd, &ProcessEnv).ok()
        })
        .and_then(|e| e.workspace)
        .map(|w| w.join(".bridle/daemon.log"))
        .filter(|p| p.exists());
    wait_for_restart(
        &client,
        async || {
            if explicit {
                return None;
            }
            client_for(cli).await.ok()
        },
        limit,
        std::time::Duration::from_millis(500),
        log.as_deref(),
    )
    .await
}

/// Wait for the daemon to go down and answer again. Once it is down, each poll asks `resolve`
/// for a fresh client (`None` keeps the current one), since the restart may move the port.
pub(super) async fn wait_for_restart(
    first: &Client,
    mut resolve: impl AsyncFnMut() -> Option<Client>,
    limit: std::time::Duration,
    poll: std::time::Duration,
    log: Option<&std::path::Path>,
) -> Result<(), CliError> {
    let deadline = std::time::Instant::now() + limit;
    let mut client = first.clone();
    let mut down = false;
    loop {
        match (client.health().await.is_ok(), down) {
            (false, _) => down = true,
            (true, true) => {
                println!("the daemon is back");
                return Ok(());
            }
            (true, false) => {}
        }
        if down && let Some(fresh) = resolve().await {
            client = fresh;
        }
        if std::time::Instant::now() >= deadline {
            let see = log
                .map(|p| format!("; see {}", p.display()))
                .unwrap_or_default();
            return Err(CliError::Other(anyhow::anyhow!(
                "the daemon did not come back within {}s{see}",
                limit.as_secs()
            )));
        }
        tokio::time::sleep(poll).await;
    }
}

pub(super) async fn stop_daemon(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    stop_daemon_with(
        &client,
        &mut std::io::stdout(),
        std::time::Duration::from_secs(60),
        std::time::Duration::from_millis(500),
    )
    .await
}

/// The daemon keeps answering health until its cleanup is done (agents
/// stopped, registry entry removed), so a failed health call means it's gone.
/// Prints as it goes because that can take most of `stop_grace` + 5 s.
pub(super) async fn stop_daemon_with(
    client: &Client,
    out: &mut impl std::io::Write,
    limit: std::time::Duration,
    poll: std::time::Duration,
) -> Result<(), CliError> {
    let started = std::time::Instant::now();
    let deadline = started + limit;
    let say = |out: &mut dyn std::io::Write, line: String| {
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    };
    say(out, "requested shutdown".into());
    let reply = client.shutdown().await?;
    let mut agents = client.health().await.ok().map(|h| h.agent_count);
    let plural = |n: u32| if n == 1 { "agent" } else { "agents" };
    say(
        out,
        match agents {
            Some(n) => {
                let limit = reply
                    .map(|r| format!(", up to {}s", r.stop_limit_secs))
                    .unwrap_or_default();
                format!(
                    "acknowledged; the daemon is stopping {n} {}{limit}",
                    plural(n)
                )
            }
            None => "acknowledged; the daemon is stopping".to_string(),
        },
    );
    loop {
        match client.health().await {
            Err(_) => {
                say(
                    out,
                    format!("shutdown complete ({}s)", started.elapsed().as_secs()),
                );
                return Ok(());
            }
            Ok(h) => {
                if agents.is_some_and(|prev| h.agent_count < prev) {
                    say(
                        out,
                        format!("{} {} still running", h.agent_count, plural(h.agent_count)),
                    );
                }
                agents = Some(h.agent_count);
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err(CliError::Other(anyhow::anyhow!(
                "the daemon did not stop within {}s; see `bridle daemons` and \
                 <workspace>/.bridle/daemon.log",
                limit.as_secs()
            )));
        }
        tokio::time::sleep(poll).await;
    }
}

pub(super) async fn rebuild(cli: &Cli, from_origin: bool) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let done = client.rebuild(from_origin).await?;
    if cli.json {
        render::print_json(&serde_json::json!({"ok": true, "origin": done.origin}))?;
    } else {
        if let Some(o) = &done.origin {
            println!("{o}");
        }
        println!("rebuilt tasks/edges/open_questions/handovers from the state branch");
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub(super) struct DaemonRow {
    #[serde(flatten)]
    info: bridle_api::DaemonInfo,
    /// `None` when the daemon didn't answer `/v1/health` within the timeout.
    agents: Option<u32>,
}

pub(super) async fn daemons(cli: &Cli) -> Result<(), CliError> {
    let list = discovery::list_registry();
    let timeout = std::time::Duration::from_secs(1);
    let counts = futures::future::join_all(list.iter().map(|d| {
        let client = Client::new_with_timeout(d.url.clone(), None, timeout);
        async move { client.health().await.ok().map(|h| h.agent_count) }
    }))
    .await;
    let rows: Vec<DaemonRow> = list
        .into_iter()
        .zip(counts)
        .map(|(info, agents)| DaemonRow { info, agents })
        .collect();

    if cli.json {
        render::print_json(&rows)?;
    } else if rows.is_empty() {
        println!("no daemons running");
    } else {
        let url_w = rows.iter().map(|d| d.info.url.len()).max().unwrap_or(3);
        println!(
            "{:<16} {:<8} {:<url_w$} {:<7} WORKSPACE",
            "PROJECT", "PID", "URL", "AGENTS"
        );
        for d in &rows {
            let agents = d
                .agents
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string());
            println!(
                "{:<16} {:<8} {:<url_w$} {:<7} {}",
                d.info.project, d.info.pid, d.info.url, agents, d.info.workspace
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod stop_daemon_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A bare HTTP fake: `POST /v1/shutdown` answers the limit; each health
    /// call pops the next agent count, and once `counts` is empty the
    /// listener is dropped (`None` in `counts_then_close` = never close).
    async fn fake_daemon(counts: Vec<u32>, close_after: bool) -> Client {
        fake_daemon_with(counts, close_after, true).await
    }

    /// `with_limit: false` mimics an older daemon: 204, no body.
    async fn fake_daemon_with(mut counts: Vec<u32>, close_after: bool, with_limit: bool) -> Client {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        counts.reverse();
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 2048];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let body = if req.starts_with("POST /v1/shutdown") {
                    if !with_limit {
                        let _ = sock
                            .write_all(b"HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n")
                            .await;
                        continue;
                    }
                    r#"{"stop_limit_secs":35}"#.to_string()
                } else {
                    let c = match counts.len() {
                        0 => {
                            if close_after {
                                return;
                            }
                            0
                        }
                        1 if !close_after => *counts.last().unwrap(),
                        _ => counts.pop().unwrap(),
                    };
                    format!(r#"{{"ok":true,"version":"t","agent_count":{c}}}"#)
                };
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        Client::new(url, None)
    }

    #[tokio::test]
    async fn prints_the_shutdown_sequence() {
        let client = fake_daemon(vec![2, 2, 1], true).await;
        let mut out = Vec::new();
        stop_daemon_with(
            &client,
            &mut out,
            std::time::Duration::from_secs(10),
            std::time::Duration::from_millis(10),
        )
        .await
        .unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "requested shutdown\n\
             acknowledged; the daemon is stopping 2 agents, up to 35s\n\
             1 agent still running\n\
             shutdown complete (0s)\n"
        );
    }

    #[tokio::test]
    async fn an_old_daemon_with_no_limit_omits_up_to() {
        let client = fake_daemon_with(vec![2, 2], true, false).await;
        let mut out = Vec::new();
        stop_daemon_with(
            &client,
            &mut out,
            std::time::Duration::from_secs(10),
            std::time::Duration::from_millis(10),
        )
        .await
        .unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(
            out.contains("acknowledged; the daemon is stopping 2 agents\n"),
            "{out}"
        );
    }

    #[tokio::test]
    async fn timeout_points_at_daemons_and_the_log() {
        let client = fake_daemon(vec![1], false).await;
        let mut out = Vec::new();
        let err = stop_daemon_with(
            &client,
            &mut out,
            std::time::Duration::from_millis(100),
            std::time::Duration::from_millis(10),
        )
        .await
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("did not stop"), "{msg}");
        assert!(msg.contains("bridle daemons"), "{msg}");
        assert!(msg.contains(".bridle/daemon.log"), "{msg}");
    }
}

#[cfg(test)]
mod restart_wait_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn health_server() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 2048];
                let _ = sock.read(&mut buf).await;
                let body = r#"{"ok":true,"version":"t","agent_count":0}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        url
    }

    fn dead_url() -> String {
        // Bind then drop so nothing listens on the port.
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}", l.local_addr().unwrap())
    }

    #[tokio::test]
    async fn follows_the_daemon_to_a_new_port() {
        let new = health_server().await;
        let old = Client::new(dead_url(), None);
        wait_for_restart(
            &old,
            async || Some(Client::new(new.clone(), None)),
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(10),
            None,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn timeout_names_the_log_only_when_given() {
        let old = Client::new(dead_url(), None);
        let run = async |log: Option<&std::path::Path>| {
            wait_for_restart(
                &old,
                async || None,
                std::time::Duration::from_millis(50),
                std::time::Duration::from_millis(10),
                log,
            )
            .await
            .unwrap_err()
            .to_string()
        };
        assert!(!run(None).await.contains("daemon.log"));
        assert!(
            run(Some(std::path::Path::new("/w/.bridle/daemon.log")))
                .await
                .contains("see /w/.bridle/daemon.log")
        );
    }
}
