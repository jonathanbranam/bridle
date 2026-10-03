//! `bridle serve`: foreground and `--detach`. See docs/design/agent-host/daemon.md.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context;
use bridle_api::discovery;

use crate::cli::{Cli, ServeArgs};
use crate::error::CliError;
use crate::render;

pub async fn run(cli: &Cli, args: &ServeArgs) -> Result<(), CliError> {
    crate::tools_only::refuse_serve(&canonical_repo(args)?)?;
    if args.check {
        bridle_daemon::rollback::preflight(&canonical_repo(args)?, None)?;
        return Ok(());
    }
    if args.detach {
        run_detached(cli, args).await
    } else {
        run_foreground(cli, args).await
    }
}

async fn run_foreground(cli: &Cli, args: &ServeArgs) -> Result<(), CliError> {
    init_tracing();
    let repo = canonical_repo(args)?;
    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: args.workspace.clone(),
        project: cli.project.clone(),
        listen: args.listen,
    };
    bridle_daemon::run(opts, args.take_over).await?;
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

fn canonical_repo(args: &ServeArgs) -> anyhow::Result<PathBuf> {
    let repo = match &args.repo {
        Some(r) => r.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    repo.canonicalize()
        .with_context(|| format!("repo path {}", repo.display()))
}

async fn run_detached(cli: &Cli, args: &ServeArgs) -> Result<(), CliError> {
    let repo = canonical_repo(args)?;
    let workspace = match &args.workspace {
        Some(w) => w.clone(),
        None => repo
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| repo.clone()),
    };
    let state_dir = discovery::state_dir(&workspace);
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("creating {}", state_dir.display()))?;

    let current_exe = std::env::current_exe().context("locating the bridle binary")?;
    let child_args: Vec<std::ffi::OsString> = std::env::args_os()
        .skip(1)
        .filter(|a| a != "--detach")
        .collect();

    let log_path = state_dir.join("daemon.log");
    let log_out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("opening {}", log_path.display()))?;
    let log_err = log_out
        .try_clone()
        .context("duplicating the daemon.log handle")?;

    let mut child = {
        use std::os::unix::process::CommandExt;
        std::process::Command::new(&current_exe)
            .args(&child_args)
            .stdin(std::process::Stdio::null())
            .stdout(log_out)
            .stderr(log_err)
            .process_group(0)
            .spawn()
            .context("spawning the detached daemon")?
    };
    wait_for_daemon(cli.json, &workspace, &mut child, &log_path, DETACH_WAIT).await
}

/// How long `--detach` waits for the daemon's health to answer. A loaded host (a build
/// running beside it) can take well past 15 s to start; see ticket cy5v.
const DETACH_WAIT: Duration = Duration::from_secs(60);

async fn wait_for_daemon(
    json: bool,
    workspace: &Path,
    child: &mut std::process::Child,
    log_path: &Path,
    wait: Duration,
) -> Result<(), CliError> {
    let child_pid = child.id() as i32;
    let deadline = Instant::now() + wait;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(CliError::Other(anyhow::anyhow!(
                "daemon exited early ({status}); tail of {}:\n{}",
                log_path.display(),
                tail_of_log(log_path, 40)
            )));
        }
        if let Ok(info) = discovery::read_daemon_json(workspace)
            && info.pid == child_pid
        {
            let client = bridle_api::Client::new(info.url.clone(), None);
            if client.health().await.is_ok() {
                if json {
                    render::print_json(&serde_json::json!({"url": info.url, "pid": info.pid}))?;
                } else {
                    println!("bridle daemon listening on {} (pid {})", info.url, info.pid);
                }
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            // Not a failure: the daemon is still coming up and is left running.
            eprintln!("{}", still_starting_message(child_pid, wait, log_path));
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

fn still_starting_message(pid: i32, wait: Duration, log_path: &Path) -> String {
    format!(
        "warning: the daemon (pid {pid}) is still starting after {}s and was left running.\n\
         Log: {}\nCheck on it with `bridle daemons`.",
        wait.as_secs(),
        log_path.display()
    )
}

fn tail_of_log(path: &Path, n: usize) -> String {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let lines: Vec<&str> = s.lines().collect();
            let start = lines.len().saturating_sub(n);
            lines[start..].join("\n")
        }
        Err(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn sleeper() -> std::process::Child {
        std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep")
    }

    #[test]
    fn still_starting_message_names_pid_log_and_daemons() {
        let m = still_starting_message(4242, DETACH_WAIT, Path::new("/w/.bridle/daemon.log"));
        assert!(m.contains("4242") && m.contains("/w/.bridle/daemon.log"));
        assert!(m.contains("bridle daemons") && m.contains("left running"));
    }

    #[tokio::test]
    async fn timeout_leaves_the_daemon_running_and_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("daemon.log");
        let mut child = sleeper();
        let r = wait_for_daemon(
            true,
            dir.path(),
            &mut child,
            &log,
            Duration::from_millis(300),
        )
        .await;
        assert!(r.is_ok());
        assert!(
            child.try_wait().unwrap().is_none(),
            "child must be left running"
        );
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[tokio::test]
    async fn comes_up_inside_the_wait() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("daemon.log");
        let mut child = sleeper();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                let mut buf = [0u8; 1024];
                let _ = sock.read(&mut buf).await;
                let body = r#"{"ok":true,"version":"t","agent_count":0}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        discovery::write_daemon_json(
            dir.path(),
            &bridle_api::DaemonInfo {
                project: "p".into(),
                workspace: dir.path().display().to_string(),
                repo: "r".into(),
                url,
                pid: child.id() as i32,
                started_at: chrono::Utc::now(),
                version: "t".into(),
            },
        )
        .unwrap();
        let r = wait_for_daemon(true, dir.path(), &mut child, &log, Duration::from_secs(10)).await;
        assert!(r.is_ok());
        child.kill().unwrap();
        child.wait().unwrap();
    }
}
