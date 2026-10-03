//! §9: restart reconciliation (docs/design/agent-host/daemon.md).
//!
//! Two daemons in one OS process would share a pid, which defeats
//! pid-liveness checks, so instead of literally running two overlapping
//! daemons we simulate exactly the state a crashed daemon would leave
//! behind: an `agents` row stuck `working` with the pid/start-time of a
//! still-live, but otherwise unsupervised, process. A fresh `start()` on
//! that workspace must find it, kill it, and mark it `lost`.

mod support;

use bridle_api::types::{AgentState, SpawnRequest, Workdir};
use bridle_daemon::containment;
use bridle_daemon::store::{NewAgent, Store};
use tokio::process::Command;

#[tokio::test]
async fn restart_marks_a_stale_running_agent_lost_and_kills_its_process() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    support::init_repo(&repo).await;
    let workspace = tmp.path().to_path_buf();

    let ws = bridle_daemon::paths::Workspace::new(repo.clone(), Some(workspace.clone()));
    ws.ensure_dirs().expect("ensure_dirs");
    let store = Store::open(ws.db()).await.expect("open store");

    // A raw, orphan-like long-lived process standing in for a still-running
    // `claude`, spawned outside any supervisor. `process_group(0)` matches
    // how bridle-claude actually spawns agents (its own group leader), since
    // that's what `containment::terminate_group` (`kill(-pgid, ...)`) needs.
    let mut child = Command::new("sleep")
        .arg("300")
        .process_group(0)
        .kill_on_drop(false)
        .spawn()
        .expect("spawn stray process");
    let pid = child.id().expect("pid") as i32;
    let start = support::wait_for("process to appear in ps", || async {
        containment::start_time(pid)
    })
    .await;

    let agent = store
        .insert_agent(NewAgent {
            components: Vec::new(),
            name: "stale".to_string(),
            role: "worker".to_string(),
            model: "sonnet".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            workdir_kind: "repo".to_string(),
            cwd: repo.to_string_lossy().into_owned(),
            worktree: None,
            branch: None,
            created_by: "human".to_string(),
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
        })
        .await
        .expect("insert agent");
    store
        .set_agent_process(&agent.id, Some(pid), Some(start))
        .await
        .expect("set process");
    store
        .set_agent_state(&agent.id, AgentState::Working)
        .await
        .expect("set working");
    drop(store);

    let opts = bridle_daemon::ServeOptions {
        repo: repo.clone(),
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let overrides = bridle_daemon::Overrides {
        claude_program: support::fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        bridle_home: Some(support::machine_home_dir(tmp.path())),
        stall_check_interval: std::time::Duration::from_secs(3600),
        tracker_interval: std::time::Duration::from_millis(200),
        governor_interval: std::time::Duration::from_secs(3600),
        governor_poll_interval_normal: std::time::Duration::from_secs(3600),
        governor_poll_interval_above_hold: std::time::Duration::from_secs(3600),
        task_flush_interval: std::time::Duration::from_secs(3600),
        claim_lease_check_interval: std::time::Duration::from_secs(3600),
        port_check_interval: std::time::Duration::from_secs(3600),
        upgrade: Default::default(),
        ci_tick_interval: std::time::Duration::from_secs(3600),
        self_upgrade_wait: std::time::Duration::from_secs(600),
        settle_wake_interval: std::time::Duration::from_secs(3600),
        queue_nudge_debounce: std::time::Duration::from_secs(3600),
        take_over: false,
        host: None,
    };
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start daemon");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    let lost = support::wait_for_state(&client, &agent.id, AgentState::Lost).await;
    let exit = lost.exit.expect("exit info");
    assert_eq!(exit.reason, "daemon_restart");
    support::wait_for_event(&client, "agent.state", Some(&agent.id), |e| {
        e.data["from"] == "working" && e.data["to"] == "lost"
    })
    .await;

    // The process was actually terminated (not just marked lost in the
    // store): reap it and check it died by signal, not by living out its
    // 300s sleep. We're its real OS parent (spawned directly in this
    // test), so `kill(pid, 0)` alone isn't reliable here — a signaled
    // child sits as a zombie, and zombie pids still answer a liveness
    // probe, until whoever reaps it (us) calls `wait()`.
    let status = tokio::time::timeout(support::HANG_GUARD_TIMEOUT, child.wait())
        .await
        .expect("stray process was not reaped promptly")
        .expect("wait on stray process");
    assert!(
        !status.success(),
        "expected the stray process to die by signal, got {status:?}"
    );

    running.shutdown();
    running.join().await.expect("join");
}

/// A `resume_on_restart` role (the built-in `manager`) must come back not
/// just after a crash (the test above) but also after a clean shutdown: the
/// bug this guards against is `run_autostart_and_resume` only ever looking
/// for `AgentState::Lost`, which a graceful `stop_all` never produces (it
/// leaves the agent `Stopped` with reason `sigterm`/`stdin_closed`).
#[tokio::test]
async fn resume_on_restart_role_comes_back_after_a_clean_shutdown_then_restart() {
    let (daemon, tmp) = support::start_daemon(None).await;
    let workspace = daemon.workspace.clone();
    let repo = daemon.repo.clone();

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "manager".to_string(),
            name: Some("mgr".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn manager");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");

    let opts = bridle_daemon::ServeOptions {
        repo: repo.clone(),
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let overrides = bridle_daemon::Overrides {
        claude_program: support::fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        bridle_home: Some(support::machine_home_dir(tmp.path())),
        stall_check_interval: std::time::Duration::from_secs(3600),
        tracker_interval: std::time::Duration::from_millis(200),
        governor_interval: std::time::Duration::from_secs(3600),
        governor_poll_interval_normal: std::time::Duration::from_secs(3600),
        governor_poll_interval_above_hold: std::time::Duration::from_secs(3600),
        task_flush_interval: std::time::Duration::from_secs(3600),
        claim_lease_check_interval: std::time::Duration::from_secs(3600),
        port_check_interval: std::time::Duration::from_secs(3600),
        upgrade: Default::default(),
        ci_tick_interval: std::time::Duration::from_secs(3600),
        self_upgrade_wait: std::time::Duration::from_secs(600),
        settle_wake_interval: std::time::Duration::from_secs(3600),
        queue_nudge_debounce: std::time::Duration::from_secs(3600),
        take_over: false,
        host: None,
    };
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start second daemon");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    // Resumed: back to a live, non-terminal state, not left `stopped`.
    let resumed = support::wait_for_agent(&client, &agent.id, |a| {
        a.state != AgentState::Stopped && a.state.is_running()
    })
    .await;
    assert_ne!(resumed.state, AgentState::Stopped);

    running.shutdown();
    running.join().await.expect("join");
    drop(tmp);
}
