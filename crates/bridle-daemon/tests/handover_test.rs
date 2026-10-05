//! The handover note record (orchestrator-supervision.md, section 7): every principal writes its
//! own, latest per identity wins, history stays readable.

mod support;

use bridle_api::types::{AgentState, SpawnRequest, Workdir};
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
            .latest_handover(None)
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

    let latest = orch
        .latest_handover(None)
        .await
        .expect("latest")
        .expect("some");
    assert_eq!(latest.body, "second note");
    let list = daemon.client.list_handovers(None).await.expect("list");
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
async fn the_unauthenticated_may_not_write_and_an_empty_note_is_refused() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let anon = Client::new(daemon.running.url.clone(), None);
    let err = anon.write_handover("nope").await.expect_err("refused");
    assert!(
        is_403(&err) || matches!(err, ClientError::Api { status: 401, .. }),
        "{err:?}"
    );
    let other = daemon.external_client("other").await;
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
    assert!(other.list_handovers(None).await.expect("read").is_empty());
}

#[tokio::test]
async fn each_identity_has_its_own_notes() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let aide = daemon.external_client("aide").await;
    let advisor = daemon.external_client("advisor").await;
    let doc = advisor.clone().with_advisor(Some("doc-review".to_string()));
    let w = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    support::wait_for_state(&daemon.client, &w.id, AgentState::Idle).await;
    let worker = daemon.agent_client(&w.id);

    let a = aide.write_handover("aide one").await.expect("aide");
    advisor.write_handover("advisor").await.expect("advisor");
    let d = doc.write_handover("doc").await.expect("named");
    let wk = worker.write_handover("worker").await.expect("worker");
    let a2 = aide.write_handover("aide two").await.expect("aide");
    assert_eq!(a.role, "aide");
    assert_eq!(a.created_by, "external:aide");
    assert_eq!(d.role, "advisor/doc-review");
    assert_eq!(wk.role, "agent:w1");

    let newest = |c: &Client, role: &str| {
        let (c, role) = (c.clone(), role.to_string());
        async move { c.latest_handover(Some(&role)).await.expect("latest") }
    };
    assert_eq!(newest(&aide, "aide").await.expect("some").id, a2.id);
    assert_eq!(
        newest(&aide, "advisor").await.expect("some").body,
        "advisor"
    );
    assert_eq!(
        newest(&aide, "advisor/doc-review").await.expect("some").id,
        d.id
    );
    assert_eq!(newest(&aide, "agent:w1").await.expect("some").id, wk.id);
    assert!(newest(&aide, "advisor/other").await.is_none());
    let list = aide.list_handovers(Some("aide")).await.expect("list");
    assert_eq!(list.len(), 2);
    // The request body carries no identity: the type has only `body`, so a client can't set one.
}
