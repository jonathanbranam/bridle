//! Incidents as a task kind, and the broadcast notice (docs/design/agent-host/incidents.md).

mod support;

use bridle_api::types::{
    AgentState, DoneTaskRequest, DropTaskRequest, EditTaskRequest, Message, MessageQuery,
    MessageState, NewTaskRequest, SpawnRequest, StopRequest, TaskKind, TaskState, Workdir,
};
use support::{TestDaemon, start_daemon, wait_for, wait_for_state};

async fn spawn(d: &TestDaemon, name: &str) -> String {
    let a = d
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
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
    wait_for_state(&d.client, &a.id, AgentState::Idle).await;
    a.id
}

async fn file(client: &bridle_api::Client, title: &str) -> String {
    client
        .new_task(&NewTaskRequest {
            for_human: false,
            priority: None,
            components: Vec::new(),
            title: title.to_string(),
            kind: TaskKind::Incident,
            body: "the build is red".to_string(),
            size: None,
        })
        .await
        .expect("file incident")
        .id
}

async fn notes(d: &TestDaemon, agent: &str) -> Vec<Message> {
    d.client
        .list_messages(&MessageQuery {
            to: Some(agent.to_string()),
            from: Some("system".to_string()),
            unread: false,
            limit: None,
        })
        .await
        .expect("messages")
}

fn only<'a>(msgs: &'a [Message], prefix: &str) -> Vec<&'a Message> {
    msgs.iter().filter(|m| m.body.starts_with(prefix)).collect()
}

#[tokio::test]
async fn anyone_files_a_potential_incident_but_only_the_owner_moves_it() {
    let (d, _tmp) = start_daemon(None).await;
    let w = spawn(&d, "w1").await;
    let orch = d.external_client("orchestrator").await;
    let advisor = d.external_client("advisor").await;
    let worker = d.agent_client(&w);

    let id = file(&worker, "main is red").await;
    let t = d.client.get_task(&id).await.expect("task");
    assert_eq!((t.kind, t.state), (TaskKind::Incident, TaskState::Open));

    for c in [&worker, &advisor] {
        assert!(c.plan_task(&id).await.is_err(), "plan must be refused");
        assert!(
            c.drop_task(
                &id,
                &DropTaskRequest {
                    reason: "no".to_string()
                }
            )
            .await
            .is_err()
        );
        assert!(
            c.done_task(&id, &DoneTaskRequest::default()).await.is_err(),
            "resolve must be refused"
        );
    }
    assert_eq!(
        d.client.get_task(&id).await.expect("task").state,
        TaskState::Open
    );

    // The orchestrator and the human may; no commit is needed to resolve.
    assert_eq!(
        orch.plan_task(&id).await.expect("plan").state,
        TaskState::Planned
    );
    let status = d.client.status().await.expect("status");
    assert_eq!(status.incidents.len(), 1);
    assert!(
        d.client
            .list_tasks()
            .await
            .expect("list")
            .iter()
            .any(|t| t.id == id)
    );
    assert!(d.client.ready_tasks().await.expect("ready").is_empty());
    let done = d
        .client
        .done_task(
            &id,
            &DoneTaskRequest {
                resolution: Some("reverted".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("resolve");
    assert_eq!(done.state, TaskState::Integrated);
    assert!(done.thread.iter().any(|e| e.body == "resolution: reverted"));
    assert!(
        d.client
            .status()
            .await
            .expect("status")
            .incidents
            .is_empty()
    );
}

#[tokio::test]
async fn a_running_agent_gets_the_notice_and_then_a_resolved_note() {
    let (d, _tmp) = start_daemon(None).await;
    let w = spawn(&d, "w1").await;
    let orch = d.external_client("orchestrator").await;
    let id = file(&orch, "main is red").await;
    orch.plan_task(&id).await.expect("plan");

    let got = wait_for("notice delivered", || async {
        let n = notes(&d, &w).await;
        n.iter()
            .any(|m| m.state != MessageState::Pending && m.state != MessageState::Held)
            .then_some(n)
    })
    .await;
    assert_eq!(only(&got, "Incident").len(), 1);
    assert!(got[0].body.contains("the build is red"));
    assert_eq!(got[0].incident_task.as_deref(), Some(id.as_str()));

    orch.done_task(
        &id,
        &DoneTaskRequest {
            resolution: Some("reverted".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("resolve");
    let n = notes(&d, &w).await;
    let resolved = only(&n, &format!("Incident {id} resolved: reverted"));
    assert_eq!(resolved.len(), 1, "got {n:?}");
    assert!(resolved[0].incident_task.is_none());
}

#[tokio::test]
async fn an_agent_started_while_active_gets_the_notice() {
    let (d, _tmp) = start_daemon(None).await;
    let orch = d.external_client("orchestrator").await;
    let id = file(&orch, "main is red").await;
    orch.plan_task(&id).await.expect("plan");

    let w = spawn(&d, "late").await;
    let n = notes(&d, &w).await;
    assert_eq!(only(&n, &format!("Incident {id}:")).len(), 1, "got {n:?}");
}

#[tokio::test]
async fn an_undelivered_notice_is_dropped_on_resolve_and_never_announced() {
    let (d, _tmp) = start_daemon(None).await;
    let w = spawn(&d, "w1").await;
    d.client
        .stop(&w, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for_state(&d.client, &w, AgentState::Stopped).await;
    let orch = d.external_client("orchestrator").await;
    let id = file(&orch, "main is red").await;
    orch.plan_task(&id).await.expect("plan");
    let n = notes(&d, &w).await;
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].state, MessageState::Pending);

    orch.drop_task(
        &id,
        &DropTaskRequest {
            reason: "false alarm".to_string(),
        },
    )
    .await
    .expect("drop while active");
    let n = notes(&d, &w).await;
    assert_eq!(n.len(), 1, "no resolved note for an unseen notice: {n:?}");
    assert_eq!(n[0].state, MessageState::Dropped);
}

#[tokio::test]
async fn editing_an_active_incident_updates_the_pending_notice_in_place() {
    let (d, _tmp) = start_daemon(None).await;
    let w = spawn(&d, "w1").await;
    d.client
        .stop(&w, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for_state(&d.client, &w, AgentState::Stopped).await;
    let orch = d.external_client("orchestrator").await;
    let id = file(&orch, "main is red").await;
    orch.plan_task(&id).await.expect("plan");
    orch.edit_task(
        &id,
        &EditTaskRequest {
            body: Some("now it's green-ish".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("edit");
    let n = notes(&d, &w).await;
    assert_eq!(n.len(), 1, "got {n:?}");
    assert!(n[0].body.contains("green-ish"));
}
