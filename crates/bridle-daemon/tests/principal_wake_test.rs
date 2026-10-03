//! `GET /v1/wake`: the daemon decides when a principal wakes: an unread message, or someone
//! else touching a task it created or claimed (`principal_wake.rs`).

mod support;

use std::time::Duration;

use bridle_api::types::{
    MessageKind, MessageQuery, NewTaskRequest, PrincipalWakeQuery, PrincipalWakeResponse,
    SendRequest, TaskKind, When,
};
use bridle_api::{Client, ClientError};

fn query(principal: &str, timeout_secs: u64) -> PrincipalWakeQuery {
    PrincipalWakeQuery {
        principal: principal.to_string(),
        timeout_secs: Some(timeout_secs),
    }
}

async fn send(client: &Client, to: &str, body: &str) -> String {
    client
        .send(&SendRequest {
            to: Some(to.to_string()),
            body: body.to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send")
        .remove(0)
        .id
}

#[tokio::test]
async fn an_unread_message_wakes_at_once() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let id = send(&daemon.client, "external:advisor", "hello").await;
    let got = tokio::time::timeout(
        Duration::from_secs(3),
        advisor.principal_wake(&query("external:advisor", 60)),
    )
    .await
    .expect("returns at once")
    .expect("wake");
    assert_eq!(got.reasons.len(), 1, "{got:?}");
    assert_eq!(got.reasons[0].reason, "message");
    assert_eq!(got.reasons[0].message_ids, vec![id]);
}

#[tokio::test]
async fn a_message_sent_after_the_call_starts_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let waiting = tokio::spawn(async move {
        advisor
            .principal_wake(&query("external:advisor", 60))
            .await
            .expect("wake")
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!waiting.is_finished());
    let id = send(&daemon.client, "external:advisor", "later").await;
    let got = tokio::time::timeout(Duration::from_secs(3), waiting)
        .await
        .expect("woken by the event")
        .expect("join");
    assert_eq!(got.reasons[0].message_ids, vec![id]);
}

#[tokio::test]
async fn it_times_out_empty_and_ignores_mail_for_others() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let _orch = daemon.external_client("orchestrator").await;
    send(&daemon.client, "external:orchestrator", "not yours").await;
    let got = advisor
        .principal_wake(&query("external:advisor", 1))
        .await
        .expect("wake");
    assert!(got.reasons.is_empty(), "{got:?}");
}

#[tokio::test]
async fn only_that_principal_or_the_human_may_wait() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let anon = Client::new(daemon.running.url.clone(), None);
    let other = daemon.external_client("orchestrator").await;
    for client in [&anon, &other] {
        let err = client
            .principal_wake(&query("external:advisor", 1))
            .await
            .expect_err("forbidden");
        assert!(
            matches!(err, ClientError::Api { status: 403, .. }),
            "got {err:?}"
        );
    }
    daemon
        .client
        .principal_wake(&query("external:advisor", 1))
        .await
        .expect("the human may");
}

async fn new_task(client: &Client) -> String {
    client
        .new_task(&NewTaskRequest {
            title: "t".to_string(),
            kind: TaskKind::Feature,
            body: String::new(),
            size: None,
            components: Vec::new(),
            for_human: false,
            priority: None,
        })
        .await
        .expect("new task")
        .id
}

/// Starts the advisor's wake, runs `act` once it is waiting, and returns the answer (after
/// `timeout_secs` at most).
async fn wake_after(
    advisor: &Client,
    timeout_secs: u64,
    act: impl std::future::Future<Output = ()>,
) -> PrincipalWakeResponse {
    let advisor = advisor.clone();
    let waiting = tokio::spawn(async move {
        advisor
            .principal_wake(&query("external:advisor", timeout_secs))
            .await
            .expect("wake")
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    act.await;
    waiting.await.expect("join")
}

#[tokio::test]
async fn another_principals_comment_on_a_task_it_created_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&advisor).await;
    let got = wake_after(&advisor, 10, async {
        daemon.client.note_task(&task, "hi").await.expect("note");
    })
    .await;
    assert_eq!(got.reasons.len(), 1, "{got:?}");
    assert_eq!(got.reasons[0].reason, "task");
    assert_eq!(got.reasons[0].task.as_deref(), Some(task.as_str()));
    assert_eq!(got.reasons[0].event.as_deref(), Some("task.note_added"));
}

#[tokio::test]
async fn a_state_change_on_a_claimed_task_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&daemon.client).await;
    daemon.client.plan_task(&task).await.expect("plan");
    advisor.claim_task(&task).await.expect("claim");
    let got = wake_after(&advisor, 10, async {
        daemon.client.note_task(&task, "x").await.expect("note");
    })
    .await;
    assert_eq!(got.reasons[0].task.as_deref(), Some(task.as_str()));
}

#[tokio::test]
async fn a_state_change_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&advisor).await;
    let got = wake_after(&advisor, 10, async {
        daemon.client.plan_task(&task).await.expect("plan");
    })
    .await;
    assert_eq!(got.reasons[0].event.as_deref(), Some("task.state"));
}

#[tokio::test]
async fn its_own_comment_does_not_wake_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&advisor).await;
    let got = wake_after(&advisor, 2, async {
        advisor.note_task(&task, "mine").await.expect("note");
    })
    .await;
    assert!(got.reasons.is_empty(), "{got:?}");
}

#[tokio::test]
async fn a_task_it_neither_created_nor_claimed_does_not_wake_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let task = new_task(&daemon.client).await;
    let got = wake_after(&advisor, 2, async {
        daemon.client.note_task(&task, "x").await.expect("note");
    })
    .await;
    assert!(got.reasons.is_empty(), "{got:?}");
}

#[tokio::test]
async fn the_wake_returns_the_text_and_marks_it_read_once() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let id = send(&daemon.client, "external:advisor", "hello there").await;
    let got = advisor
        .principal_wake(&query("external:advisor", 60))
        .await
        .expect("wake");
    let msgs = &got.reasons[0].messages;
    assert_eq!(msgs.len(), 1, "{got:?}");
    assert_eq!(
        (msgs[0].id.as_str(), msgs[0].body.as_str()),
        (&*id, "hello there")
    );
    // Read now, so the next wake has nothing to repeat.
    let again = advisor
        .principal_wake(&query("external:advisor", 1))
        .await
        .expect("wake");
    assert!(again.reasons.is_empty(), "{again:?}");
}

#[tokio::test]
async fn the_humans_wake_leaves_messages_unread() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let _advisor = daemon.external_client("advisor").await;
    send(&daemon.client, "external:advisor", "hello").await;
    let got = daemon
        .client
        .principal_wake(&query("external:advisor", 60))
        .await
        .expect("wake");
    assert!(got.reasons[0].messages.is_empty(), "{got:?}");
    let got = daemon
        .client
        .principal_wake(&query("external:advisor", 60))
        .await
        .expect("wake again");
    assert_eq!(got.reasons.len(), 1, "still unread: {got:?}");
}

#[tokio::test]
async fn a_non_human_inbox_marks_what_it_lists_read_and_cannot_unread() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let a = send(&daemon.client, "external:advisor", "one").await;
    let b = send(&daemon.client, "external:advisor", "two").await;
    // `show` of one message marks only that one.
    let shown = advisor
        .list_messages(&MessageQuery {
            to: Some("me".into()),
            id: Some(a.clone()),
            mark_read: true,
            ..Default::default()
        })
        .await
        .expect("show");
    assert_eq!(shown.len(), 1);
    let unread = |c: Client| async move {
        c.list_messages(&MessageQuery {
            to: Some("me".into()),
            unread: true,
            ..Default::default()
        })
        .await
        .expect("list")
        .into_iter()
        .map(|m| m.id)
        .collect::<Vec<_>>()
    };
    assert_eq!(unread(advisor.clone()).await, vec![b.clone()]);
    // A plain list marks nothing unless asked; the marking list takes the rest.
    advisor
        .list_messages(&MessageQuery {
            to: Some("me".into()),
            unread: true,
            mark_read: true,
            ..Default::default()
        })
        .await
        .expect("inbox");
    assert!(unread(advisor.clone()).await.is_empty());
    // An agent or external principal can't put it back; the human can.
    let err = advisor.mark_unread(&a).await.expect_err("refused");
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );
    daemon.client.mark_unread(&a).await.expect("human unread");
}
