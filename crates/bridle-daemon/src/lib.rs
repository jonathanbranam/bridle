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

pub mod ci;
pub mod config;
pub mod containment;
pub mod cost_audit;
pub mod disk;
mod events;
pub mod governor;
pub mod impact;
mod integrator;
mod orchestrator;
pub mod paths;
pub mod ports;
pub mod reevaluate;
pub mod rules;
mod server;
pub mod state_branch;
pub mod store;
mod supervisor;
pub mod sync;
mod tasks;
mod wake;
pub mod worktree;

pub use governor::Governor;
pub use supervisor::{AgentManager, DAEMON_SHUTDOWN_REASON, SupervisorError, ToTarget};

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
    /// this to `false` and rely on `bridle_home` isolation instead.
    pub write_registry: bool,
    /// Overrides the machine-wide `~/.bridle` (or `$BRIDLE_HOME`) directory
    /// that [`config::Config::load`] reads `[budget]` from. `None` (the
    /// real `serve` default) uses the real machine home; tests set this to
    /// a per-test tempdir so they never read the real machine's
    /// `~/.bridle/config.toml`.
    pub bridle_home: Option<PathBuf>,
    /// Also drives the context governor's per-agent check (htp6b): both
    /// are cheap per-agent scans with no need for separate cadences.
    pub stall_check_interval: Duration,
    pub tracker_interval: Duration,
    /// How often the governor tick fires; each tick separately decides
    /// whether a `get_usage` poll is actually due, per the next two fields.
    pub governor_interval: Duration,
    /// usage-and-budget.md, Seeing the windows: how long to wait between
    /// polls below `hold_at`.
    pub governor_poll_interval_normal: Duration,
    /// ...and at or above it.
    pub governor_poll_interval_above_hold: Duration,
    /// How often pending state-branch writes are flushed to one commit
    /// (docs/design/storage.md, "The state branch"). There's no immediate-
    /// flush trigger yet (that arrives with `accept`), so this is the only
    /// thing that commits task records durably right now.
    pub task_flush_interval: Duration,
    /// How often [`tasks::TaskManager::tick_claim_lease_check`] walks open
    /// claims for a stale lease.
    pub claim_lease_check_interval: Duration,
    /// How often [`ports::tick`] frees ports whose pid or owner agent is gone.
    pub port_check_interval: Duration,
}

impl Default for Overrides {
    fn default() -> Self {
        Self {
            claude_program: std::env::var("BRIDLE_CLAUDE_BIN")
                .unwrap_or_else(|_| "claude".to_string()),
            write_registry: true,
            bridle_home: None,
            stall_check_interval: Duration::from_secs(30),
            tracker_interval: Duration::from_secs(2),
            governor_interval: Duration::from_secs(30),
            governor_poll_interval_normal: Duration::from_secs(5 * 60),
            governor_poll_interval_above_hold: Duration::from_secs(30),
            task_flush_interval: Duration::from_secs(30),
            claim_lease_check_interval: Duration::from_secs(30),
            port_check_interval: Duration::from_secs(30),
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
    let config = Config::load_with_home(&opts.repo, overrides.bridle_home.as_deref())
        .context("loading .bridle/config.toml")?;
    // An unset `[branches] integration` means `main`; a repo on `master` would otherwise
    // fail every spawn with `invalid reference` (g3ck). No guessing from HEAD.
    if !config.branches.integration_set
        && !worktree::branch_exists(&opts.repo, &config.branches.integration).await?
    {
        anyhow::bail!(
            "branch `{0}` doesn't exist in {1}: set `branches.integration` in \
             .bridle/config.toml to this project's integration branch",
            config.branches.integration,
            opts.repo.display()
        );
    }
    let project = opts.project.clone().unwrap_or_else(|| {
        opts.repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "bridle".to_string())
    });

    refuse_if_already_running(&project, &ws)?;

    let store = Store::open(ws.db()).await.context("opening the store")?;
    ensure_human_token(&store, &ws).await?;

    let state_branch = state_branch::StateBranch::open(&ws.repo, &ws.state_branch_dir())
        .await
        .context("opening the state branch")?;
    let task_prefix = config
        .task_prefix
        .clone()
        .unwrap_or_else(|| config::default_task_prefix(&project));
    let tasks = tasks::TaskManager::open(
        store.clone(),
        state_branch,
        task_prefix,
        config.claim_lease_after,
    )
    .await
    .context("loading tasks")?;

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

    let governor_handle: governor::GovernorHandle =
        std::sync::Arc::new(std::sync::Mutex::new(governor::GovernorSnapshot::default()));
    let manager = AgentManager::new(
        store.clone(),
        ws.clone(),
        config.clone(),
        overrides.claude_program.clone(),
        url.clone(),
        project.clone(),
        emitter.clone(),
        governor_handle.clone(),
    );
    let governor = Governor::new(
        store.clone(),
        manager.clone(),
        emitter.clone(),
        config.budget.clone(),
        overrides.claude_program.clone(),
        ws.clone(),
        governor_handle,
        overrides.governor_poll_interval_normal,
        overrides.governor_poll_interval_above_hold,
    );

    let wakes = wake::Wakes::new(
        store.clone(),
        ws.repo.clone(),
        config.branches.integration.clone(),
    );
    let waiters = wake::Waiters::new(Utc::now());
    let ci = ci::CiWatcher::new(
        config.ci.github,
        config.branches.integration.clone(),
        std::sync::Arc::new(ci::RealGh::new(ws.repo.clone())),
        store.clone(),
        manager.clone(),
        emitter.clone(),
    );

    let disk = disk::DiskMonitor::new(
        ws.clone(),
        config.disk.min_free_gb,
        emitter.clone(),
        manager.clone(),
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
        governor: governor.clone(),
        ci: ci.clone(),
        wakes: wakes.clone(),
        waiters: waiters.clone(),
        tasks: tasks.clone(),
        integration: config.branches.integration.clone(),
        ports: config.ports.clone(),
        integration_check: config.integration.check.clone(),
        landing: Default::default(),
    };
    let app = server::router(state);

    // The axum listener must close only after the cleanup sequence below
    // (through removing daemon.json) has finished, not merely when
    // `shutdown_rx` flips — otherwise a client polling health() can see the
    // listener go away and conclude shutdown is done while daemon.json still
    // exists.
    let (serve_shutdown_tx, mut serve_shutdown_rx) = watch::channel(false);
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
            async move {
                manager.tick_stall_check().await;
                manager.tick_context_check().await;
            }
        }
    });
    let tracker_task = spawn_loop(shutdown_rx.clone(), overrides.tracker_interval, {
        let manager = manager.clone();
        move || {
            let manager = manager.clone();
            async move { manager.tick_tracker().await }
        }
    });
    let governor_task = spawn_loop(shutdown_rx.clone(), overrides.governor_interval, {
        let governor = governor.clone();
        move || {
            let governor = governor.clone();
            async move { governor.tick().await }
        }
    });
    let ci_task = spawn_loop(shutdown_rx.clone(), ci::TICK_INTERVAL, {
        let ci = ci.clone();
        move || {
            let ci = ci.clone();
            async move { ci.tick().await }
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
    let task_flush_task = spawn_loop(shutdown_rx.clone(), overrides.task_flush_interval, {
        let tasks = tasks.clone();
        move || {
            let tasks = tasks.clone();
            async move {
                if let Err(e) = tasks.flush_now().await {
                    tracing::warn!(error = %e, "flushing the task state branch failed");
                }
            }
        }
    });
    let claim_lease_task = spawn_loop(shutdown_rx.clone(), overrides.claim_lease_check_interval, {
        let tasks = tasks.clone();
        move || {
            let tasks = tasks.clone();
            async move { tasks.tick_claim_lease_check(Utc::now()).await }
        }
    });
    let ports_task = spawn_loop(shutdown_rx.clone(), overrides.port_check_interval, {
        let store = store.clone();
        move || {
            let store = store.clone();
            async move { ports::tick(&store).await }
        }
    });
    // A zero interval turns the monitor off.
    let disk_task = (!config.disk.check_interval.is_zero()).then(|| {
        spawn_loop(shutdown_rx.clone(), config.disk.check_interval, {
            let disk = disk.clone();
            move || {
                let disk = disk.clone();
                async move { disk.tick().await }
            }
        })
    });
    let orchestrator_task = config.orchestrator.enabled.then(|| {
        let home = overrides
            .bridle_home
            .clone()
            .unwrap_or_else(discovery::bridle_home);
        let launcher = ws.repo.join(&config.orchestrator.launcher);
        let sup = orchestrator::real(
            home,
            orchestrator::shell_word(&launcher),
            &config.orchestrator,
            waiters.clone(),
            manager.clone(),
            emitter.clone(),
        );
        spawn_loop(
            shutdown_rx.clone(),
            orchestrator::TICK_INTERVAL,
            move || {
                let sup = sup.clone();
                async move { sup.tick(Utc::now()).await }
            },
        )
    });
    // The baseline (cursor at the tail, main's head) is taken now, not a tick from now.
    wakes.tick(Utc::now()).await;
    let wake_task = spawn_loop(shutdown_rx.clone(), orchestrator::TICK_INTERVAL, {
        let wakes = wakes.clone();
        move || {
            let wakes = wakes.clone();
            async move { wakes.tick(Utc::now()).await }
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
        let cap = config.stop_grace + Duration::from_secs(5);
        let n = manager.running_ids().len();
        if n > 0 {
            tracing::info!(
                "stopping {n} agent{} (up to {}s)...",
                if n == 1 { "" } else { "s" },
                cap.as_secs()
            );
        }
        manager.stop_all(cap).await;

        // Best-effort: flush whatever's pending on the state branch so a
        // clean shutdown doesn't wait up to `task_flush_interval` to become
        // durable there. A crash instead of a clean shutdown still relies
        // on the next flush after restart, per the batching trade-off
        // (docs/design/storage.md).
        if let Err(e) = tasks.flush_now().await {
            tracing::warn!(error = %e, "flushing the task state branch on shutdown failed");
        }

        let _ = discovery::remove_registry(&project);
        let _ = std::fs::remove_file(ws.daemon_json());

        let _ = serve_shutdown_tx.send(true);
        // Open connections (SSE streams end on shutdown, but anything else
        // slow) must not keep the process alive indefinitely.
        if tokio::time::timeout(Duration::from_secs(5), serve_task)
            .await
            .is_err()
        {
            tracing::warn!("http server did not drain within 5s of shutdown; exiting anyway");
        }
        signal_task.abort();
        stall_task.abort();
        tracker_task.abort();
        governor_task.abort();
        ci_task.abort();
        if let Some(t) = disk_task {
            t.abort();
        }
        if let Some(t) = orchestrator_task {
            t.abort();
        }
        prune_task.abort();
        task_flush_task.abort();
        claim_lease_task.abort();
        ports_task.abort();
        wake_task.abort();
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
        let alive = match tokio::task::spawn_blocking(containment::snapshot).await {
            Ok(Ok(snap)) => match (ra.pid, ra.pid_start.clone()) {
                (Some(pid), Some(start)) => snap
                    .iter()
                    .any(|p| p.pid == pid && p.start == start)
                    .then_some((pid, start)),
                _ => None,
            },
            _ => None,
        };
        if let Some((pid, start)) = alive {
            containment::terminate_group(pid, Duration::from_secs(3)).await;
            let mut tracker = containment::Tracker::new(pid, start);
            if let Ok(Ok(snap)) = tokio::task::spawn_blocking(containment::snapshot).await {
                tracker.update(&snap);
                containment::sweep(&mut tracker, &snap, Duration::from_secs(2)).await;
            }
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
    let Ok(agents) = store.list_agents(true).await else {
        tracing::warn!("autostart lookup failed");
        return;
    };
    for (name, role) in &config.roles {
        // By role, not name: a manager named `manager-2` still counts.
        if role.autostart && !agents.iter().any(|a| &a.role == name) {
            let req = SpawnRequest {
                role: name.clone(),
                name: Some(name.clone()),
                prompt: None, // the role's start_prompt
                workdir: None,
                model: None,
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                ignore_budget: false,
                components: Vec::new(),
            };
            if let Err(e) = manager.spawn(req, &system).await {
                tracing::warn!(role = %name, error = %e, "autostart failed");
            }
        }
    }
    for a in agents {
        let resumable_after_restart = a.state == AgentState::Lost
            || (a.state == AgentState::Stopped
                && a.exit
                    .as_ref()
                    .is_some_and(|e| e.reason == supervisor::DAEMON_SHUTDOWN_REASON));
        if resumable_after_restart
            && config
                .roles
                .get(&a.role)
                .is_some_and(|r| r.resume_on_restart)
            && let Err(e) = manager.resume(&a.id, false, &system).await
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
                    _ = sigint.recv() => { tracing::warn!("shutdown requested: received SIGINT"); let _ = tx.send(true); break; }
                    _ = sigterm.recv() => { tracing::warn!("shutdown requested: received SIGTERM"); let _ = tx.send(true); break; }
                    _ = sighup.recv() => { tracing::info!("received SIGHUP; ignoring"); }
                    _ = rx.changed() => { if *rx.borrow() { break; } }
                }
            }
        })
    }
}
