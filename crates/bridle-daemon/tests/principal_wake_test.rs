//! `GET /v1/wake`: the daemon decides when a principal wakes; the one reason so far is an
//! unread message (`principal_wake.rs`).

mod support;

use std::time::Duration;

use bridle_api::types::{MessageKind, PrincipalWakeQuery, SendRequest, When};
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
