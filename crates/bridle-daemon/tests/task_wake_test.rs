//! A new `open` task wakes the manager when no project manager is running
//! to triage it (coordination.md, "Waking the manager").

mod support;
use support::ClientExt as _;

use bridle_api::types::{
    AgentState, MessageQuery, NewTaskRequest, SpawnRequest, TaskKind, Workdir,
};
use support::{start_daemon_with_config, wait_for_state};

const CONFIG: &str =
    "[roles.manager]\nautostart = false\n[roles.project-manager]\nmodel = \"sonnet\"\n";

async fn spawn_role(daemon: &support::TestDaemon, role: &str, name: &str) -> String {
    let a = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: role.to_string(),
            name: Some(name.to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &a.id, AgentState::Idle).await;
    a.id
}

async fn file(daemon: &support::TestDaemon, title: &str) {
    daemon
        .client
        .new_open_task(&NewTaskRequest {
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: title.to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task");
}

async fn wakes(daemon: &support::TestDaemon, agent: &str) -> Vec<String> {
    daemon
        .client
        .list_messages(&MessageQuery {
            to: Some(agent.to_string()),
            from: Some("system".to_string()),
            unread: false,
            limit: None,
            ..Default::default()
        })
        .await
        .expect("messages")
        .into_iter()
        .map(|m| m.body)
        .collect()
}

#[tokio::test]
async fn no_pm_means_one_coalesced_message_to_the_manager() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some(CONFIG)).await;
    let mgr = spawn_role(&daemon, "manager", "mgr").await;
    file(&daemon, "first").await;
    file(&daemon, "second").await;
    file(&daemon, "third").await;
    let bodies = wakes(&daemon, &mgr).await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains("first"), "got {bodies:?}");
    assert!(bodies[0].contains("Plan it or queue it"), "got {bodies:?}");
}

#[tokio::test]
async fn a_running_pm_means_no_message() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some(CONFIG)).await;
    let mgr = spawn_role(&daemon, "manager", "mgr").await;
    spawn_role(&daemon, "project-manager", "pm").await;
    file(&daemon, "first").await;
    assert!(wakes(&daemon, &mgr).await.is_empty());
}

#[tokio::test]
async fn no_running_manager_means_the_message_goes_to_the_orchestrator() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some(CONFIG)).await;
    file(&daemon, "first").await;
    file(&daemon, "second").await;
    let bodies = wakes(&daemon, "external:orchestrator").await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains("first"), "got {bodies:?}");
}
