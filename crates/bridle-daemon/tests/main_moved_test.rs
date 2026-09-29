//! A landing tells the other running workers that main moved
//! (impact-and-conflicts.md, conflict protocol step 4): one message each,
//! none to the agent whose task landed.

mod support;

use bridle_api::types::{
    AgentState, DoneTaskRequest, MessageQuery, NewTaskRequest, SpawnRequest, TaskKind, Workdir,
};
use support::{start_daemon, wait_for_state};

async fn worker(daemon: &support::TestDaemon, name: &str) -> String {
    let a = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some(name.to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
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

async fn notices(daemon: &support::TestDaemon, agent: &str) -> Vec<String> {
    daemon
        .client
        .list_messages(&MessageQuery {
            to: Some(agent.to_string()),
            from: Some("system".to_string()),
            unread: false,
            limit: None,
        })
        .await
        .expect("messages")
        .into_iter()
        .map(|m| m.body)
        .collect()
}

#[tokio::test]
async fn landing_notifies_the_other_worker_once_and_not_the_lander() {
    let (daemon, _tmp) = start_daemon(None).await;
    let lander = worker(&daemon, "w1").await;
    let other = worker(&daemon, "w2").await;
    let task = daemon
        .client
        .new_task(&NewTaskRequest {
            components: Vec::new(),
            title: "Landed".to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task");
    let head = String::from_utf8(
        std::process::Command::new("git")
            .arg("-C")
            .arg(&daemon.repo)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("rev-parse")
            .stdout,
    )
    .expect("utf8")
    .trim()
    .to_string();
    daemon
        .client
        .done_task(
            &task.id,
            &DoneTaskRequest {
                commit: head.clone(),
                branch: Some("bridle/w1".to_string()),
            },
        )
        .await
        .expect("done");

    let bodies = notices(&daemon, &other).await;
    assert_eq!(bodies.len(), 1, "got {bodies:?}");
    assert!(bodies[0].contains("main moved"), "got {bodies:?}");
    assert!(bodies[0].contains(&task.id), "got {bodies:?}");
    assert!(bodies[0].contains(&head), "got {bodies:?}");
    assert!(notices(&daemon, &lander).await.is_empty());
}
