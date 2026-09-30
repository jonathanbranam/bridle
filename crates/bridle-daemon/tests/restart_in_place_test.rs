//! `POST /v1/restart` (docs/design/agent-host/daemon.md, "Restart in place"). The exec itself is
//! `run`'s and can't happen inside a test process; what's checked here is everything around it:
//! who may ask, the quiet-point wait, the shutdown with the restart flag set, and the resume of
//! every running agent (a plain worker, not a `resume_on_restart` role) by the next start.

mod support;

use bridle_api::types::{AgentState, MessageQuery, RestartRequest, SpawnRequest, Workdir};

fn spawn_req(name: &str, prompt: Option<&str>) -> SpawnRequest {
    SpawnRequest {
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some(name.to_string()),
        prompt: prompt.map(str::to_string),
        workdir: Some(Workdir::Repo),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    }
}

#[tokio::test]
async fn only_the_human_and_the_orchestrator_may_restart() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&spawn_req("w", None))
        .await
        .expect("spawn");
    let other = daemon.external_client("someone-else").await;
    let err = other
        .restart(&RestartRequest::default())
        .await
        .expect_err("external non-orchestrator refused");
    assert!(err.to_string().contains("forbidden") || err.to_string().contains("may restart"));
    let err = daemon
        .agent_client(&agent.id)
        .restart(&RestartRequest::default())
        .await
        .expect_err("agent refused");
    assert!(err.to_string().contains("may restart"), "{err}");
    assert!(!daemon.running.restart_requested());
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_busy_agent_means_no_restart_after_the_wait() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&spawn_req("busy", Some("SLEEP 4")))
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;
    let orch = daemon.external_client("orchestrator").await;
    let err = orch
        .restart(&RestartRequest { wait_secs: Some(1) })
        .await
        .expect_err("busy");
    assert!(err.to_string().contains("still busy: busy"), "{err}");
    assert!(!daemon.running.restart_requested());
    // Still up and serving, agent untouched.
    let a = daemon.client.get_agent(&agent.id).await.expect("agent");
    assert_eq!(a.state, AgentState::Working);
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn a_restart_stops_the_daemon_and_the_next_start_resumes_the_worker_with_a_message() {
    let (daemon, tmp) = support::start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&spawn_req("w1", None))
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let orch = daemon.external_client("orchestrator").await;
    let reply = orch
        .restart(&RestartRequest {
            wait_secs: Some(30),
        })
        .await
        .expect("restart accepted");
    assert_eq!(reply.agents, vec!["w1".to_string()]);
    assert!(daemon.running.restart_requested());
    let (workspace, repo) = (daemon.workspace.clone(), daemon.repo.clone());
    daemon.running.join().await.expect("join");

    // What `run` does after the exec: a fresh start on the same workspace.
    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = support::default_overrides();
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start again");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    let resumed = support::wait_for_agent(&client, &agent.id, |a| a.state.is_running()).await;
    assert_ne!(resumed.state, AgentState::Stopped);
    support::wait_for("the restart note reaches the worker", || async {
        client
            .list_messages(&MessageQuery {
                to: Some(agent.id.clone()),
                ..Default::default()
            })
            .await
            .ok()?
            .into_iter()
            .find(|m| m.from == "system" && m.body.contains("restarted for an upgrade"))
    })
    .await;

    running.shutdown();
    running.join().await.expect("join");
}
