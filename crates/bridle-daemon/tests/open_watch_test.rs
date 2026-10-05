//! A readied task tells the running PM (else the orchestrator) it is open, one message per
//! settled burst; one left open and unplanned too long goes back to pending (xz4f).

mod support;
use support::ClientExt as _;

use std::time::Duration;

use bridle_api::types::{
    AgentState, MessageQuery, NewTaskRequest, SpawnRequest, TaskKind, TaskState, Workdir,
};
use support::{default_overrides, start_daemon_with_config, wait_for_state};

const DEBOUNCE: Duration = Duration::from_millis(1000);
const CONFIG: &str =
    "[roles.manager]\nautostart = false\n[roles.project-manager]\nmodel = \"sonnet\"\n";

async fn daemon(config: &str) -> (support::TestDaemon, tempfile::TempDir) {
    let mut o = default_overrides();
    o.open_watch_debounce = DEBOUNCE;
    o.settle_wake_interval = Duration::from_millis(200);
    start_daemon_with_config(Some(o), Some(config)).await
}

fn req(title: &str) -> NewTaskRequest {
    NewTaskRequest {
        for_human: false,
        priority: None,
        components: Vec::new(),
        title: title.to_string(),
        kind: TaskKind::Feature,
        body: String::new(),
        size: None,
    }
}

async fn open_task(daemon: &support::TestDaemon, title: &str) -> String {
    daemon
        .client
        .new_open_task(&req(title))
        .await
        .expect("new task")
        .id
}

async fn messages(daemon: &support::TestDaemon, to: &str) -> Vec<String> {
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
        .filter(|b| b.contains("not yet planned") || b.contains("never planned"))
        .collect()
}

async fn settle() {
    tokio::time::sleep(DEBOUNCE * 3).await;
}

async fn spawn_pm(daemon: &support::TestDaemon) -> String {
    let a = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "project-manager".to_string(),
            name: Some("pm".to_string()),
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
async fn a_readied_task_is_named_to_the_running_pm_once() {
    let (daemon, _tmp) = daemon(CONFIG).await;
    let pm = spawn_pm(&daemon).await;
    let a = open_task(&daemon, "first").await;
    assert!(
        messages(&daemon, &pm).await.is_empty(),
        "not before it settles"
    );
    settle().await;
    let bodies = messages(&daemon, &pm).await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains(&a), "got {bodies:?}");
    assert!(messages(&daemon, "external:orchestrator").await.is_empty());
}

#[tokio::test]
async fn a_burst_is_one_message_naming_every_task() {
    let (daemon, _tmp) = daemon(CONFIG).await;
    let pm = spawn_pm(&daemon).await;
    let a = open_task(&daemon, "first").await;
    let b = open_task(&daemon, "second").await;
    settle().await;
    let bodies = messages(&daemon, &pm).await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(
        bodies[0].contains(&a) && bodies[0].contains(&b),
        "got {bodies:?}"
    );
}

#[tokio::test]
async fn no_running_pm_means_the_orchestrator_is_told() {
    let (daemon, _tmp) = daemon(CONFIG).await;
    let a = open_task(&daemon, "first").await;
    settle().await;
    let bodies = messages(&daemon, "external:orchestrator").await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains(&a), "got {bodies:?}");
}

#[tokio::test]
async fn a_task_open_too_long_goes_back_to_pending_with_a_comment() {
    let (daemon, _tmp) = daemon(&format!("{CONFIG}[tasks]\nopen_stale = \"1s\"\n")).await;
    let a = open_task(&daemon, "stale").await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    let t = daemon.client.get_task(&a).await.expect("get");
    assert_eq!(t.state, TaskState::Pending);
    assert!(
        t.thread.iter().any(|e| e.body.contains("never planned")),
        "thread: {:?}",
        t.thread
    );
    let bodies = messages(&daemon, "external:orchestrator").await;
    assert!(
        bodies
            .iter()
            .any(|b| b.contains("never planned") && b.contains(&a)),
        "got {bodies:?}"
    );
}

#[tokio::test]
async fn a_task_with_an_open_question_stays_open() {
    let (daemon, _tmp) = daemon(&format!("{CONFIG}[tasks]\nopen_stale = \"1s\"\n")).await;
    let a = open_task(&daemon, "asked").await;
    daemon
        .client
        .ask_question(&a, "which way?", Some("human"))
        .await
        .expect("ask");
    tokio::time::sleep(Duration::from_secs(3)).await;
    let t = daemon.client.get_task(&a).await.expect("get");
    assert_eq!(t.state, TaskState::Open);
}
