//! The handover note record (orchestrator-supervision.md, section 7): who may write, latest
//! wins, history stays readable.

mod support;

use bridle_api::{Client, ClientError};

fn is_403(e: &ClientError) -> bool {
    matches!(e, ClientError::Api { status: 403, .. })
}

#[tokio::test]
async fn write_list_show_and_latest_wins() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    assert!(
        daemon
            .client
            .latest_handover()
            .await
            .expect("latest")
            .is_none()
    );

    let orch = daemon.external_client("orchestrator").await;
    let first = orch.write_handover("first note").await.expect("write");
    assert_eq!(first.id, "h-0001");
    assert_eq!(first.created_by, "external:orchestrator");
    assert_eq!(first.role, "orchestrator");
    let second = daemon
        .client
        .write_handover("second note")
        .await
        .expect("human writes");
    assert_eq!(second.created_by, "human");

    let latest = orch.latest_handover().await.expect("latest").expect("some");
    assert_eq!(latest.body, "second note");
    let list = daemon.client.list_handovers().await.expect("list");
    assert_eq!(
        list.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
        ["h-0002", "h-0001"]
    );
    let shown = daemon.client.get_handover("h-0001").await.expect("show");
    assert_eq!(shown.body, "first note");
    let err = daemon
        .client
        .get_handover("h-0099")
        .await
        .expect_err("missing");
    assert!(
        matches!(err, ClientError::Api { status: 404, .. }),
        "{err:?}"
    );
}

#[tokio::test]
async fn only_the_human_and_the_orchestrator_may_write() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let anon = Client::new(daemon.running.url.clone(), None);
    let other = daemon.external_client("other").await;
    for client in [&anon, &other] {
        let err = client.write_handover("nope").await.expect_err("refused");
        assert!(
            is_403(&err) || matches!(err, ClientError::Api { status: 401, .. }),
            "{err:?}"
        );
    }
    let err = daemon
        .client
        .write_handover("  \n")
        .await
        .expect_err("empty");
    assert!(
        matches!(err, ClientError::Api { status: 400, .. }),
        "{err:?}"
    );
    // Reading is open to any principal.
    assert!(other.list_handovers().await.expect("read").is_empty());
}
