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
    startup_migrations(&opts).await;
    tokio::spawn(warn_if_not_logged_in(
        std::process::Command::new("claude"),
        LOGIN_CHECK_TIMEOUT,
    ));
    bridle_daemon::run(opts, args.take_over)
        .await
        .map_err(owner_refusal)?;
    Ok(())
}

/// Maps the owner refusal (a typed `OwnerConflict` somewhere in the chain) to its own exit code,
/// so a supervisor can be told not to restart it; the message is the one log line.
fn owner_refusal(e: anyhow::Error) -> CliError {
    match e.downcast_ref::<bridle_daemon::state_branch::OwnerConflict>() {
        Some(c) => CliError::OwnerRefused(format!(
            "this project is owned by {}; refusing to start; run `bridle serve --take-over` here to take it over, or uninstall this unit (since {})",
            c.host, c.since
        )),
        None => CliError::Other(e),
    }
}

/// Apply the project's pending migrations before the daemon starts serving (docs/design/migrations.md).
/// Nothing in here may stop the daemon: errors and panics are logged, and the reporting
/// (events, incident) is a background task that waits for the daemon to be up.
async fn startup_migrations(opts: &bridle_daemon::ServeOptions) {
    let repo = opts.repo.clone();
    let run = tokio::task::spawn_blocking(move || {
        crate::migrate::run_at_startup(&repo, crate::migrate::MIGRATIONS)
    })
    .await;
    let result = match run {
        Ok(Some(r)) => r,
        Ok(None) => return,
        Err(e) => Err(format!("the migration run panicked: {e}")),
    };
    let workspace = opts.workspace.clone().unwrap_or_else(|| {
        opts.repo
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| opts.repo.clone())
    });
    let project = opts.project.clone().unwrap_or_else(|| {
        opts.repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    tokio::spawn(report_startup_migrations(workspace, project, result));
}

async fn report_startup_migrations(
    workspace: PathBuf,
    project: String,
    result: Result<crate::migrate::Outcome, String>,
) {
    use crate::migrate::is_refusal;
    // What went wrong, if it's a failure; a refusal is only logged and retried at next start.
    let failure = match &result {
        Ok(out) => {
            for r in &out.done {
                tracing::info!("migrated {}: {}", r.id, r.summary);
            }
            match &out.failed {
                Some((id, e)) if is_refusal(e) => {
                    tracing::warn!("migration {id} skipped, will retry at next start: {e:#}");
                    None
                }
                Some((id, e)) => Some(format!("migration {id} failed: {e:#}")),
                None => None,
            }
        }
        Err(e) => Some(format!("the migration run failed: {e}")),
    };
    if let Some(f) = &failure {
        tracing::error!("{f}");
    }
    let done = match &result {
        Ok(out) => out.done.as_slice(),
        Err(_) => &[],
    };
    if done.is_empty() && failure.is_none() {
        return;
    }

    // The daemon writes daemon.json once it's listening; wait for it, up to a minute.
    let deadline = Instant::now() + Duration::from_secs(60);
    let url = loop {
        if let Ok(info) = discovery::read_daemon_json(&workspace)
            && info.pid == std::process::id() as i32
        {
            break info.url;
        }
        if Instant::now() >= deadline {
            tracing::warn!("daemon never came up; migration events and incident not posted");
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    };
    let token = discovery::resolve_token(
        None,
        Some(&workspace),
        Some(&project),
        None,
        &discovery::ProcessEnv,
        false,
    )
    .ok()
    .flatten();
    let client = bridle_api::Client::new(url, token);
    for rec in done {
        if let Err(e) = client.record_migration(rec).await {
            tracing::warn!("couldn't record {} as an event: {e}", rec.id);
        }
    }
    if let Some(f) = failure {
        let req = bridle_api::types::NewTaskRequest {
            title: "Project migration failed at start-up".into(),
            kind: bridle_api::types::TaskKind::Incident,
            body: format!(
                "{f}\n\nThe daemon started anyway. Fix the cause and restart, or run \
                 `bridle migrate` by hand; details are in the daemon log."
            ),
            size: None,
            components: vec![],
            for_human: false,
            priority: None,
            ticket: None,
            parent: None,
        };
        if let Err(e) = client.new_task(&req).await {
            tracing::warn!("couldn't file an incident for the migration failure: {e}");
        }
    }
}

const LOGIN_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Off the start-up path: a daemon not logged in to claude runs agents that silently do
/// nothing (nrbf), so say so loudly. A timeout, a missing `claude` or any error is "unknown"
/// and stays silent; this never affects start-up.
async fn warn_if_not_logged_in(cmd: std::process::Command, timeout: Duration) {
    if known_logged_out(cmd, timeout).await {
        tracing::warn!(
            "{}; {}",
            crate::doctor::NOT_LOGGED_IN_DETAIL,
            crate::doctor::not_logged_in_fix(std::env::consts::OS)
        );
    }
}

async fn known_logged_out(cmd: std::process::Command, timeout: Duration) -> bool {
    let check = tokio::task::spawn_blocking(move || crate::doctor::claude_logged_in(cmd));
    matches!(
        tokio::time::timeout(timeout, check).await,
        Ok(Ok(Some(false)))
    )
}

pub(crate) fn init_tracing() {
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

    let log_path = state_dir.join("daemon.log");
    let mut child = spawn_detached(&log_path, "daemon")?;
    wait_for_daemon(cli.json, &workspace, &mut child, &log_path, DETACH_WAIT).await
}

/// Re-executes this command line without `--detach` in a new process group, output appended
/// to `log_path`. Shared by `bridle serve --detach` and `bridle gateway --detach`.
pub(crate) fn spawn_detached(log_path: &Path, what: &str) -> anyhow::Result<std::process::Child> {
    let child_args: Vec<std::ffi::OsString> = std::env::args_os()
        .skip(1)
        .filter(|a| a != "--detach")
        .collect();
    spawn_detached_with(log_path, what, &child_args)
}

/// Like [`spawn_detached`] with the child's arguments given, for a caller (`gateway restart`)
/// whose own arguments aren't the child's.
pub(crate) fn spawn_detached_with(
    log_path: &Path,
    what: &str,
    child_args: &[std::ffi::OsString],
) -> anyhow::Result<std::process::Child> {
    let current_exe = std::env::current_exe().context("locating the bridle binary")?;
    let log_out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("opening {}", log_path.display()))?;
    let log_err = log_out
        .try_clone()
        .with_context(|| format!("duplicating the {} handle", log_path.display()))?;
    use std::os::unix::process::CommandExt;
    detached_command(
        &current_exe,
        child_args,
        std::env::vars_os().map(|(k, _)| k),
    )
    .stdin(std::process::Stdio::null())
    .stdout(log_out)
    .stderr(log_err)
    .process_group(0)
    .spawn()
    .with_context(|| format!("spawning the detached {what}"))
}

/// Variables a Claude Code session sets in its caller's environment; a detached child that
/// inherited them would think it runs inside that session (n57nt).
fn is_claude_env(key: &std::ffi::OsStr) -> bool {
    let k = key.to_string_lossy();
    k == "CLAUDECODE" || k.starts_with("CLAUDE_CODE_") || k.starts_with("ANTHROPIC_")
}

/// The detached child never inherits a principal or project from the starting shell (ppa6):
/// the gateway ignores them, and a detached daemon has its own identity. Nor does it inherit
/// the caller's Claude Code variables (n57nt); `caller_vars` is the caller's variable names.
fn detached_command(
    exe: &Path,
    child_args: &[std::ffi::OsString],
    caller_vars: impl Iterator<Item = std::ffi::OsString>,
) -> std::process::Command {
    let mut cmd = std::process::Command::new(exe);
    cmd.args(child_args);
    for k in ["BRIDLE_AS", "BRIDLE_PROJECT", "BRIDLE_TOKEN"] {
        cmd.env_remove(k);
    }
    for k in caller_vars.filter(|k| is_claude_env(k)) {
        cmd.env_remove(k);
    }
    cmd
}

/// How long `--detach` waits for the daemon's health to answer. A loaded host (a build
/// running beside it) can take well past 15 s to start; see ticket cy5v.
pub(crate) const DETACH_WAIT: Duration = Duration::from_secs(60);

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

pub(crate) fn tail_of_log(path: &Path, n: usize) -> String {
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

    #[test]
    fn owner_conflict_maps_to_exit_78() {
        let e = anyhow::Error::new(bridle_daemon::state_branch::OwnerConflict {
            project: "p".into(),
            host: "nuc".into(),
            since: "t".into(),
        })
        .context("starting");
        let c = owner_refusal(e);
        assert_eq!(c.exit_code(), 78);
        assert!(c.to_string().contains("owned by nuc"));
        assert_eq!(owner_refusal(anyhow::anyhow!("x")).exit_code(), 1);
    }

    #[test]
    fn detached_command_removes_principal_env() {
        let vars = [
            "CLAUDECODE",
            "CLAUDE_CODE_ENTRYPOINT",
            "ANTHROPIC_API_KEY",
            "PATH",
        ]
        .map(std::ffi::OsString::from);
        let cmd = super::detached_command(std::path::Path::new("/bin/true"), &[], vars.into_iter());
        let found = cmd.get_envs().find(|(key, _)| *key == "PATH");
        assert_eq!(found, None, "unrelated variables are kept");
        for k in [
            "BRIDLE_AS",
            "BRIDLE_PROJECT",
            "BRIDLE_TOKEN",
            "CLAUDECODE",
            "CLAUDE_CODE_ENTRYPOINT",
            "ANTHROPIC_API_KEY",
        ] {
            let found = cmd.get_envs().find(|(key, _)| *key == k);
            assert_eq!(found, Some((std::ffi::OsStr::new(k), None)), "{k}");
        }
    }

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

    fn fake_claude(dir: &Path, body: &str) -> std::process::Command {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join("claude");
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).expect("write");
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        std::process::Command::new(p)
    }

    #[tokio::test]
    async fn login_warning_only_when_known_logged_out() {
        let d = tempfile::tempdir().expect("tmp");
        // Generous: exec of the fake can stall for seconds in a loaded suite (br-khg3).
        let t = Duration::from_secs(60);
        let out = fake_claude(d.path(), "echo '{\"loggedIn\": false}'; exit 1");
        assert!(known_logged_out(out, t).await);
        let ok = fake_claude(d.path(), "echo '{\"loggedIn\": true}'");
        assert!(!known_logged_out(ok, t).await);
        let junk = fake_claude(d.path(), "echo huh");
        assert!(!known_logged_out(junk, t).await);
        let missing = std::process::Command::new(d.path().join("nope"));
        assert!(!known_logged_out(missing, t).await);
        let slow = fake_claude(d.path(), "sleep 2; echo '{\"loggedIn\": false}'");
        assert!(!known_logged_out(slow, Duration::from_millis(100)).await);
    }
}
