//! The settle period (ny9u): a new task can't be claimed or listed ready for
//! `[tasks] settle`, and only the human, or the orchestrator with a reason,
//! may skip it. See docs/design/coordination.md.

mod support;
use support::ClientExt as _;

use bridle_api::ClientError;
use bridle_api::types::{NewTaskRequest, TaskKind};
use support::start_daemon_with_config;

#[tokio::test]
async fn a_new_task_settles_and_skip_settle_is_gated() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some("[tasks]\nsettle = \"5m\"\n")).await;
    let c = &daemon.client;
    let task = c
        .new_open_task(&NewTaskRequest {
            ticket: None,
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: "T".to_string(),
            kind: TaskKind::Chore,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new task");
    assert!(task.settle_until.is_some(), "settling from creation");
    c.plan_task(&task.id)
        .await
        .expect("the PM can still plan it");
    assert!(c.ready_tasks().await.expect("ready").is_empty());
    let err = c.claim_task(&task.id).await.expect_err("settling");
    assert!(err.to_string().contains("settling until"), "{err}");

    let advisor = daemon.external_client("advisor").await;
    let err = advisor.skip_settle(&task.id, "urgent").await.unwrap_err();
    assert!(matches!(err, ClientError::Api { status: 403, .. }), "{err}");

    let orch = daemon.external_client("orchestrator").await;
    let err = orch.skip_settle(&task.id, " ").await.unwrap_err();
    assert!(matches!(err, ClientError::Api { status: 400, .. }), "{err}");
    let skipped = orch
        .skip_settle(&task.id, "the human asked")
        .await
        .expect("orchestrator with a reason");
    assert!(skipped.settle_until.is_none());
    assert_eq!(c.ready_tasks().await.expect("ready").len(), 1);
    c.claim_task(&task.id).await.expect("claim after skip");
}

#[tokio::test]
async fn a_human_todo_is_claimed_at_creation_despite_settling() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some("[tasks]\nsettle = \"5m\"\n")).await;
    let task = daemon
        .client
        .new_open_task(&NewTaskRequest {
            ticket: None,
            for_human: true,
            priority: None,
            components: Vec::new(),
            title: "mine".to_string(),
            kind: TaskKind::Chore,
            body: String::new(),
            size: None,
        })
        .await
        .expect("new to-do");
    assert_eq!(task.claimed_by.as_deref(), Some("human"));
}
