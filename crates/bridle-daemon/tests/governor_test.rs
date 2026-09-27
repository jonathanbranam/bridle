//! Integration tests for the budget governor: state transitions from a
//! scripted `get_usage` probe, the hold it enforces on spawn/resume and
//! idle-agent messages, and the wind-down/resume/human-hold/escape-hatch
//! behaviour (docs/design/usage-and-budget.md, The budget governor).

mod support;

use bridle_api::types::{
    AgentState, BudgetHoldRequest, GovernorState, MessageQuery, ResumeRequest, SendRequest,
    SpawnRequest, When,
};
use support::{start_daemon, wait_for, wait_for_state};

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
            ignore_budget: false,
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
            ignore_budget: false,
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

async fn has_message_containing(
    daemon: &support::TestDaemon,
    agent_id: &str,
    needle: &str,
) -> bool {
    daemon
        .client
        .list_messages(&MessageQuery {
            to: Some(agent_id.to_string()),
            ..Default::default()
        })
        .await
        .unwrap_or_default()
        .iter()
        .any(|m| m.body.contains(needle))
}

/// The wind-down (usage-and-budget.md, The wind-down): an idle agent is
/// stopped at once, with no notice, and the governor auto-resumes it once
/// usage drops back below `resume_below`.
#[tokio::test]
async fn idle_agent_is_stopped_at_once_on_wind_down_then_resumed() {
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
            ignore_budget: false,
        })
        .await
        .expect("spawn while normal");
    let agent = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    // wind_down_at default is 90: an idle agent is stopped at once.
    script_usage(std::path::Path::new(&agent.cwd), 90.0, 10.0);
    let stopped = wait_for_state(&daemon.client, &agent.id, AgentState::Stopped).await;
    assert_eq!(
        stopped.exit.as_ref().map(|e| e.reason.as_str()),
        Some("budget_paused")
    );
    assert!(
        !has_message_containing(&daemon, &agent.id, "Usage pause").await,
        "an idle agent gets no notice, just stopped"
    );

    // resume_below default is 70: dropping back under it resumes the agent
    // with the same session, and a short note since it has no pending mail.
    // The agent isn't running anymore, so the probe goes to a dedicated
    // process in the daemon's repo, not the (now-stopped) agent's worktree.
    script_usage(&daemon.repo, 10.0, 10.0);
    let resumed = wait_for(&format!("{} resumed", agent.id), || async {
        let a = daemon.client.get_agent(&agent.id).await.ok()?;
        a.state.is_running().then_some(a)
    })
    .await;
    assert_eq!(resumed.session_id, agent.session_id);
    assert!(has_message_containing(&daemon, &agent.id, "pause is over").await);
}

/// A working agent gets the usage-pause notice, then is stopped with
/// `budget_paused` once its turn ends on its own.
#[tokio::test]
async fn working_agent_is_notified_then_stopped_when_its_turn_ends() {
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
            prompt: Some("SLEEP 3".to_string()),
            workdir: Some(bridle_api::types::Workdir::Repo),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn while normal");
    let agent = wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;

    script_usage(std::path::Path::new(&agent.cwd), 90.0, 10.0);
    wait_for("usage pause notice sent", || {
        let daemon = &daemon;
        let id = agent.id.clone();
        async move {
            has_message_containing(daemon, &id, "Usage pause:")
                .await
                .then_some(())
        }
    })
    .await;

    // Still working: nothing interrupted it, the notice just folded into
    // the running turn.
    assert_eq!(
        daemon.client.get_agent(&agent.id).await.unwrap().state,
        AgentState::Working
    );

    let stopped = wait_for_state(&daemon.client, &agent.id, AgentState::Stopped).await;
    assert_eq!(
        stopped.exit.as_ref().map(|e| e.reason.as_str()),
        Some("budget_paused")
    );
}

/// `bridle budget hold`/`release`: `hold` forces `winding_down` (and stops
/// idle agents) even with every window quiet; `release` lets it drop back.
#[tokio::test]
async fn human_hold_forces_winding_down_and_release_lifts_it() {
    let (daemon, _tmp) = start_daemon(None).await;
    script_usage(&daemon.repo, 10.0, 10.0);
    wait_for("normal before hold", || async {
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
            ignore_budget: false,
        })
        .await
        .expect("spawn while normal");
    let agent = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    let held = daemon
        .client
        .budget_hold(&BudgetHoldRequest { until: None })
        .await
        .expect("hold");
    assert_eq!(held.state, GovernorState::WindingDown);
    assert!(held.human_hold.is_some());
    wait_for_state(&daemon.client, &agent.id, AgentState::Stopped).await;

    let released = daemon.client.budget_release().await.expect("release");
    assert!(released.human_hold.is_none());
    wait_for("normal after release", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;
}

/// `--ignore-budget` skips the governor's refusal for that one call only.
#[tokio::test]
async fn ignore_budget_bypasses_the_holding_refusal() {
    let (daemon, _tmp) = start_daemon(None).await;
    script_usage(&daemon.repo, 82.0, 10.0);
    wait_for("holding", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Holding).then_some(())
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
            ignore_budget: true,
        })
        .await
        .expect("spawn ignores the governor with --ignore-budget");

    let stopped = daemon
        .client
        .stop(&agent.id, &Default::default())
        .await
        .expect("stop");
    assert!(stopped.state.is_resumable());

    // Still holding: resume needs the same escape hatch.
    daemon
        .client
        .resume(
            &agent.id,
            &ResumeRequest {
                ignore_budget: true,
            },
        )
        .await
        .expect("resume ignores the governor with --ignore-budget");
}
