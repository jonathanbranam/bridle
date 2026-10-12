//! `bridle project drain`'s hold (kx5u K4): no spawns or claims while held, and `--undo` lifts it.

mod support;

use bridle_api::client::ClientError;
use bridle_api::types::{SpawnRequest, Workdir};

fn worker(name: &str) -> SpawnRequest {
    SpawnRequest {
        components: Vec::new(),
        role: "worker".to_string(),
        name: Some(name.to_string()),
        prompt: Some("hi".to_string()),
        workdir: Some(Workdir::Repo),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ..Default::default()
    }
}

#[tokio::test]
async fn a_held_project_refuses_spawns_and_claims_until_the_hold_is_lifted() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let c = &daemon.client;
    assert!(!c.status().await.expect("status").held);

    c.hold_project(true).await.expect("hold");
    assert!(c.status().await.expect("status").held);
    let err = c.spawn(&worker("w1")).await.expect_err("held spawn");
    assert!(
        matches!(err, ClientError::Api { status: 409, .. }),
        "{err:?}"
    );
    let err = c.claim_task("br-any").await.expect_err("held claim");
    assert!(
        matches!(err, ClientError::Api { status: 409, .. }),
        "{err:?}"
    );

    c.hold_project(false).await.expect("undo");
    assert!(!c.status().await.expect("status").held);
    c.spawn(&worker("w2")).await.expect("spawn after undo");
}
