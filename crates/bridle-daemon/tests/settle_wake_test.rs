//! A task whose settle period ends is announced to the running manager once
//! (ny9u follow-up). Already-settled and dependency-blocked tasks aren't.

mod support;
use support::ClientExt as _;

use std::time::Duration;

use bridle_api::types::{
    AgentState, EdgeKind, MessageQuery, NewEdgeRequest, NewTaskRequest, SpawnRequest, TaskKind,
    Workdir,
};
use support::{default_overrides, start_daemon_with_config, wait_for_state};

async fn daemon(settle: &str) -> (support::TestDaemon, tempfile::TempDir) {
    let mut o = default_overrides();
    o.settle_wake_interval = Duration::from_millis(200);
    start_daemon_with_config(Some(o), Some(&format!("[tasks]\nsettle = \"{settle}\"\n"))).await
}

async fn planned_task(daemon: &support::TestDaemon, title: &str) -> String {
    let id = daemon
        .client
        .new_open_task(&NewTaskRequest {
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: title.to_string(),
            kind: TaskKind::Chore,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task")
        .id;
    daemon.client.plan_task(&id).await.expect("plan");
    id
}

async fn spawn_manager(daemon: &support::TestDaemon) -> String {
    let a = daemon
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
        .expect("spawn");
    wait_for_state(&daemon.client, &a.id, AgentState::Idle).await;
    a.id
}

async fn notes(daemon: &support::TestDaemon, to: &str) -> Vec<String> {
    daemon
        .client
        .list_messages(&MessageQuery {
            to: Some(to.to_string()),
            from: Some("system".to_string()),
            unread: false,
            limit: None,
            ..Default::default()
        })
        .await
        .expect("messages")
        .into_iter()
        .map(|m| m.body)
        .filter(|b| b.ends_with("is now startable"))
        .collect()
}

#[tokio::test]
async fn crossing_the_settle_time_notes_the_manager_once() {
    let (daemon, _tmp) = daemon("2s").await;
    let mgr = spawn_manager(&daemon).await;
    let t = planned_task(&daemon, "T").await;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert!(notes(&daemon, &mgr).await.is_empty(), "still settling");
    tokio::time::sleep(Duration::from_millis(2000)).await;
    assert_eq!(
        notes(&daemon, &mgr).await,
        vec![format!("{t} is now startable")]
    );
}

#[tokio::test]
async fn an_already_settled_task_makes_no_note() {
    let (daemon, _tmp) = daemon("0").await;
    let mgr = spawn_manager(&daemon).await;
    planned_task(&daemon, "T").await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert!(notes(&daemon, &mgr).await.is_empty());
}

#[tokio::test]
async fn a_task_blocked_by_a_dependency_makes_no_note() {
    let (daemon, _tmp) = daemon("1s").await;
    let mgr = spawn_manager(&daemon).await;
    let blocker = planned_task(&daemon, "blocker").await;
    let blocked = planned_task(&daemon, "blocked").await;
    daemon
        .client
        .add_edge(&NewEdgeRequest {
            from: blocker.clone(),
            to: blocked.clone(),
            kind: EdgeKind::Blocks,
        })
        .await
        .expect("edge");
    tokio::time::sleep(Duration::from_millis(2500)).await;
    let got = notes(&daemon, &mgr).await;
    assert_eq!(got, vec![format!("{blocker} is now startable")], "{got:?}");
}
