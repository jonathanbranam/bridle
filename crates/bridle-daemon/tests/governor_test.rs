//! Integration tests for the budget governor: state transitions from a
//! scripted `get_usage` probe, and the hold it enforces on spawn/resume and
//! idle-agent messages (docs/design/usage-and-budget.md, The budget
//! governor).

mod support;

use bridle_api::types::{GovernorState, SendRequest, SpawnRequest, When};
use support::{start_daemon, wait_for};

/// Writes `.fake-claude-usage` in the repo (the governor probe's cwd) so
/// the next poll reports this utilization for both default windows.
fn script_usage(repo: &std::path::Path, five_hour_pct: f64, seven_day_pct: f64) {
    std::fs::write(
        repo.join(".fake-claude-usage"),
        serde_json::json!({
            "rate_limits": {
                "five_hour": {"utilization": five_hour_pct},
                "seven_day": {"utilization": seven_day_pct}
            },
            "limits": []
        })
        .to_string(),
    )
    .expect("write .fake-claude-usage");
}

#[tokio::test]
async fn governor_holds_then_pauses_as_usage_climbs() {
    let (daemon, _tmp) = start_daemon(None).await;

    script_usage(&daemon.repo, 10.0, 10.0);
    wait_for("normal", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;

    // hold_at default is 80: no spawns.
    script_usage(&daemon.repo, 82.0, 10.0);
    wait_for("holding", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Holding).then_some(())
    })
    .await;
    let err = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: None,
            model: None,
        })
        .await
        .expect_err("spawn refused while holding");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 409, .. }
    ));

    // stop_at default is 95: paused.
    script_usage(&daemon.repo, 96.0, 10.0);
    wait_for("paused", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Paused).then_some(())
    })
    .await;
}

#[tokio::test]
async fn idle_message_is_held_not_written_while_governor_is_holding() {
    let (daemon, _tmp) = start_daemon(None).await;
    script_usage(&daemon.repo, 10.0, 10.0);
    wait_for("normal before spawn", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: None,
            model: None,
        })
        .await
        .expect("spawn while normal");
    let agent = support::wait_for_state(
        &daemon.client,
        &agent.id,
        bridle_api::types::AgentState::Idle,
    )
    .await;

    // Once an agent is running, the governor's `get_usage` probe goes to its
    // handle instead of a dedicated probe process (governor.rs's
    // `any_running_handle`), and fake-claude reads `.fake-claude-usage`
    // from its own cwd — the agent's worktree, not `daemon.repo`.
    script_usage(std::path::Path::new(&agent.cwd), 82.0, 10.0);
    wait_for("holding", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Holding).then_some(())
    })
    .await;

    let msg = daemon
        .client
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "hello".to_string(),
                kind: Default::default(),
                when: When::Now,
                reply_to: None,
            },
        )
        .await
        .expect("message accepted (held, not refused)");
    assert_eq!(msg.state, bridle_api::types::MessageState::Held);
}
