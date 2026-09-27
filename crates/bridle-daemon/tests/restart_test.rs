//! §9: restart reconciliation (docs/agent-host.md §4.7).
//!
//! Two daemons in one OS process would share a pid, which defeats
//! pid-liveness checks, so instead of literally running two overlapping
//! daemons we simulate exactly the state a crashed daemon would leave
//! behind: an `agents` row stuck `working` with the pid/start-time of a
//! still-live, but otherwise unsupervised, process. A fresh `start()` on
//! that workspace must find it, kill it, and mark it `lost`.

mod support;

use bridle_api::types::AgentState;
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
            name: "stale".to_string(),
            role: "worker".to_string(),
            model: "sonnet".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            workdir_kind: "repo".to_string(),
            cwd: repo.to_string_lossy().into_owned(),
            worktree: None,
            branch: None,
            created_by: "human".to_string(),
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
        stall_check_interval: std::time::Duration::from_secs(3600),
        tracker_interval: std::time::Duration::from_millis(200),
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

    // The process was actually terminated (not just marked lost in the
    // store): reap it and check it died by signal, not by living out its
    // 300s sleep. We're its real OS parent (spawned directly in this
    // test), so `kill(pid, 0)` alone isn't reliable here — a signaled
    // child sits as a zombie, and zombie pids still answer a liveness
    // probe, until whoever reaps it (us) calls `wait()`.
    let status = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
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
