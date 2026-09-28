//! §2-3: spawning a worker with a prompt, and message delivery timing.

mod support;

use bridle_api::types::{
    AgentState, MessageKind, MessageQuery, MessageState, SendRequest, When, Workdir,
};
use bridle_api::types::{SpawnRequest, event_kind};
use bridle_daemon::Overrides;
use support::{
    default_overrides, fake_claude_argv_dump_wrapper, fake_claude_env_dump_wrapper, start_daemon,
    start_daemon_with_config, wait_for_agent, wait_for_event, wait_for_state,
};

/// k8dw: `extra_allowed_tools` grants a tool beyond the role's own
/// `allowed_tools` for this one spawn, without touching the role.
#[tokio::test]
async fn spawn_extra_allowed_tools_grants_a_tool_the_role_lacks() {
    let argv_dir = tempfile::tempdir().expect("argv tempdir");
    let argv_path = argv_dir.path().join("argv.json");
    let wrapper = fake_claude_argv_dump_wrapper(argv_dir.path(), &argv_path);
    let overrides = Overrides {
        claude_program: wrapper.to_string_lossy().into_owned(),
        ..default_overrides()
    };

    let (daemon, _tmp) = start_daemon_with_config(Some(overrides), None).await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: vec!["WebSearch".to_string()],
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    wait_for_event(
        &daemon.client,
        event_kind::AGENT_SPAWNED,
        Some(&agent.id),
        |_| true,
    )
    .await;

    // The daemon's own governor probes reuse this same overridden
    // `claude_program` and dump their argv too; filter on `--name w1` (only
    // this agent's own process gets `cmd.name`) so a probe can't be mistaken
    // for it.
    let argv: Vec<String> =
        support::wait_for_dump("fake-claude argv file", &argv_path, |argv: &Vec<String>| {
            argv.windows(2).any(|w| w[0] == "--name" && w[1] == "w1")
        })
        .await;

    let flag_idx = argv
        .iter()
        .position(|a| a == "--allowedTools")
        .expect("--allowedTools flag present");
    assert!(
        argv[flag_idx + 1..].iter().any(|a| a == "WebSearch"),
        "expected WebSearch among allowed tools, got {argv:?}"
    );
    // The role's own tools are still there too: this only adds.
    assert!(argv[flag_idx + 1..].iter().any(|a| a == "Read"));
}

/// 2ty9: `extra_env` sets an env var in this one spawn's process only, and
/// it doesn't carry over to the next spawn of the same role.
#[tokio::test]
async fn spawn_extra_env_reaches_only_that_one_process() {
    let env_dir = tempfile::tempdir().expect("env tempdir");
    let env_path = env_dir.path().join("env.json");
    let wrapper = fake_claude_env_dump_wrapper(env_dir.path(), &env_path);
    let overrides = Overrides {
        claude_program: wrapper.to_string_lossy().into_owned(),
        ..default_overrides()
    };

    let (daemon, _tmp) = start_daemon_with_config(Some(overrides), None).await;

    // w1 gets the secret.
    let agent1 = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: vec![("PIXELLAB_TOKEN".to_string(), "secret-1".to_string())],
            ignore_budget: false,
        })
        .await
        .expect("spawn w1");
    wait_for_event(
        &daemon.client,
        event_kind::AGENT_SPAWNED,
        Some(&agent1.id),
        |_| true,
    )
    .await;
    // Same reasoning as the argv filter above: filter on `BRIDLE_AGENT_NAME`
    // so a governor probe's own env dump (it isn't a named agent) can't be
    // mistaken for w1's.
    let env1: std::collections::HashMap<String, String> = support::wait_for_dump(
        "fake-claude env file (w1)",
        &env_path,
        |env: &std::collections::HashMap<String, String>| {
            env.get("BRIDLE_AGENT_NAME").map(String::as_str) == Some("w1")
        },
    )
    .await;
    assert_eq!(
        env1.get("PIXELLAB_TOKEN").map(String::as_str),
        Some("secret-1")
    );

    // w2, same role, no --env: doesn't inherit w1's secret.
    let agent2 = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w2".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn w2");
    wait_for_event(
        &daemon.client,
        event_kind::AGENT_SPAWNED,
        Some(&agent2.id),
        |_| true,
    )
    .await;
    let env2: std::collections::HashMap<String, String> = support::wait_for_dump(
        "fake-claude env file (w2)",
        &env_path,
        |env: &std::collections::HashMap<String, String>| {
            env.get("BRIDLE_AGENT_NAME").map(String::as_str) == Some("w2")
        },
    )
    .await;
    assert!(
        !env2.contains_key("PIXELLAB_TOKEN"),
        "w2 should not inherit w1's extra_env, got {env2:?}"
    );
}

#[tokio::test]
async fn spawn_with_prompt_runs_a_turn_and_creates_a_worktree() {
    let (daemon, _tmp) = start_daemon(None).await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("hello there".to_string()),
            workdir: None,
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    assert_eq!(agent.name, "w1");
    assert_eq!(agent.role, "worker");

    let wt = agent.worktree.clone().expect("worktree path");
    assert!(std::path::Path::new(&wt).join(".git").exists());
    assert_eq!(agent.branch.as_deref(), Some("bridle/w1"));

    wait_for_event(
        &daemon.client,
        event_kind::AGENT_SPAWNED,
        Some(&agent.id),
        |_| true,
    )
    .await;
    wait_for_event(
        &daemon.client,
        event_kind::MESSAGE_SENT,
        Some(&agent.id),
        |_| true,
    )
    .await;
    wait_for_event(
        &daemon.client,
        event_kind::MESSAGE_DELIVERED,
        Some(&agent.id),
        |_| true,
    )
    .await;
    wait_for_event(
        &daemon.client,
        event_kind::TURN_STARTED,
        Some(&agent.id),
        |_| true,
    )
    .await;
    let ended = wait_for_event(
        &daemon.client,
        event_kind::TURN_ENDED,
        Some(&agent.id),
        |_| true,
    )
    .await;
    assert_eq!(ended.data["n"], 1);

    let done = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    assert_eq!(done.turns, 1);
    assert!(done.cost_usd_total > 0.0, "expected nonzero cost");
}

/// Fix for docs/questions/open/v1-follow-ups-from-the-build-9c6e.md's spawn
/// readiness gap: with a first message, `spawn` waits for that turn's
/// `system/init` before answering, so the response (and the store, and the
/// event log) already reflect the turn having started — no extra polling
/// needed, unlike the assertions above this test that use `wait_for_*`.
#[tokio::test]
async fn spawn_with_prompt_waits_for_the_turn_to_start_before_returning() {
    let (daemon, _tmp) = start_daemon(None).await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("hello there".to_string()),
            workdir: None,
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    assert_eq!(agent.state, AgentState::Working);

    let events = daemon
        .client
        .events(&bridle_api::types::EventQuery {
            since: None,
            agent: Some(agent.id.clone()),
            kind: Some(event_kind::TURN_STARTED.to_string()),
            limit: None,
        })
        .await
        .expect("list events");
    assert_eq!(events.len(), 1, "turn.started should already be recorded");
}

/// The other side of the same fix: a spawn with no first message starts no
/// turn, so there's no `system/init` to wait for, and `spawn` returns at
/// once instead of waiting out `SPAWN_READY_TIMEOUT`.
#[tokio::test]
async fn spawn_without_a_prompt_returns_promptly_and_stays_idle() {
    let (daemon, _tmp) = start_daemon(None).await;

    let started = std::time::Instant::now();
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
        .expect("spawn");

    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "an idle spawn should not wait for a turn that never starts"
    );
    assert_eq!(agent.state, AgentState::Idle);
}

/// `spawn` waits (best-effort, bounded by `SPAWN_READY_TIMEOUT`) for either
/// the first turn's `system/init` or the process's exit, whichever comes
/// first, per docs/design/agent-host/agents.md "Spawn waits for readiness".
/// That's a wait, not a promptness guarantee, so this only checks that
/// `spawn` succeeds and the agent eventually lands in `Crashed`.
#[tokio::test]
async fn spawn_with_a_crashing_first_message_reaches_crashed_state() {
    let (daemon, _tmp) = start_daemon(None).await;

    daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("CRASH".to_string()),
            workdir: None,
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    wait_for_state(&daemon.client, "w1", AgentState::Crashed).await;
}

#[tokio::test]
async fn message_now_mid_turn_folds_into_the_running_turn() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 3".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;

    let msg = daemon
        .client
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "banana".to_string(),
                kind: MessageKind::Note,
                when: When::Now,
                reply_to: None,
            },
        )
        .await
        .expect("send mid-turn message");

    let delivered = wait_for(&daemon.client, &msg.id).await;
    assert_eq!(delivered.state, MessageState::Delivered);

    let final_agent = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    // The fold-in means the sleep turn is the only turn; the CLAUDE.md docs
    // call for "at most one extra turn", so allow up to 2.
    assert!(final_agent.turns <= 2, "turns = {}", final_agent.turns);
}

/// kc4v: `context_tokens` must come from the turn-end `get_context_usage`
/// probe, not from summing `result.usage` across the turn's tool round
/// trips (docs/spikes/01-stream-json-findings.md row 8 — that sum is
/// per-API-call, not the context size).
#[tokio::test]
async fn context_tokens_comes_from_get_context_usage_not_turn_usage_sum() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 2".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;

    // fake-claude's fixed per-call `usage` sums to 110 regardless of how
    // many tool round trips the turn makes; 55_000 is unreachable from that
    // sum, so seeing it proves context_tokens came from get_context_usage.
    std::fs::write(
        std::path::Path::new(&agent.cwd).join(".fake-claude-context-usage"),
        serde_json::json!({"totalTokens": 55_000}).to_string(),
    )
    .expect("write .fake-claude-context-usage");

    let final_agent = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    assert_eq!(final_agent.context_tokens, Some(55_000));
}

#[tokio::test]
async fn message_idle_is_held_until_the_turn_ends_then_starts_its_own_turn() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 2".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;

    let msg = daemon
        .client
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "later please".to_string(),
                kind: MessageKind::Note,
                when: When::Idle,
                reply_to: None,
            },
        )
        .await
        .expect("send held message");
    assert_eq!(msg.state, MessageState::Held);

    // The first turn ends...
    wait_for_agent(&daemon.client, &agent.id, |a| a.turns >= 1).await;
    // ...and the held message starts its own turn.
    let delivered = wait_for(&daemon.client, &msg.id).await;
    assert_eq!(delivered.state, MessageState::Delivered);
    let done = wait_for_agent(&daemon.client, &agent.id, |a| {
        a.turns >= 2 && a.state == AgentState::Idle
    })
    .await;
    assert_eq!(done.turns, 2);
}

#[tokio::test]
async fn human_inbox_receives_agent_messages_and_mark_read_works() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    let token_path = daemon
        .workspace
        .join(".bridle/agents")
        .join(&agent.id)
        .join("token");
    let token = std::fs::read_to_string(token_path)
        .expect("agent token")
        .trim()
        .to_string();
    let agent_client = bridle_api::Client::new(daemon.running.url.clone(), Some(token));

    let sent = agent_client
        .send(&SendRequest {
            to: Some("human".to_string()),
            body: "task complete".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
        })
        .await
        .expect("agent sends to human");
    let sent = sent.into_iter().next().expect("one message sent");

    let unread = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            unread: true,
            ..Default::default()
        })
        .await
        .expect("list unread");
    assert!(unread.iter().any(|m| m.id == sent.id));
    let found = unread
        .iter()
        .find(|m| m.id == sent.id)
        .expect("message present");
    assert_eq!(found.from, format!("agent:{}", agent.name));

    let marked = daemon.client.mark_read(&sent.id).await.expect("mark read");
    assert_eq!(marked.state, MessageState::Read);

    let unread_after = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            unread: true,
            ..Default::default()
        })
        .await
        .expect("list unread after read");
    assert!(!unread_after.iter().any(|m| m.id == sent.id));
}

#[tokio::test]
async fn send_to_role_delivers_to_every_live_agent_with_that_role() {
    let (daemon, _tmp) = start_daemon(None).await;

    async fn spawn_idle(
        daemon: &support::TestDaemon,
        role: &str,
        name: &str,
    ) -> bridle_api::types::Agent {
        let agent = daemon
            .client
            .spawn(&SpawnRequest {
                role: role.to_string(),
                name: Some(name.to_string()),
                prompt: None,
                workdir: Some(Workdir::Repo),
                model: None,
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                ignore_budget: false,
            })
            .await
            .expect("spawn");
        wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await
    }

    let w1 = spawn_idle(&daemon, "worker", "w1").await;
    let w2 = spawn_idle(&daemon, "worker", "w2").await;
    let manager = spawn_idle(&daemon, "manager", "m1").await;

    let sent = daemon
        .client
        .send(&SendRequest {
            to: Some("role:worker".to_string()),
            body: "standup".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
        })
        .await
        .expect("send to role");
    assert_eq!(sent.len(), 2);
    let recipients: std::collections::HashSet<_> = sent.iter().map(|m| m.to.clone()).collect();
    assert_eq!(
        recipients,
        std::collections::HashSet::from([w1.id.clone(), w2.id.clone()])
    );

    for agent_id in [&w1.id, &w2.id] {
        let inbox = daemon
            .client
            .list_messages(&MessageQuery {
                to: Some(agent_id.clone()),
                ..Default::default()
            })
            .await
            .expect("list messages");
        assert!(inbox.iter().any(|m| m.body == "standup"));
    }

    let manager_inbox = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some(manager.id.clone()),
            ..Default::default()
        })
        .await
        .expect("list messages");
    assert!(!manager_inbox.iter().any(|m| m.body == "standup"));
}

#[tokio::test]
async fn send_to_role_with_no_live_agents_errors() {
    let (daemon, _tmp) = start_daemon(None).await;
    let err = daemon
        .client
        .send(&SendRequest {
            to: Some("role:nobody-has-this".to_string()),
            body: "hello".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
        })
        .await
        .expect_err("no live agents with role");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 404, .. }
    ));
}

async fn wait_for(client: &bridle_api::Client, id: &str) -> bridle_api::types::Message {
    support::wait_for(&format!("message {id} delivered"), || async {
        let m = client.list_messages(&MessageQuery::default()).await.ok()?;
        m.into_iter()
            .find(|m| m.id == id && m.state == MessageState::Delivered)
    })
    .await
}
