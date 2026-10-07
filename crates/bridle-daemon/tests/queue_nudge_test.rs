//! Every queue change tells the running manager (else the orchestrator) to re-read
//! the queue, one message per settled burst (f5ww).

mod support;
use support::ClientExt as _;

use std::time::Duration;

use bridle_api::types::{
    AgentState, MessageQuery, NewTaskRequest, SpawnRequest, TaskKind, Workdir,
};
use support::{default_overrides, start_daemon, wait_for_state};

const DEBOUNCE: Duration = Duration::from_millis(1000);
const BURST_DEBOUNCE: Duration = Duration::from_secs(5);
const GAP: Duration = Duration::from_millis(100);

async fn daemon() -> (support::TestDaemon, tempfile::TempDir) {
    daemon_with(DEBOUNCE).await
}

async fn daemon_with(debounce: Duration) -> (support::TestDaemon, tempfile::TempDir) {
    let mut o = default_overrides();
    o.queue_nudge_debounce = debounce;
    start_daemon(Some(o)).await
}

async fn task(daemon: &support::TestDaemon, title: &str) -> String {
    daemon
        .client
        .new_open_task(&NewTaskRequest {
            ticket: None,
            parent: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: title.to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task")
        .id
}

async fn nudges(daemon: &support::TestDaemon, to: &str) -> Vec<String> {
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
        .filter(|b| b.starts_with("queue updated"))
        .collect()
}

async fn settle() {
    tokio::time::sleep(DEBOUNCE * 3).await;
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

#[tokio::test]
async fn one_change_one_message_that_only_says_to_re_read_the_queue() {
    let (daemon, _tmp) = daemon().await;
    let mgr = spawn_manager(&daemon).await;
    let t = task(&daemon, "a").await;
    daemon.client.set_queue(vec![vec![t]]).await.expect("set");
    assert!(
        nudges(&daemon, &mgr).await.is_empty(),
        "not before it settles"
    );
    settle().await;
    let bodies = nudges(&daemon, &mgr).await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains("re-read `bridle queue` before you next start something"));
    assert!(bodies[0].contains("already in flight"));
}

#[tokio::test]
async fn a_burst_is_one_message_after_it_settles() {
    let (daemon, _tmp) = daemon_with(BURST_DEBOUNCE).await;
    let mgr = spawn_manager(&daemon).await;
    let a = task(&daemon, "a").await;
    let b = task(&daemon, "b").await;
    let c = task(&daemon, "c").await;
    // Each change lands inside the previous one's wait: a trailing edge. The wait is long against
    // the gap because a stalled CI runner (tr22: two nudges on ubuntu) can delay a request by
    // most of a second, and a stall longer than the wait sends a nudge before the burst ends.
    daemon.client.set_queue(vec![vec![a]]).await.expect("set");
    tokio::time::sleep(GAP).await;
    daemon.client.add_queue_tier(vec![b]).await.expect("add");
    tokio::time::sleep(GAP).await;
    daemon.client.add_queue_tier(vec![c]).await.expect("add");
    tokio::time::sleep(BURST_DEBOUNCE + Duration::from_secs(3)).await;
    assert_eq!(nudges(&daemon, &mgr).await.len(), 1);
}

#[tokio::test]
async fn no_running_manager_means_the_orchestrator_is_nudged() {
    let (daemon, _tmp) = daemon().await;
    let t = task(&daemon, "a").await;
    daemon.client.set_queue(vec![vec![t]]).await.expect("set");
    settle().await;
    assert_eq!(nudges(&daemon, "external:orchestrator").await.len(), 1);
}
