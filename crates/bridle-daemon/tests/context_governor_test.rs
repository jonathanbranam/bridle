//! htp6b: the `[context]` wind-down config governs every agent's context
//! size the way `[budget] wind_down_at` governs account usage — once
//! `context_tokens` crosses the role's threshold, bridle sends a "Context
//! handoff:" notice and renews the agent in place, exactly once per
//! crossing.

mod support;

use std::time::Duration;

use bridle_api::types::{MessageQuery, SpawnRequest, Workdir};
use bridle_daemon::Overrides;
use support::{fake_claude_path, start_daemon_with_config, wait_for_event};

fn fast_overrides() -> Overrides {
    Overrides {
        claude_program: fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        stall_check_interval: Duration::from_millis(50),
        tracker_interval: Duration::from_millis(200),
        governor_interval: Duration::from_millis(200),
        governor_poll_interval_normal: Duration::ZERO,
        governor_poll_interval_above_hold: Duration::ZERO,
        task_flush_interval: Duration::from_secs(3600),
    }
}

#[tokio::test]
async fn crossing_wind_down_at_sends_handoff_and_renews_once() {
    // An artificially low threshold: any completed turn's context_tokens
    // will cross it.
    let config = "[context.wind_down_at]\ndefault = 1\n";
    let (daemon, _tmp) = start_daemon_with_config(Some(fast_overrides()), Some(config)).await;
    let c = &daemon.client;

    let agent = c
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("hi".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    // With a threshold this low, the crossing (and its handoff-turn
    // renewal) can happen within a poll or two of the first turn ending, so
    // don't try to catch the transient "Idle right after turn 1" state —
    // it's a race by construction. Go straight to the renewal itself.
    wait_for_event(c, "agent.renewed", Some(&agent.id), |_| true).await;

    let messages = c
        .list_messages(&MessageQuery {
            to: Some(agent.id.clone()),
            from: None,
            unread: false,
            limit: None,
        })
        .await
        .expect("list messages");
    let handoffs: Vec<_> = messages
        .iter()
        .filter(|m| m.body.starts_with("Context handoff:"))
        .collect();
    assert_eq!(
        handoffs.len(),
        1,
        "expected exactly one handoff notice, got {handoffs:?}"
    );

    // Renew clears context_tokens (a fresh session has no completed turn
    // yet), so later ticks see nothing to cross and don't renew again.
    let renewed = c.get_agent(&agent.id).await.expect("get renewed agent");
    assert_eq!(renewed.context_tokens, None);
    assert_ne!(
        renewed.session_id, agent.session_id,
        "renew starts a fresh session"
    );

    tokio::time::sleep(Duration::from_millis(500)).await;
    let events = c
        .events(&bridle_api::types::EventQuery {
            since: None,
            agent: Some(agent.id.clone()),
            kind: Some("agent.renewed".to_string()),
            limit: None,
        })
        .await
        .expect("events");
    assert_eq!(events.len(), 1, "renew must fire exactly once per crossing");
}
