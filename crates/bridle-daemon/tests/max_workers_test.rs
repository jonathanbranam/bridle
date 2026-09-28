//! `max_workers` at spawn and its live override (docs/design/usage-and-budget.md,
//! Max-workers override).

mod support;

use bridle_api::types::{BudgetOverrideRequest, MaxWorkersRequest, SpawnRequest, Workdir};
use bridle_api::{Client, ClientError};

async fn spawn(client: &Client, role: &str, name: &str) -> Result<(), ClientError> {
    client
        .spawn(&SpawnRequest {
            components: Vec::new(),
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

#[tokio::test]
async fn schedule_less_preset_caps_workers_while_overridden_and_reverts() {
    let config = "[budget]\nmax_workers = 3\n\n[[budget.schedule]]\nname = \"low\"\n\
                  hold_at = 50\nwind_down_at = 60\nstop_at = 70\nmax_workers = 1\n";
    let (daemon, _tmp) = support::start_daemon_with_config(None, Some(config)).await;
    let c = &daemon.client;
    let ov = |period: &str| BudgetOverrideRequest {
        period: Some(period.to_string()),
        until: None,
    };

    // The preset has no span and the clock never picks it.
    let b = c.budget().await.expect("budget");
    assert_eq!(b.five_hour.source, "default");
    assert_eq!(b.schedule[0].span, None);
    assert_eq!(b.schedule[0].max_workers, Some(1));

    spawn(c, "worker", "w1").await.expect("w1");
    let b = c.budget_override(&ov("low")).await.expect("override");
    assert_eq!(b.max_workers_override, Some(1));
    assert_eq!(b.five_hour.max_workers, Some(1));
    assert!(is_conflict(spawn(c, "worker", "w2").await));

    // Clearing the override reverts the cap to the configured 3.
    let b = c.budget_override_clear().await.expect("clear");
    assert_eq!(b.max_workers_override, None);
    spawn(c, "worker", "w2").await.expect("w2 after clear");

    // A cap the human set by hand survives the preset's override ending.
    c.budget_override(&ov("low")).await.expect("override again");
    c.budget_max_workers(&MaxWorkersRequest {
        max_workers: Some(2),
    })
    .await
    .expect("hand set");
    let b = c.budget_override_clear().await.expect("clear");
    assert_eq!(b.max_workers_override, Some(2));
}
