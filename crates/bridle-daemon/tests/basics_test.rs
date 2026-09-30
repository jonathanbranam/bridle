//! §1: health, status, auth. See docs/design/agent-host/principals.md, api.md.

mod support;

use bridle_api::types::SpawnRequest;
use bridle_api::{Client, ClientError};

#[tokio::test]
async fn health_needs_no_auth_and_status_needs_a_token() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let health = daemon.client.health().await.expect("health");
    assert!(health.ok);
    assert_eq!(health.agent_count, 0);

    let anon = Client::new(daemon.running.url.clone(), None);
    assert!(anon.health().await.is_ok(), "health should need no auth");

    let status = daemon.client.status().await.expect("status");
    assert_eq!(status.principal, "human");
    assert!(!status.daemon.project.is_empty());
}

/// docs/tickets/open/read-only-access-without-a-token-9c63.md: a GET with
/// no bearer token reads as a synthetic `local` principal rather than
/// 401ing, since the daemon only listens on 127.0.0.1. A GET with a valid
/// token still authenticates normally and keeps real attribution.
#[tokio::test]
async fn get_routes_work_without_a_token_and_keep_real_attribution_with_one() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let anon = Client::new(daemon.running.url.clone(), None);
    let status = anon.status().await.expect("status without a token");
    assert_eq!(status.principal, "local");

    let agents = anon
        .list_agents()
        .await
        .expect("list_agents without a token");
    assert!(agents.is_empty());

    let status = daemon.client.status().await.expect("status with a token");
    assert_eq!(status.principal, "human");
}

/// Same ticket: writes still need a token, even from localhost.
#[tokio::test]
async fn write_routes_still_401_without_a_token() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let anon = Client::new(daemon.running.url.clone(), None);
    let err = anon
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(bridle_api::types::Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .unwrap_err();
    match err {
        ClientError::Api { status, code, .. } => {
            assert_eq!(status, 401);
            assert_eq!(code, "unauthorized");
        }
        other => panic!("expected 401, got {other:?}"),
    }
}

#[tokio::test]
async fn health_counts_non_terminal_agents() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(bridle_api::types::Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    let health = daemon.client.health().await.expect("health");
    assert_eq!(health.agent_count, 1);

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn invalid_token_is_401_even_on_a_get_route() {
    let (daemon, _tmp) = support::start_daemon(None).await;

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
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(bridle_api::types::Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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
            machine: None,
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

    let err = agent_client.list_tokens().await.unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );

    let err = agent_client.revoke_token("orchestrator").await.unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );

    // `bridle budget override`/`override clear`, like `hold`/`release`, are
    // human-only.
    let err = agent_client
        .budget_override(&bridle_api::types::BudgetOverrideRequest {
            period: None,
            until: None,
        })
        .await
        .unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );

    let err = agent_client.budget_override_clear().await.unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 403, .. }),
        "{err:?}"
    );

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}

#[tokio::test]
async fn token_list_and_revoke() {
    let (daemon, _tmp) = support::start_daemon(None).await;

    let created = daemon
        .client
        .create_token(&bridle_api::types::TokenCreateRequest {
            name: "orchestrator".to_string(),
            machine: None,
        })
        .await
        .expect("create token");
    assert_eq!(created.principal, "external:orchestrator");

    let tokens = daemon.client.list_tokens().await.expect("list tokens");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].name, "orchestrator");
    assert!(!tokens[0].revoked);

    let external = Client::new(daemon.running.url.clone(), Some(created.token.clone()));
    assert!(external.status().await.is_ok(), "token works before revoke");

    daemon
        .client
        .revoke_token("orchestrator")
        .await
        .expect("revoke token");

    let tokens = daemon
        .client
        .list_tokens()
        .await
        .expect("list after revoke");
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].revoked);

    let err = external.status().await.unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 401, .. }),
        "revoked token should no longer authenticate: {err:?}"
    );

    let err = daemon
        .client
        .revoke_token("no-such-name")
        .await
        .unwrap_err();
    assert!(
        matches!(err, ClientError::Api { status: 404, .. }),
        "{err:?}"
    );

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
