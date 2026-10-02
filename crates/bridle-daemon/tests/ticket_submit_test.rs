//! `POST /v1/tasks/submit`: any principal, visitors included, files an open task for the PM
//! to triage (ticket 93xm).

mod support;

use bridle_api::Client;
use bridle_api::types::{
    AgentState, DropTaskRequest, MessageQuery, SpawnRequest, SubmitTaskRequest, TaskKind,
    TaskState, TokenCreateRequest, Workdir,
};
use support::{start_daemon_with_config, wait_for_state};

const CONFIG: &str =
    "[roles.manager]\nautostart = false\n[roles.product-manager]\nmodel = \"sonnet\"\n";

async fn visitor(daemon: &support::TestDaemon) -> Client {
    let created = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "advisor".to_string(),
            machine: Some("nuc".to_string()),
        })
        .await
        .expect("visitor token");
    Client::new(daemon.running.url.clone(), Some(created.token))
}

fn req() -> SubmitTaskRequest {
    SubmitTaskRequest {
        title: "idea".to_string(),
        kind: TaskKind::Feature,
        body: "details".to_string(),
    }
}

async fn inbox(client: &Client, to: &str) -> Vec<String> {
    client
        .list_messages(&MessageQuery {
            to: Some(to.to_string()),
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
async fn a_visitor_submits_an_open_task_and_cannot_plan_or_claim_it() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some(CONFIG)).await;
    let pm = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "product-manager".to_string(),
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
    wait_for_state(&daemon.client, &pm.id, AgentState::Idle).await;
    let v = visitor(&daemon).await;
    let task = v.submit_task(&req()).await.expect("submit");
    assert_eq!(task.state, TaskState::Open);
    assert!(task.body.starts_with("submitted by external:advisor@nuc\n"));
    assert!(
        task.thread[0]
            .body
            .contains("submitted by external:advisor@nuc")
    );

    assert!(v.plan_task(&task.id).await.is_err());
    assert!(v.claim_task(&task.id).await.is_err());
    // It may comment on its own submission, but not on another task.
    let other = daemon
        .client
        .submit_task(&req())
        .await
        .expect("human submit");
    assert!(v.note_task(&task.id, "more").await.is_ok());
    assert!(v.note_task(&other.id, "more").await.is_err());

    let msgs = inbox(&daemon.client, &pm.id).await;
    assert!(
        msgs.iter()
            .any(|b| b.contains(&task.id) && b.contains("external:advisor@nuc")),
        "got {msgs:?}"
    );
}

#[tokio::test]
async fn dropping_a_submission_tells_the_submitter_why() {
    let (daemon, _tmp) = start_daemon_with_config(None, Some(CONFIG)).await;
    let v = visitor(&daemon).await;
    let task = v.submit_task(&req()).await.expect("submit");
    assert!(
        v.drop_task(
            &task.id,
            &DropTaskRequest {
                reason: "mine".to_string()
            }
        )
        .await
        .is_err()
    );
    daemon
        .client
        .drop_task(
            &task.id,
            &DropTaskRequest {
                reason: "duplicate of abcd".to_string(),
            },
        )
        .await
        .expect("drop");
    let msgs = inbox(&v, "external:advisor@nuc").await;
    assert!(
        msgs.iter().any(|b| b.contains("duplicate of abcd")),
        "got {msgs:?}"
    );
}
