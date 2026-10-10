//! The message audit trail (ticket 9aj2): `message.*` events carry the channel, and the events
//! query filters by message id and recipient.

mod support;

use std::time::Duration;

use bridle_api::Client;
use bridle_api::types::{
    EventQuery, MessageKind, MessageQuery, PrincipalWakeQuery, SendRequest, When, event_kind,
};

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

async fn events(
    client: &Client,
    message: Option<&str>,
    to: Option<&str>,
) -> Vec<bridle_api::types::Event> {
    client
        .events(&EventQuery {
            kind: Some("message.".into()),
            message: message.map(str::to_string),
            to: to.map(str::to_string),
            ..Default::default()
        })
        .await
        .expect("events")
}

#[tokio::test]
async fn a_waiter_delivery_and_its_acknowledgement_are_separate_events() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let id = send(&daemon.client, "external:advisor", "hello").await;
    advisor
        .principal_wake(&PrincipalWakeQuery {
            principal: "external:advisor".into(),
            timeout_secs: Some(5),
            session: Some("s-1".into()),
            pid: Some(4242),
        })
        .await
        .expect("wake");
    let evs = events(&daemon.client, Some(&id), None).await;
    let delivered = evs
        .iter()
        .find(|e| e.kind == event_kind::MESSAGE_DELIVERED)
        .expect("delivered");
    assert_eq!(delivered.data["channel"], "waiter");
    assert_eq!(delivered.data["pid"], 4242);
    assert_eq!(delivered.data["session"], "s-1");
    assert!(
        !evs.iter().any(|e| e.kind == event_kind::MESSAGE_READ),
        "not acknowledged yet"
    );
    // The same session waiting again is the acknowledgement.
    advisor
        .principal_wake(&PrincipalWakeQuery {
            principal: "external:advisor".into(),
            timeout_secs: Some(1),
            session: Some("s-1".into()),
            pid: Some(4243),
        })
        .await
        .expect("wake");

    let evs = events(&daemon.client, Some(&id), None).await;
    let sent = evs
        .iter()
        .find(|e| e.kind == event_kind::MESSAGE_SENT)
        .expect("sent");
    assert_eq!(sent.data["channel"], "ui");
    let read = evs
        .iter()
        .find(|e| e.kind == event_kind::MESSAGE_READ)
        .expect("read");
    assert_eq!(read.actor, "external:advisor");
    assert_eq!(read.data["channel"], "waiter");
    assert_eq!(read.data["acknowledged"], true);
    assert_eq!(read.data["session"], "s-1");
}

#[tokio::test]
async fn an_inbox_read_records_the_inbox_channel() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let advisor = daemon.external_client("advisor").await;
    let id = send(&daemon.client, "external:advisor", "hello").await;
    advisor
        .list_messages(&MessageQuery {
            to: Some("me".into()),
            unread: true,
            mark_read: true,
            ..Default::default()
        })
        .await
        .expect("inbox");
    let evs = events(&daemon.client, Some(&id), None).await;
    let read = evs
        .iter()
        .find(|e| e.kind == event_kind::MESSAGE_READ)
        .expect("read");
    assert_eq!(read.data["channel"], "inbox");
}

#[tokio::test]
async fn events_filter_by_message_and_by_recipient() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let _advisor = daemon.external_client("advisor").await;
    let _orch = daemon.external_client("orchestrator").await;
    let a = send(&daemon.client, "external:advisor", "one").await;
    let b = send(&daemon.client, "external:orchestrator", "two").await;

    let only_a = events(&daemon.client, Some(&a), None).await;
    assert!(!only_a.is_empty());
    assert!(only_a.iter().all(|e| e.data["message"] == a.as_str()));

    let to_b = events(&daemon.client, None, Some("external:orchestrator")).await;
    assert!(!to_b.is_empty());
    assert!(
        to_b.iter().all(|e| e.data["message"] == b.as_str()),
        "{to_b:?}"
    );
}

#[tokio::test]
async fn messages_since_secs_limits_the_window() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let _advisor = daemon.external_client("advisor").await;
    send(&daemon.client, "external:advisor", "one").await;
    let list = |since_secs| {
        let client = daemon.client.clone();
        async move {
            client
                .list_messages(&MessageQuery {
                    to: Some("external:advisor".into()),
                    since_secs,
                    ..Default::default()
                })
                .await
                .expect("list")
        }
    };
    assert_eq!(list(Some(3600)).await.len(), 1);
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert!(list(Some(0)).await.is_empty());
}

#[test]
fn old_events_without_a_channel_still_parse() {
    let ev: bridle_api::types::Event = serde_json::from_value(serde_json::json!({
        "seq": 1, "ts": "2026-10-01T00:00:00Z", "kind": "message.read",
        "actor": "external:aide", "agent": null, "data": {"message": "m-1"}
    }))
    .expect("parses");
    assert!(ev.data["channel"].is_null());
    // And an old client's query, without the new fields, still deserializes.
    let q: EventQuery = serde_json::from_str("{}").expect("query");
    assert!(q.message.is_none() && q.to.is_none());
}
