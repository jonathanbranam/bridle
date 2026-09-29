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
        let err = client.orchestrator_wake().await.expect_err("forbidden");
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
    let got = tokio::time::timeout(std::time::Duration::from_secs(30), orch.orchestrator_wake())
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
