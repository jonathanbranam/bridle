//! The bridle daemon: store, supervisor, containment and the HTTP API.
//! See docs/design/agent-host/.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use bridle_api::discovery;
use bridle_api::types::{
    AgentState, DaemonInfo, ExitInfo, PrincipalKind, SpawnRequest, event_kind,
};
use chrono::Utc;
use nix::errno::Errno;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use serde_json::json;
use tokio::sync::watch;

pub mod config;
pub mod containment;
mod events;
pub mod paths;
mod server;
pub mod store;
mod supervisor;
pub mod worktree;

pub use supervisor::{AgentManager, SupervisorError, ToTarget};

use config::Config;
use events::Emitter;
use paths::{Workspace, write_secret_file};
use store::{Principal, Store};

/// How long the event log keeps rows (docs/design/agent-host/api.md, Events).
const EVENT_RETENTION_DAYS: i64 = 30;

/// Options for `bridle serve`.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// The clone's main checkout.
    pub repo: PathBuf,
    /// Defaults to the repo's parent directory.
    pub workspace: Option<PathBuf>,
    /// Defaults to the repo directory's name.
    pub project: Option<String>,
    /// Defaults to config `[daemon] listen`, else `127.0.0.1:0`.
    pub listen: Option<SocketAddr>,
}

/// Test/embedding hooks that aren't part of the CLI-facing `ServeOptions`
/// (adding fields there would break `bridle`'s struct literal).
#[derive(Debug, Clone)]
pub struct Overrides {
    /// The `claude` executable to spawn. Defaults to `$BRIDLE_CLAUDE_BIN`,
    /// else `"claude"`.
    pub claude_program: String,
    /// Whether to also write the machine-wide registry entry. Tests set
    /// this to `false` and rely on `BRIDLE_HOME` isolation instead.
    pub write_registry: bool,
    pub stall_check_interval: Duration,
    pub tracker_interval: Duration,
}

impl Default for Overrides {
    fn default() -> Self {
        Self {
            claude_program: std::env::var("BRIDLE_CLAUDE_BIN")
                .unwrap_or_else(|_| "claude".to_string()),
            write_registry: true,
            stall_check_interval: Duration::from_secs(30),
            tracker_interval: Duration::from_secs(2),
        }
    }
}

/// A running daemon, for tests and for `run()` itself to drive.
pub struct RunningDaemon {
    pub url: String,
    pub info: DaemonInfo,
    shutdown_tx: watch::Sender<bool>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl RunningDaemon {
    /// Triggers the same graceful shutdown sequence as SIGTERM or
    /// `POST /v1/shutdown`.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Waits for the daemon to finish shutting down (which must have been
    /// triggered already, by [`RunningDaemon::shutdown`], a signal, or the
    /// API).
    pub async fn join(self) -> anyhow::Result<()> {
        self.join_handle
            .await
            .map_err(|e| anyhow::anyhow!("daemon task panicked: {e}"))
    }
}

/// Run the daemon until shutdown (SIGINT/SIGTERM or `POST /v1/shutdown`).
/// Writes `daemon.json` and the machine registry entry once listening.
pub async fn run(opts: ServeOptions) -> anyhow::Result<()> {
    let running = start(opts, Overrides::default()).await?;
    running.join().await
}

/// Starts the daemon and returns once it's listening, autostart has run,
/// and background tasks are up. Does not block for shutdown; see
/// [`RunningDaemon::join`].
pub async fn start(opts: ServeOptions, overrides: Overrides) -> anyhow::Result<RunningDaemon> {
    // First, so a detached daemon ignores SIGHUP from the moment it runs.
    let signals = Signals::install().context("installing signal handlers")?;
    if !worktree::is_git_repo(&opts.repo).await {
        anyhow::bail!("{} is not a git repository", opts.repo.display());
    }
    let ws = Workspace::new(opts.repo.clone(), opts.workspace.clone());
    ws.ensure_dirs().context("creating workspace directories")?;
    let config = Config::load(&opts.repo).context("loading .bridle/config.toml")?;
    let project = opts.project.clone().unwrap_or_else(|| {
        opts.repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "bridle".to_string())
    });

    refuse_if_already_running(&project, &ws)?;

    let store = Store::open(ws.db()).await.context("opening the store")?;
    ensure_human_token(&store, &ws).await?;

    let emitter = Emitter::new(store.clone());
    reconcile(&store, &emitter)
        .await
        .context("reconciling running agents")?;
    // The daily loop below waits a day before its first tick.
    let _ = store
        .prune_events(Utc::now() - chrono::Duration::days(EVENT_RETENTION_DAYS))
        .await;

    let listen_addr = opts.listen.unwrap_or(config.listen);
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("binding {listen_addr}"))?;
    let bound = listener.local_addr().context("reading bound address")?;
    let url = format!("http://{bound}");

    let info = DaemonInfo {
        project: project.clone(),
        workspace: ws.workspace.to_string_lossy().into_owned(),
        repo: ws.repo.to_string_lossy().into_owned(),
        url: url.clone(),
        pid: std::process::id() as i32,
        started_at: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    discovery::write_daemon_json(&ws.workspace, &info).context("writing daemon.json")?;
    if overrides.write_registry {
        discovery::write_registry(&info).context("writing registry entry")?;
    }

    let _ = emitter
        .emit(
            event_kind::DAEMON_STARTED,
            "system".to_string(),
            None,
            json!({}),
        )
        .await;

    let manager = AgentManager::new(
        store.clone(),
        ws.clone(),
        config.clone(),
        overrides.claude_program.clone(),
        url.clone(),
        project.clone(),
        emitter.clone(),
    );

    run_autostart_and_resume(&store, &config, &manager).await;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let state = server::AppState {
        store: store.clone(),
        manager: manager.clone(),
        emitter: emitter.clone(),
        workspace: ws.clone(),
        project: project.clone(),
        repo: ws.repo.to_string_lossy().into_owned(),
        url: url.clone(),
        version: info.version.clone(),
        started_at: info.started_at,
        pid: info.pid,
        shutdown_tx: shutdown_tx.clone(),
    };
    let app = server::router(state);

    let mut serve_shutdown_rx = shutdown_rx.clone();
    let serve_task = tokio::spawn(async move {
        let graceful = async move {
            let _ = serve_shutdown_rx.wait_for(|v| *v).await;
        };
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(graceful)
            .await
        {
            tracing::error!(error = %e, "axum serve error");
        }
    });

    let stall_task = spawn_loop(shutdown_rx.clone(), overrides.stall_check_interval, {
        let manager = manager.clone();
        move || {
            let manager = manager.clone();
            async move { manager.tick_stall_check().await }
        }
    });
    let tracker_task = spawn_loop(shutdown_rx.clone(), overrides.tracker_interval, {
        let manager = manager.clone();
        move || {
            let manager = manager.clone();
            async move { manager.tick_tracker().await }
        }
    });
    let prune_task = spawn_loop(shutdown_rx.clone(), Duration::from_secs(24 * 60 * 60), {
        let store = store.clone();
        move || {
            let store = store.clone();
            async move {
                let cutoff = Utc::now() - chrono::Duration::days(EVENT_RETENTION_DAYS);
                let _ = store.prune_events(cutoff).await;
            }
        }
    });
    let signal_task = signals.listen(shutdown_tx.clone());

    let join_handle = tokio::spawn(async move {
        let mut rx = shutdown_rx;
        let _ = rx.wait_for(|v| *v).await;

        let _ = emitter
            .emit(
                event_kind::DAEMON_STOPPING,
                "system".to_string(),
                None,
                json!({}),
            )
            .await;
        manager
            .stop_all(config.stop_grace + Duration::from_secs(5))
            .await;

        let _ = discovery::remove_registry(&project);
        let _ = std::fs::remove_file(ws.daemon_json());

        let _ = serve_task.await;
        signal_task.abort();
        stall_task.abort();
        tracker_task.abort();
        prune_task.abort();
    });

    Ok(RunningDaemon {
        url,
        info,
        shutdown_tx,
        join_handle,
    })
}

fn pid_alive(pid: i32) -> bool {
    match kill(Pid::from_raw(pid), None) {
        Ok(()) => true,
        Err(Errno::EPERM) => true,
        Err(_) => false,
    }
}

fn refuse_if_already_running(project: &str, ws: &Workspace) -> anyhow::Result<()> {
    for d in discovery::list_registry() {
        if d.project == project && Path::new(&d.workspace) != ws.workspace {
            anyhow::bail!(
                "a daemon for project '{project}' is already running at workspace {} (pid {})",
                d.workspace,
                d.pid
            );
        }
    }
    if let Ok(existing) = discovery::read_daemon_json(&ws.workspace)
        && pid_alive(existing.pid)
    {
        anyhow::bail!(
            "a daemon is already running for this workspace (pid {})",
            existing.pid
        );
    }
    Ok(())
}

async fn ensure_human_token(store: &Store, ws: &Workspace) -> anyhow::Result<()> {
    match store.ensure_human_token().await? {
        Some(token) => {
            write_secret_file(&ws.human_token(), &token)?;
            tracing::info!(path = %ws.human_token().display(), "wrote human token");
        }
        None => {
            // The principal exists, but if the file is gone (e.g. workspace
            // state dir was cleaned) the token is unrecoverable: revoke and
            // re-mint.
            if !ws.human_token().is_file() {
                store.revoke_principal("human").await?;
                if let Some(token) = store.ensure_human_token().await? {
                    write_secret_file(&ws.human_token(), &token)?;
                    tracing::info!(path = %ws.human_token().display(), "re-minted human token");
                }
            }
        }
    }
    Ok(())
}

/// docs/design/agent-host/daemon.md, restart and recovery: kill any surviving process for a recorded
/// "running" agent, mark it `lost`, and put its `written` messages back to
/// `pending`.
async fn reconcile(store: &Store, emitter: &Emitter) -> anyhow::Result<()> {
    for ra in store.running_agents().await? {
        if let (Some(pid), Some(start)) = (ra.pid, ra.pid_start.clone())
            && containment::is_same_process(pid, &start)
        {
            containment::terminate_group(pid, Duration::from_secs(3)).await;
            let mut tracker = containment::Tracker::new(pid, start);
            if let Ok(Ok(snap)) = tokio::task::spawn_blocking(containment::snapshot).await {
                tracker.update(&snap);
            }
            containment::sweep(&mut tracker, Duration::from_secs(2)).await;
        }
        store
            .set_agent_exit(
                &ra.id,
                ExitInfo {
                    code: None,
                    signal: None,
                    reason: "daemon_restart".to_string(),
                },
            )
            .await?;
        store.set_agent_state(&ra.id, AgentState::Lost).await?;
        let _ = emitter
            .emit(
                event_kind::AGENT_STATE,
                "system".to_string(),
                Some(ra.id.clone()),
                json!({"from": ra.state, "to": AgentState::Lost.as_str()}),
            )
            .await;
        let _ = emitter
            .emit(
                event_kind::AGENT_EXITED,
                "system".to_string(),
                Some(ra.id.clone()),
                json!({"code": null, "signal": null, "reason": "daemon_restart"}),
            )
            .await;
        for m in store
            .messages_for_agent(
                &ra.id,
                &[
                    bridle_api::types::MessageState::Written,
                    bridle_api::types::MessageState::Held,
                ],
            )
            .await?
        {
            store
                .set_message_state(&m.id, bridle_api::types::MessageState::Pending, Utc::now())
                .await?;
        }
    }
    Ok(())
}

async fn run_autostart_and_resume(store: &Store, config: &Config, manager: &AgentManager) {
    let system = Principal {
        id: "system".to_string(),
        kind: PrincipalKind::System,
    };
    for (name, role) in &config.roles {
        if role.autostart {
            match store.get_agent(name).await {
                Ok(None) => {
                    let req = SpawnRequest {
                        role: name.clone(),
                        name: Some(name.clone()),
                        prompt: None, // the role's start_prompt
                        workdir: None,
                        model: None,
                    };
                    if let Err(e) = manager.spawn(req, &system).await {
                        tracing::warn!(role = %name, error = %e, "autostart failed");
                    }
                }
                Ok(Some(_)) => {}
                Err(e) => tracing::warn!(role = %name, error = %e, "autostart lookup failed"),
            }
        }
    }
    let Ok(agents) = store.list_agents(true).await else {
        return;
    };
    for a in agents {
        if a.state == AgentState::Lost
            && config
                .roles
                .get(&a.role)
                .is_some_and(|r| r.resume_on_restart)
            && let Err(e) = manager.resume(&a.id, &system).await
        {
            tracing::warn!(agent = %a.id, error = %e, "resume-on-restart failed");
        }
    }
}

fn spawn_loop<F, Fut>(
    mut shutdown_rx: watch::Receiver<bool>,
    interval: Duration,
    mut tick: F,
) -> tokio::task::JoinHandle<()>
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => { tick().await; }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
    })
}

/// Signal handlers, registered as soon as the daemon starts (registering
/// replaces the default action, so SIGHUP no longer kills it) and listened
/// to once the shutdown channel exists.
struct Signals {
    sigint: tokio::signal::unix::Signal,
    sigterm: tokio::signal::unix::Signal,
    sighup: tokio::signal::unix::Signal,
}

impl Signals {
    fn install() -> std::io::Result<Self> {
        use tokio::signal::unix::{SignalKind, signal};
        Ok(Signals {
            sigint: signal(SignalKind::interrupt())?,
            sigterm: signal(SignalKind::terminate())?,
            sighup: signal(SignalKind::hangup())?,
        })
    }

    fn listen(self, tx: watch::Sender<bool>) -> tokio::task::JoinHandle<()> {
        let Signals {
            mut sigint,
            mut sigterm,
            mut sighup,
        } = self;
        tokio::spawn(async move {
            let mut rx = tx.subscribe();
            loop {
                tokio::select! {
                    _ = sigint.recv() => { let _ = tx.send(true); break; }
                    _ = sigterm.recv() => { let _ = tx.send(true); break; }
                    _ = sighup.recv() => { tracing::info!("received SIGHUP; ignoring"); }
                    _ = rx.changed() => { if *rx.borrow() { break; } }
                }
            }
        })
    }
}
