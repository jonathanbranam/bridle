//! Integration tests for the budget governor: state transitions from a
//! scripted `get_usage` probe, the hold it enforces on spawn/resume and
//! idle-agent messages, and the wind-down/resume/human-hold/escape-hatch
//! behaviour (docs/design/usage-and-budget.md, The budget governor).

mod support;

use bridle_api::types::{
    AgentState, BudgetHoldRequest, BudgetOverrideRequest, GovernorState, MessageQuery,
    ResumeRequest, SendRequest, SpawnRequest, When,
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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

/// An idle agent's held message (docs/design/agent-host/messages.md,
/// Delivery) has no turn ending of its own to trigger delivery once the
/// governor recovers; the governor's drop back to `normal` must deliver it
/// itself instead of leaving it held forever.
#[tokio::test]
async fn held_message_for_idle_agent_is_delivered_when_governor_returns_to_normal() {
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn while normal");
    let agent = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

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
                body: "hello while holding".to_string(),
                kind: Default::default(),
                when: When::Now,
                reply_to: None,
            },
        )
        .await
        .expect("message accepted (held, not refused)");
    assert_eq!(msg.state, bridle_api::types::MessageState::Held);

    // Back below every threshold: the agent is still idle (nothing else
    // poked it), so only the governor's own recovery path can deliver it.
    script_usage(std::path::Path::new(&agent.cwd), 10.0, 10.0);
    wait_for("normal after recovery", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;

    wait_for("held message delivered", || {
        let daemon = &daemon;
        let msg_id = msg.id.clone();
        let agent_id = agent.id.clone();
        async move {
            let messages = daemon
                .client
                .list_messages(&MessageQuery {
                    to: Some(agent_id),
                    ..Default::default()
                })
                .await
                .ok()?;
            let m = messages.iter().find(|m| m.id == msg_id)?;
            (m.state != bridle_api::types::MessageState::Held).then_some(())
        }
    })
    .await;
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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
    // The agent's state flips to running before the governor's resume note
    // is written (`maybe_resume` in governor.rs: `manager.resume` then a
    // separate `manager.send`), so poll for the message instead of
    // checking once right after the state transition.
    wait_for("resume note delivered", || {
        let daemon = &daemon;
        let id = agent.id.clone();
        async move {
            has_message_containing(daemon, &id, "pause is over")
                .await
                .then_some(())
        }
    })
    .await;
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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

/// `bridle budget override`: forces a named `[[budget.schedule]]` period's
/// `five_hour` thresholds regardless of whether the schedule itself would
/// currently pick it, an explicit `--until` is stored as given, and
/// `override clear` reverts to the plain thresholds right away.
#[tokio::test]
async fn budget_override_forces_period_thresholds_and_clear_reverts() {
    // `start == end` never naturally matches (`SchedulePeriod::matches`), so
    // this period only ever applies while forced by the override — the test
    // doesn't depend on the real wall-clock time it happens to run at.
    let (daemon, _tmp) = support::start_daemon_with_config(
        None,
        Some(
            r#"
[[budget.schedule]]
name = "night"
days = "all"
start = "03:00"
end = "03:00"
hold_at = 50
wind_down_at = 60
stop_at = 70
"#,
        ),
    )
    .await;

    // 55%: below the plain default hold_at (80), but at/above `night`'s (50).
    script_usage(&daemon.repo, 55.0, 10.0);
    wait_for("normal before override", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;

    let explicit_until = chrono::Utc::now() + chrono::Duration::hours(1);
    let overridden = daemon
        .client
        .budget_override(&BudgetOverrideRequest {
            period: Some("night".to_string()),
            until: Some(explicit_until),
        })
        .await
        .expect("override");
    let ov = overridden.schedule_override.expect("override in force");
    assert_eq!(ov.period.as_deref(), Some("night"));
    assert_eq!(ov.until, Some(explicit_until));

    wait_for("holding under the forced night thresholds", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Holding).then_some(())
    })
    .await;

    let cleared = daemon.client.budget_override_clear().await.expect("clear");
    assert!(cleared.schedule_override.is_none());
    wait_for("normal again after clear", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;
}

/// With no `--until`, the override's `until` is computed on the daemon side:
/// the next instant the schedule (unforced) would change periods.
#[tokio::test]
async fn budget_override_without_until_computes_next_schedule_change() {
    let (daemon, _tmp) = support::start_daemon_with_config(
        None,
        Some(
            r#"
[[budget.schedule]]
name = "night"
days = "all"
start = "23:00"
end = "07:00"
hold_at = 70
wind_down_at = 85
stop_at = 95
"#,
        ),
    )
    .await;

    let before = chrono::Utc::now();
    let overridden = daemon
        .client
        .budget_override(&BudgetOverrideRequest {
            period: None,
            until: None,
        })
        .await
        .expect("override");
    let ov = overridden.schedule_override.expect("override in force");
    assert_eq!(ov.period, None);
    // `night` recurs daily, so the schedule always changes within 24h of
    // any instant; the computed `until` must be a real, future instant.
    let until = ov.until.expect("a computed next-change instant");
    assert!(until > before && until <= before + chrono::Duration::days(1));
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
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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

/// Writes `.fake-claude-usage` with a per-model window on top of the two
/// default-scoped ones, for [`GovernorSnapshot::for_model`] step-down tests.
fn script_usage_with_model_window(
    repo: &std::path::Path,
    five_hour_pct: f64,
    seven_day_pct: f64,
    model_window: &str,
    model_window_pct: f64,
) {
    std::fs::write(
        repo.join(".fake-claude-usage"),
        serde_json::json!({
            "rate_limits": {
                "five_hour": {"utilization": five_hour_pct},
                "seven_day": {"utilization": seven_day_pct},
                model_window: {"utilization": model_window_pct}
            },
            "limits": []
        })
        .to_string(),
    )
    .expect("write .fake-claude-usage");
}

/// docs/design/usage-and-budget.md, Model choice: with no `model` pinned,
/// the manager's `["opus", "sonnet"]` list steps down to sonnet once
/// `seven_day_opus` alone is tight, per `GovernorSnapshot::for_model`.
#[tokio::test]
async fn spawn_steps_down_to_the_next_model_when_the_first_ones_window_is_tight() {
    let (daemon, _tmp) = start_daemon(None).await;
    script_usage_with_model_window(&daemon.repo, 10.0, 10.0, "seven_day_opus", 96.0);
    wait_for(
        "seven_day_opus is paused, default windows stay normal",
        || async {
            let b = daemon.client.budget().await.ok()?;
            let opus_paused = b
                .windows
                .iter()
                .any(|w| w.window == "seven_day_opus" && w.state == GovernorState::Paused);
            (b.state == GovernorState::Normal && opus_paused).then_some(())
        },
    )
    .await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "manager".to_string(),
            name: Some("m1".to_string()),
            prompt: None,
            workdir: None,
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn steps down instead of refusing");
    assert_eq!(agent.model, "sonnet");
}

/// A task pinning a model (`model = "opus"`) bypasses the step-down
/// entirely, per usage-and-budget.md's existing escape hatch — the governor
/// still refuses the spawn if that model's own window is blocked.
#[tokio::test]
async fn explicit_model_pin_bypasses_step_down() {
    let (daemon, _tmp) = start_daemon(None).await;
    script_usage_with_model_window(&daemon.repo, 10.0, 10.0, "seven_day_opus", 96.0);
    wait_for(
        "seven_day_opus is paused, default windows stay normal",
        || async {
            let b = daemon.client.budget().await.ok()?;
            let opus_paused = b
                .windows
                .iter()
                .any(|w| w.window == "seven_day_opus" && w.state == GovernorState::Paused);
            (b.state == GovernorState::Normal && opus_paused).then_some(())
        },
    )
    .await;

    let err = daemon
        .client
        .spawn(&SpawnRequest {
            role: "manager".to_string(),
            name: Some("m2".to_string()),
            prompt: None,
            workdir: None,
            model: Some("opus".to_string()),
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect_err("pinned opus is refused on its own tight window, not stepped down");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 409, .. }
    ));
}

/// Stepping down delays a wind-down, it never replaces one: with every
/// model in the manager's list blocked, spawn falls through to the
/// existing hold-enforcement 409 rather than inventing a new failure mode.
#[tokio::test]
async fn spawn_refuses_when_every_candidate_model_is_blocked() {
    let (daemon, _tmp) = start_daemon(None).await;
    script_usage_with_model_window(&daemon.repo, 10.0, 10.0, "seven_day_opus", 96.0);
    // Both of the manager's candidates (opus, sonnet) are blocked: opus by
    // its own window, sonnet by the default-scoped hold below.
    script_usage(&daemon.repo, 82.0, 10.0);
    wait_for("holding", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Holding).then_some(())
    })
    .await;

    let err = daemon
        .client
        .spawn(&SpawnRequest {
            role: "manager".to_string(),
            name: Some("m3".to_string()),
            prompt: None,
            workdir: None,
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect_err("every candidate blocked: refuse rather than spawn");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 409, .. }
    ));
}

/// k7nr: `max_workers` limits workers only, so a manager paused last still
/// resumes when the workers have already used every slot.
#[tokio::test]
async fn resume_brings_back_a_manager_even_when_workers_fill_max_workers() {
    let (daemon, _tmp) =
        support::start_daemon_with_config(None, Some("[budget]\nmax_workers = 2\n")).await;
    script_usage(&daemon.repo, 10.0, 10.0);
    wait_for("normal before spawn", || async {
        let b = daemon.client.budget().await.ok()?;
        (b.state == GovernorState::Normal).then_some(())
    })
    .await;

    let mut ids = Vec::new();
    for (role, name) in [("worker", "w1"), ("worker", "w2"), ("manager", "m1")] {
        let a = daemon
            .client
            .spawn(&SpawnRequest {
                role: role.to_string(),
                name: Some(name.to_string()),
                prompt: None,
                workdir: Some(bridle_api::types::Workdir::Repo),
                model: None,
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                ignore_budget: false,
            })
            .await
            .expect("spawn while normal");
        wait_for_state(&daemon.client, &a.id, AgentState::Idle).await;
        ids.push(a.id);
    }

    // Idle agents stop at once on wind-down; the manager is stopped last.
    script_usage(&daemon.repo, 90.0, 10.0);
    for id in &ids {
        wait_for_state(&daemon.client, id, AgentState::Stopped).await;
    }

    script_usage(&daemon.repo, 10.0, 10.0);
    for id in &ids {
        wait_for(&format!("{id} resumed"), || async {
            let a = daemon.client.get_agent(id).await.ok()?;
            a.state.is_running().then_some(())
        })
        .await;
    }
}
