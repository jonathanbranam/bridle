//! `GET /v1/orchestrator/wake` (orchestrator-supervision.md, section 5): the orchestrator's
//! long poll. The conditions themselves are unit-tested in `wake.rs`.

mod support;

use bridle_api::types::{MessageKind, SendRequest, When};
use bridle_api::{Client, ClientError};

#[tokio::test]
async fn only_the_orchestrator_may_wait_and_a_message_wakes_it() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let anon = Client::new(daemon.running.url.clone(), None);
    for client in [
        &daemon.client,
        &anon,
        &daemon.external_client("other").await,
    ] {
        let err = client.orchestrator_wake(None).await.expect_err("forbidden");
        assert!(
            matches!(err, ClientError::Api { status: 403, .. }),
            "got {err:?}"
        );
    }

    let orch = daemon.external_client("orchestrator").await;
    daemon
        .client
        .send(&SendRequest {
            to: Some("external:orchestrator".to_string()),
            body: "plan is ready".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send");
    // The wake loop ticks every 10 s.
    let got = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        orch.orchestrator_wake(None),
    )
    .await
    .expect("woken within a couple of ticks")
    .expect("wake");
    assert_eq!(got.wakes.len(), 1, "{got:?}");
    assert_eq!(got.wakes[0].reason, "message");
    assert_eq!(got.wakes[0].detail["body"], "plan is ready");
}

#[tokio::test]
async fn handover_done_is_for_the_human_and_the_orchestrator() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let anon = Client::new(daemon.running.url.clone(), None);
    let err = anon.handover_done().await.expect_err("no token");
    assert!(
        matches!(err, ClientError::Api { status: 401, .. }),
        "got {err:?}"
    );
    let err = daemon
        .external_client("other")
        .await
        .handover_done()
        .await
        .expect_err("forbidden");
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "got {err:?}"
    );
    daemon.client.handover_done().await.expect("human");
    let orch = daemon.external_client("orchestrator").await;
    orch.handover_done().await.expect("orchestrator");
}

#[tokio::test]
async fn agent_wake_serves_the_orchestrators_wakes_with_the_same_payload() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let orch = daemon.external_client("orchestrator").await;
    let query = || bridle_api::types::PrincipalWakeQuery {
        principal: "external:orchestrator".to_string(),
        timeout_secs: Some(30),
        session: None,
    };

    // Only the orchestrator drains its queue: not another principal, not the human.
    for client in [&daemon.client, &daemon.external_client("other").await] {
        let err = client
            .principal_wake(&query())
            .await
            .expect_err("forbidden");
        assert!(
            matches!(err, ClientError::Api { status: 403, .. }),
            "got {err:?}"
        );
    }

    daemon
        .client
        .send(&SendRequest {
            to: Some("external:orchestrator".to_string()),
            body: "plan is ready".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send");
    // The wake loop ticks every 10 s.
    let got = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        orch.principal_wake(&query()),
    )
    .await
    .expect("woken within a couple of ticks")
    .expect("wake");
    assert_eq!(got.reasons.len(), 1, "{got:?}");
    assert_eq!(got.reasons[0].reason, "message");
    let detail = got.reasons[0].detail.as_ref().expect("detail");
    assert_eq!(detail["body"], "plan is ready");
    assert!(
        got.reasons[0]
            .text
            .as_deref()
            .unwrap()
            .starts_with("message ")
    );

    // Delivered once, and read: neither route returns it again.
    let again = orch
        .principal_wake(&bridle_api::types::PrincipalWakeQuery {
            timeout_secs: Some(1),
            ..query()
        })
        .await
        .expect("wake");
    assert!(again.reasons.is_empty(), "{again:?}");
}
