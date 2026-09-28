//! `max_workers` at spawn and its live override (docs/design/usage-and-budget.md,
//! Max-workers override).

mod support;

use bridle_api::types::{MaxWorkersRequest, SpawnRequest, Workdir};
use bridle_api::{Client, ClientError};

async fn spawn(client: &Client, role: &str, name: &str) -> Result<(), ClientError> {
    client
        .spawn(&SpawnRequest {
            role: role.to_string(),
            name: Some(name.to_string()),
            prompt: None,
            // Repo workdir: no worktree, so these don't depend on git branches.
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .map(|_| ())
}

fn is_conflict(r: Result<(), ClientError>) -> bool {
    matches!(r, Err(ClientError::Api { status: 409, .. }))
}

#[tokio::test]
async fn spawn_is_refused_at_the_cap_and_only_workers_count() {
    let (daemon, _tmp) =
        support::start_daemon_with_config(None, Some("[budget]\nmax_workers = 1\n")).await;
    let c = &daemon.client;

    // A manager and an orchestrator don't count against the cap.
    spawn(c, "manager", "m1").await.expect("manager spawns");
    spawn(c, "orchestrator", "o1")
        .await
        .expect("orchestrator spawns");
    spawn(c, "worker", "w1").await.expect("first worker spawns");
    assert!(is_conflict(spawn(c, "worker", "w2").await));
    // Only workers are limited: a manager still spawns at the cap.
    spawn(c, "manager", "m2")
        .await
        .expect("manager spawns at cap");
}

#[tokio::test]
async fn live_override_changes_the_cap_without_a_restart_and_clears() {
    let (daemon, _tmp) =
        support::start_daemon_with_config(None, Some("[budget]\nmax_workers = 1\n")).await;
    let c = &daemon.client;
    let set = |n| MaxWorkersRequest { max_workers: n };

    spawn(c, "worker", "w1").await.expect("w1");
    assert!(is_conflict(spawn(c, "worker", "w2").await));

    let b = c.budget_max_workers(&set(Some(2))).await.expect("raise");
    assert_eq!(b.max_workers_override, Some(2));
    assert_eq!(b.thresholds.max_workers, 1, "config value is unchanged");
    spawn(c, "worker", "w2")
        .await
        .expect("w2 under the raised cap");

    // Lowering never stops running workers; it only blocks new spawns.
    c.budget_max_workers(&set(Some(1))).await.expect("lower");
    assert!(is_conflict(spawn(c, "worker", "w3").await));
    let running = c.list_agents().await.expect("list").len();
    assert_eq!(running, 2);

    let b = c.budget_max_workers(&set(None)).await.expect("clear");
    assert_eq!(b.max_workers_override, None);
    assert!(
        is_conflict(spawn(c, "worker", "w3").await),
        "back to config 1"
    );
}
