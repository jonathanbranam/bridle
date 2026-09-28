//! The built-in manager autostarts with the daemon (docs/design/agent-host/daemon.md,
//! startup); explicit config overrides the default.

mod support;

use bridle_api::types::AgentState;
use std::time::Duration;
use support::{start_daemon_verbatim_config, wait_for_agent, wait_for_state};

#[tokio::test]
async fn default_config_starts_a_manager() {
    let (daemon, _tmp) = start_daemon_verbatim_config(None, None).await;
    let agents = daemon.client.list_agents().await.expect("list");
    assert_eq!(agents.len(), 1, "{agents:?}");
    assert_eq!(agents[0].name, "manager");
    assert_eq!(agents[0].role, "manager");
}

#[tokio::test]
async fn autostart_false_on_the_manager_suppresses_it() {
    let (daemon, _tmp) =
        start_daemon_verbatim_config(None, Some("[roles.manager]\nautostart = false\n")).await;
    assert!(daemon.client.list_agents().await.expect("list").is_empty());
}

#[tokio::test]
async fn a_resumed_manager_is_not_spawned_twice() {
    let (daemon, tmp) = start_daemon_verbatim_config(None, None).await;
    let workspace = daemon.workspace.clone();
    let repo = daemon.repo.clone();
    let first = wait_for_state(&daemon.client, "manager", AgentState::Idle).await;
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");

    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = support::default_overrides();
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    overrides.governor_interval = Duration::from_secs(3600);
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start second daemon");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    let agents = client.list_agents().await.expect("list");
    assert_eq!(agents.len(), 1, "{agents:?}");
    assert_eq!(agents[0].id, first.id);
    wait_for_agent(&client, &first.id, |a| a.state.is_running()).await;

    running.shutdown();
    running.join().await.expect("join");
}

#[tokio::test]
async fn a_differently_named_manager_suppresses_autostart() {
    let (daemon, tmp) =
        start_daemon_verbatim_config(None, Some("[roles.manager]\nautostart = false\n")).await;
    let workspace = daemon.workspace.clone();
    let repo = daemon.repo.clone();
    let req = bridle_api::types::SpawnRequest {
        role: "manager".to_string(),
        name: Some("manager-2".to_string()),
        prompt: None,
        workdir: None,
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
        components: Vec::new(),
    };
    let first = daemon.client.spawn(&req).await.expect("spawn manager-2");
    wait_for_state(&daemon.client, "manager-2", AgentState::Idle).await;
    daemon.running.shutdown();
    daemon.running.join().await.expect("join");

    // Autostart is back on (the default) for the restart.
    std::fs::write(repo.join(".bridle/config.toml"), "").expect("rewrite config");
    let opts = bridle_daemon::ServeOptions {
        repo,
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = support::default_overrides();
    overrides.bridle_home = Some(support::machine_home_dir(tmp.path()));
    overrides.governor_interval = Duration::from_secs(3600);
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start second daemon");
    let token =
        std::fs::read_to_string(workspace.join(".bridle/tokens/human")).expect("human token");
    let client = bridle_api::Client::new(running.url.clone(), Some(token.trim().to_string()));

    let agents = client.list_agents().await.expect("list");
    assert_eq!(agents.len(), 1, "{agents:?}");
    assert_eq!(agents[0].id, first.id);

    running.shutdown();
    running.join().await.expect("join");
}
