//! §1: health, status, auth. See docs/design/agent-host/principals.md, api.md.

mod support;

use bridle_api::types::SpawnRequest;
use bridle_api::{Client, ClientError};

#[tokio::test]
async fn health_needs_no_auth_and_status_needs_a_token() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let health = daemon.client.health().await.expect("health");
    assert!(health.ok);

    let anon = Client::new(daemon.running.url.clone(), None);
    assert!(anon.health().await.is_ok(), "health should need no auth");

    let status = daemon.client.status().await.expect("status");
    assert_eq!(status.principal, "human");
    assert!(!status.daemon.project.is_empty());
}

#[tokio::test]
async fn missing_or_invalid_token_is_401() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let anon = Client::new(daemon.running.url.clone(), None);
    let err = anon.status().await.unwrap_err();
    match err {
        ClientError::Api { status, code, .. } => {
            assert_eq!(status, 401);
            assert_eq!(code, "unauthorized");
        }
        other => panic!("expected 401, got {other:?}"),
    }

    let bad = Client::new(
        daemon.running.url.clone(),
        Some("not-a-real-token".to_string()),
    );
    let err = bad.status().await.unwrap_err();
    assert!(matches!(err, ClientError::Api { status: 401, .. }));
}

#[tokio::test]
async fn agent_token_cannot_create_tokens() {
    let (daemon, _tmp) = support::start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(bridle_api::types::Workdir::Repo),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    let token_path = daemon
        .workspace
        .join(".bridle/agents")
        .join(&agent.id)
        .join("token");
    let token = std::fs::read_to_string(token_path).expect("agent token file");
    let agent_client = Client::new(daemon.running.url.clone(), Some(token.trim().to_string()));

    // An agent token authenticates fine for ordinary endpoints...
    let status = agent_client.status().await.expect("status as agent");
    assert_eq!(status.principal, format!("agent:{}", agent.name));

    // ...but not for the human-only ones.
    let err = agent_client
        .create_token(&bridle_api::types::TokenCreateRequest {
            name: "x".to_string(),
        })
        .await
        .unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );

    let err = agent_client.shutdown().await.unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
