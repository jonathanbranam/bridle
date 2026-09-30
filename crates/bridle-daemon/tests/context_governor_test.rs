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
        bridle_home: None,
        stall_check_interval: Duration::from_millis(50),
        tracker_interval: Duration::from_millis(200),
        governor_interval: Duration::from_millis(200),
        governor_poll_interval_normal: Duration::ZERO,
        governor_poll_interval_above_hold: Duration::ZERO,
        task_flush_interval: Duration::from_secs(3600),
        claim_lease_check_interval: Duration::from_secs(3600),
        port_check_interval: Duration::from_secs(3600),
        upgrade: Default::default(),
    }
}

/// A threshold above fake-claude's fixed `get_context_usage` default (999,
/// see fake-claude.py), so a turn only crosses it when a test deliberately
/// overrides `.fake-claude-context-usage` — including the synthetic
/// continuation-note turn `renew` now sends the moment it swaps a process in
/// (br-ab66): without this margin, that turn's own default-999 reading would
/// re-cross a threshold as low as `1` and renew forever. The override file is
/// consumed on read (fake-claude.py), so writing it once is enough: the
/// deliberately crossing turn reads it, every later turn (including the
/// renewed process's own continuation-note turn) falls back to the
/// under-threshold default.
const THRESHOLD: u64 = 2_000;
const CROSSING_TOKENS: u64 = 5_000;

fn write_high_context_usage(cwd: &str) {
    std::fs::write(
        std::path::Path::new(cwd).join(".fake-claude-context-usage"),
        serde_json::json!({"totalTokens": CROSSING_TOKENS}).to_string(),
    )
    .expect("write .fake-claude-context-usage");
}

#[tokio::test]
async fn crossing_wind_down_at_sends_handoff_and_renews_once() {
    let config = format!("[context.wind_down_at]\ndefault = {THRESHOLD}\n");
    let (daemon, _tmp) = start_daemon_with_config(Some(fast_overrides()), Some(&config)).await;
    let c = &daemon.client;

    let agent = c
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            // SLEEP gives the test a window, after `spawn` returns at
            // `system/init`, to drop the override below before this first
            // turn's own `get_context_usage` probe at turn end.
            prompt: Some("SLEEP 1".to_string()),
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    write_high_context_usage(&agent.cwd);

    // With a threshold this low relative to the deliberate crossing, the
    // crossing (and its handoff-turn renewal) can happen within a poll or
    // two of the first turn ending, so don't try to catch the transient
    // "Idle right after turn 1" state — it's a race by construction. Go
    // straight to the renewal itself.
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

    let renewed = c.get_agent(&agent.id).await.expect("get renewed agent");
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

/// Regression coverage for two races the single-shot test above can't catch
/// reliably (both found by manually stress-running that test tens of times,
/// not by a single run): (1) `expire_context_renews`'s sweep and the
/// turn-end hook both spawning a renew for the same crossing, and (2) the
/// crossing turn itself racing `tick_context_check` for its own
/// `context_renew_pending` flag and renewing immediately (on `stop_grace`)
/// instead of leaving a real handoff turn to run first. Many agents
/// crossing at once, repeated, gives `tick_context_check` many chances to
/// interleave with each agent's own turn-end in the ways that produced
/// those races.
#[tokio::test]
async fn many_concurrent_crossings_each_renew_exactly_once() {
    let config =
        format!("[context.wind_down_at]\ndefault = {THRESHOLD}\n[budget]\nmax_workers = 64\n");
    let (daemon, _tmp) = start_daemon_with_config(Some(fast_overrides()), Some(&config)).await;
    let c = &daemon.client;

    const N: usize = 30;
    let mut agents = Vec::with_capacity(N);
    for i in 0..N {
        let agent = c
            .spawn(&SpawnRequest {
                components: Vec::new(),
                role: "worker".to_string(),
                name: Some(format!("w{i}")),
                prompt: Some("SLEEP 1".to_string()),
                // Each agent needs its own `.fake-claude-context-usage`
                // (below), so each gets its own worktree/cwd rather than
                // sharing the plain repo checkout.
                workdir: Some(Workdir::Worktree { base: None }),
                model: None,
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                ignore_budget: false,
            })
            .await
            .expect("spawn");
        write_high_context_usage(&agent.cwd);
        agents.push(agent);
    }

    for agent in &agents {
        wait_for_event(c, "agent.renewed", Some(&agent.id), |_| true).await;
    }

    // A little slack past each agent's first renewal: if the crossing turn
    // raced its own notice and renewed on `stop_grace`, or two paths both
    // renewed the same crossing, a second `agent.renewed` would land here.
    tokio::time::sleep(Duration::from_millis(500)).await;

    for agent in &agents {
        let events = c
            .events(&bridle_api::types::EventQuery {
                since: None,
                agent: Some(agent.id.clone()),
                kind: Some("agent.renewed".to_string()),
                limit: None,
            })
            .await
            .expect("events");
        assert_eq!(
            events.len(),
            1,
            "agent {} must renew exactly once, got {events:?}",
            agent.id
        );

        let renewed = c.get_agent(&agent.id).await.expect("get renewed agent");
        assert_ne!(
            renewed.session_id, agent.session_id,
            "agent {} should have a fresh session after renew",
            agent.id
        );
    }
}
