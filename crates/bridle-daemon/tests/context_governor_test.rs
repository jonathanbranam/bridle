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
        usage_http: bridle_daemon::usage_http::UsageHttp {
            url: "http://127.0.0.1:9/".to_string(),
            token: Some("test".to_string()),
        },
        task_flush_interval: Duration::from_secs(3600),
        claim_lease_check_interval: Duration::from_secs(3600),
        port_check_interval: Duration::from_secs(3600),
        load_watch: false,
        upgrade: Default::default(),
        ci_tick_interval: std::time::Duration::from_secs(3600),
        drain_wake_after: std::time::Duration::from_secs(600),
        settle_wake_interval: std::time::Duration::from_secs(3600),
        doc_watch_interval: std::time::Duration::from_secs(3600),
        queue_nudge_debounce: std::time::Duration::from_secs(3600),
        open_watch_debounce: std::time::Duration::from_secs(3600),
        host: None,
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
            ticket: None,
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
            ..Default::default()
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
            ..Default::default()
        })
        .await
        .expect("events");
    assert_eq!(events.len(), 1, "renew must fire exactly once per crossing");
}

/// br-9y7v: a renewed agent's new claude session counts its cost from 0, so the first turn
/// after a renew must not record a negative delta against the agent's lifetime total.
#[tokio::test]
async fn renew_keeps_turn_costs_non_negative_and_the_total_summing() {
    let config = format!("[context.wind_down_at]\ndefault = {THRESHOLD}\n");
    let (daemon, _tmp) = start_daemon_with_config(Some(fast_overrides()), Some(&config)).await;
    let c = &daemon.client;

    let agent = c
        .spawn(&SpawnRequest {
            ticket: None,
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 1".to_string()),
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: vec![("FAKE_CLAUDE_COST_STEP".to_string(), "1".to_string())],
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    write_high_context_usage(&agent.cwd);
    wait_for_event(c, "agent.renewed", Some(&agent.id), |_| true).await;

    // The first turn (1.0), the handoff turn (1.0) and the renewed session's continuation
    // turn (1.0, its counter restarting at 0): 3.0 in all. The old baseline made the last
    // one -1.0.
    let mut total = 0.0;
    for _ in 0..100 {
        total = c
            .get_agent(&agent.id)
            .await
            .expect("get agent")
            .cost_usd_total;
        if total >= 3.0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        total >= 3.0,
        "lifetime cost {total} should reach 3.0 across the renew"
    );
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
                ticket: None,
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
                ..Default::default()
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

/// br-2kam: a renewal shows as one session ending (renewal, at N tokens) and the next starting
/// (renewal), each carrying the renewal limit in force.
#[allow(clippy::type_complexity)]
#[tokio::test]
async fn renewal_is_recorded_as_one_session_ending_and_the_next_starting() {
    let config = format!("[context.wind_down_at]\ndefault = {THRESHOLD}\n");
    let (daemon, _tmp) = start_daemon_with_config(Some(fast_overrides()), Some(&config)).await;
    let c = &daemon.client;
    let agent = c
        .spawn(&SpawnRequest {
            ticket: None,
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
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
    wait_for_event(c, "agent.renewed", Some(&agent.id), |_| true).await;
    let renewed = c.get_agent(&agent.id).await.expect("get renewed agent");

    let db = rusqlite::Connection::open(daemon.workspace.join(".bridle/bridle.db")).expect("db");
    let rows: Vec<(String, String, Option<String>, Option<i64>, f64)> = db
        .prepare(
            "SELECT session_id, start_reason, end_reason, end_tokens, renewal_limit
             FROM agent_sessions WHERE agent_id = ?1 ORDER BY id",
        )
        .expect("prepare")
        .query_map([&agent.id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0].0, agent.session_id);
    assert_eq!(rows[0].1, "spawn");
    assert_eq!(rows[0].2.as_deref(), Some("renewal"));
    // The context size at the end of the last turn (the handoff turn re-reads it), not the crossing.
    assert!(rows[0].3.is_some_and(|t| t > 0), "{rows:?}");
    assert_eq!(rows[0].4, THRESHOLD as f64);
    assert_eq!(rows[1].0, renewed.session_id);
    assert_eq!(rows[1].1, "renewal");
    assert_eq!(rows[1].4, THRESHOLD as f64);
}
