//! §2-3: spawning a worker with a prompt, and message delivery timing.

mod support;

use bridle_api::Client;
use bridle_api::types::{
    AgentState, MessageKind, MessageQuery, MessageState, SendRequest, When, Workdir,
};
use bridle_api::types::{SpawnRequest, TokenCreateRequest, event_kind};
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
            components: Vec::new(),
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
            components: Vec::new(),
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
            components: Vec::new(),
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
            components: Vec::new(),
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

/// Fix for docs/tickets/open/v1-follow-ups-from-the-build-9c6e.md's spawn
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
            components: Vec::new(),
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
/// once instead of waiting out `SPAWN_READY_TIMEOUT`. Not timed: under load
/// a wall-clock bound only measures the machine (ticket f1ky), so this checks
/// that the spawn returns and the agent is `Idle`.
#[tokio::test]
async fn spawn_without_a_prompt_returns_promptly_and_stays_idle() {
    let (daemon, _tmp) = start_daemon(None).await;

    let agent = tokio::time::timeout(
        support::HANG_GUARD_TIMEOUT,
        daemon.client.spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: None,
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        }),
    )
    .await
    .expect("spawn hung")
    .expect("spawn");

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
            components: Vec::new(),
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
            components: Vec::new(),
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
                task: None,
            },
        )
        .await
        .expect("send mid-turn message");

    let delivered = wait_for(&daemon.client, &msg.id).await;
    assert_eq!(delivered.state, MessageState::Read);

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
            components: Vec::new(),
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
            components: Vec::new(),
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
                task: None,
            },
        )
        .await
        .expect("send held message");
    assert_eq!(msg.state, MessageState::Held);

    // The first turn ends...
    wait_for_agent(&daemon.client, &agent.id, |a| a.turns >= 1).await;
    // ...and the held message starts its own turn.
    let delivered = wait_for(&daemon.client, &msg.id).await;
    assert_eq!(delivered.state, MessageState::Read);
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
            components: Vec::new(),
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
            task: None,
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

    let back = daemon
        .client
        .mark_unread(&sent.id)
        .await
        .expect("mark unread");
    assert_eq!(back.state, MessageState::Delivered);
    assert!(back.read_at.is_none());
    let unread_again = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            unread: true,
            ..Default::default()
        })
        .await
        .expect("list unread again");
    assert!(unread_again.iter().any(|m| m.id == sent.id));

    let err = daemon
        .client
        .mark_unread("m-nope")
        .await
        .expect_err("unknown id");
    assert!(format!("{err:?}").contains("not_found"), "{err:?}");
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
                components: Vec::new(),
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
            task: None,
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
            task: None,
        })
        .await
        .expect_err("no live agents with role");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 404, .. }
    ));
}

/// a7h3: an external principal (e.g. `external:orchestrator`) is an
/// addressable recipient with its own inbox, once minted with `token create`.
#[tokio::test]
async fn send_to_external_principal_lands_in_its_own_inbox() {
    let (daemon, _tmp) = start_daemon(None).await;

    let created = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "orchestrator".to_string(),
            machine: None,
        })
        .await
        .expect("create external token");
    assert_eq!(created.principal, "external:orchestrator");

    let sent = daemon
        .client
        .send(&SendRequest {
            to: Some("external:orchestrator".to_string()),
            body: "please look at this".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send to external principal")
        .into_iter()
        .next()
        .expect("one message sent");
    assert_eq!(sent.to, "external:orchestrator");

    let orchestrator = Client::new(daemon.running.url.clone(), Some(created.token));
    let inbox = orchestrator
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("orchestrator reads its own inbox");
    assert!(inbox.iter().any(|m| m.id == sent.id));

    // Never minted with `token create`: still 404s.
    let err = daemon
        .client
        .send(&SendRequest {
            to: Some("external:nobody-minted-this".to_string()),
            body: "hello".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect_err("unminted external name should 404");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 404, .. }
    ));
}

/// h5qd: a reply from a principal in `[messages] answer_for_human` (default
/// `external:orchestrator`) closes a question addressed to the human; a reply
/// from anyone else doesn't.
#[tokio::test]
async fn delegate_reply_closes_the_humans_question_but_others_do_not() {
    let (daemon, _tmp) = start_daemon(None).await;
    let mut clients = Vec::new();
    for name in ["orchestrator", "pm"] {
        let created = daemon
            .client
            .create_token(&TokenCreateRequest {
                name: name.to_string(),
                machine: None,
            })
            .await
            .expect("create external token");
        clients.push(Client::new(daemon.running.url.clone(), Some(created.token)));
    }
    let (orchestrator, pm) = (&clients[0], &clients[1]);

    let send = |c: &Client, to: &str, kind, reply_to: Option<String>, body: &str| {
        let req = SendRequest {
            to: Some(to.to_string()),
            body: body.to_string(),
            kind,
            when: When::Now,
            reply_to,
            task: None,
        };
        let c = c.clone();
        async move { c.send(&req).await.expect("send").remove(0) }
    };

    let q1 = send(pm, "human", MessageKind::Question, None, "where?").await;
    let q2 = send(pm, "human", MessageKind::Question, None, "when?").await;
    assert_eq!(
        daemon
            .client
            .status()
            .await
            .expect("status")
            .unread_human_messages,
        2
    );

    // Not on the list: q2 stays open.
    let stray = send(pm, "human", MessageKind::Note, Some(q2.id.clone()), "self").await;
    let q2_now = daemon
        .client
        .list_messages(&MessageQuery::default())
        .await
        .expect("list");
    let q2_now = q2_now.iter().find(|m| m.id == q2.id).expect("q2");
    assert!(q2_now.answered_by.is_none());
    assert_ne!(q2_now.state, MessageState::Read);
    assert_eq!(stray.answered_by, None);

    let reply = send(
        orchestrator,
        "external:pm",
        MessageKind::Note,
        Some(q1.id.clone()),
        "merge to main\nsee the rule",
    )
    .await;
    let all = daemon
        .client
        .list_messages(&MessageQuery::default())
        .await
        .expect("list");
    let q1_now = all.iter().find(|m| m.id == q1.id).expect("q1");
    assert_eq!(q1_now.state, MessageState::Read);
    assert_eq!(q1_now.answered_by.as_deref(), Some("external:orchestrator"));
    assert_eq!(q1_now.answered_reply.as_deref(), Some(reply.id.as_str()));
    assert_eq!(q1_now.answered_line.as_deref(), Some("merge to main"));
    assert_eq!(q1_now.body, "where?");

    // q1 left the count; q2 and the stray note remain.
    assert_eq!(
        daemon
            .client
            .status()
            .await
            .expect("status")
            .unread_human_messages,
        2
    );
    let unread = daemon
        .client
        .list_messages(&MessageQuery {
            to: Some("human".to_string()),
            unread: true,
            ..Default::default()
        })
        .await
        .expect("unread");
    assert!(unread.iter().all(|m| m.id != q1.id));
}

async fn wait_for(client: &bridle_api::Client, id: &str) -> bridle_api::types::Message {
    support::wait_for(&format!("message {id} delivered"), || async {
        let m = client.list_messages(&MessageQuery::default()).await.ok()?;
        m.into_iter()
            .find(|m| m.id == id && m.state == MessageState::Read)
    })
    .await
}

/// hx7t: an empty or whitespace-only body is refused, on both send routes.
#[tokio::test]
async fn send_with_an_empty_body_is_rejected() {
    let (daemon, _tmp) = start_daemon(None).await;
    for body in ["", "  \n"] {
        let err = daemon
            .client
            .send(&SendRequest {
                to: Some("human".to_string()),
                body: body.to_string(),
                ..Default::default()
            })
            .await
            .expect_err("empty body");
        assert!(err.to_string().contains("must not be empty"), "{err}");
    }
}

/// k7mw: `name@machine` is a visitor. It sends and reads its own inbox, is not the
/// daemon's `external:orchestrator`, and `@` can't be smuggled into a local name.
#[tokio::test]
async fn visitor_principal_sends_and_reads_but_is_not_the_orchestrator() {
    let (daemon, _tmp) = start_daemon(None).await;

    let err = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "orchestrator@nuc".to_string(),
            machine: None,
        })
        .await
        .unwrap_err();
    assert!(
        matches!(err, bridle_api::ClientError::Api { status: 400, .. }),
        "{err:?}"
    );

    let created = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "orchestrator".to_string(),
            machine: Some("nuc".to_string()),
        })
        .await
        .expect("create visitor token");
    assert_eq!(created.principal, "external:orchestrator@nuc");
    assert!(!created.token.is_empty());
    let visitor = Client::new(daemon.running.url.clone(), Some(created.token));

    daemon
        .client
        .send(&SendRequest {
            to: Some("external:orchestrator@nuc".to_string()),
            body: "bridle 0.4 is out".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send to visitor");
    let inbox = visitor
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("visitor reads its inbox");
    assert_eq!(inbox.len(), 1);

    visitor
        .send(&SendRequest {
            to: Some("human".to_string()),
            body: "thanks".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("visitor sends");

    let err = visitor.orchestrator_wake(None).await.unwrap_err();
    assert!(
        matches!(err, bridle_api::ClientError::Api { status: 403, .. }),
        "{err:?}"
    );
    let err = visitor.handover_done().await.unwrap_err();
    assert!(
        matches!(err, bridle_api::ClientError::Api { status: 403, .. }),
        "{err:?}"
    );
}

/// jttf B: `external:advisor/<name>` reaches a running named advisor; otherwise the shared
/// advisor inbox, marked; unread mail follows when the session ends; unknown owner 404s.
#[tokio::test]
async fn named_advisor_addressing_and_delivery_fallbacks() {
    use bridle_api::types::SessionRegister;
    let (daemon, _tmp) = start_daemon(None).await;
    let token = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "advisor".to_string(),
            machine: None,
        })
        .await
        .expect("create advisor token")
        .token;
    let url = daemon.running.url.clone();
    let research =
        Client::new(url.clone(), Some(token.clone())).with_advisor(Some("research".into()));
    let shared = Client::new(url, Some(token));
    let send = |to: &str, reply_to: Option<String>| SendRequest {
        to: Some(to.to_string()),
        body: "hello".to_string(),
        kind: MessageKind::Note,
        when: When::Now,
        reply_to,
        task: None,
    };
    let inbox = |c: &Client| {
        let c = c.clone();
        async move {
            c.list_messages(&MessageQuery {
                to: Some("me".to_string()),
                ..Default::default()
            })
            .await
            .expect("inbox")
        }
    };

    // Never existed: the shared inbox, marked, and the sender can tell by `to`.
    let m = daemon
        .client
        .send(&send("external:advisor/research", None))
        .await
        .expect("send")[0]
        .clone();
    assert_eq!(m.to, "external:advisor");
    assert_eq!(m.body, "(originally for advisor/research)\nhello");
    assert_eq!(inbox(&shared).await.len(), 1);
    assert!(inbox(&research).await.is_empty());

    // Running: its own inbox, unmarked.
    research
        .session_register(&SessionRegister {
            identity: "advisor/research".into(),
            pid: 4_000_001,
            pid_start: "t0".into(),
            pane: None,
            claude_session_id: None,
            project: None,
            machine: None,
        })
        .await
        .expect("register");
    let m = daemon
        .client
        .send(&send("external:advisor/research", None))
        .await
        .expect("send")[0]
        .clone();
    assert_eq!(
        (m.to.as_str(), m.body.as_str()),
        ("external:advisor/research", "hello")
    );
    assert_eq!(inbox(&research).await.len(), 1);
    assert_eq!(inbox(&shared).await.len(), 1);

    // Attribution, and a reply returns to the named advisor.
    let q = research.send(&send("human", None)).await.expect("send")[0].clone();
    assert_eq!(q.from, "external:advisor/research");
    let r = daemon
        .client
        .send(&send("external:advisor/research", Some(q.id.clone())))
        .await
        .expect("reply")[0]
        .clone();
    assert_eq!(r.to, "external:advisor/research");

    // Ending moves what's unread, marked; what it read stays.
    let msgs = inbox(&research).await;
    research.mark_read(&msgs[0].id).await.expect("read");
    research.session_end(4_000_001).await.expect("end");
    let shared_inbox = inbox(&shared).await;
    assert_eq!(shared_inbox.len(), 2, "{shared_inbox:?}");
    assert!(
        shared_inbox
            .iter()
            .all(|m| m.body.starts_with("(originally for advisor/research)\n"))
    );
    assert_eq!(inbox(&research).await.len(), 1);

    // The part before `/` must be an active principal.
    let err = daemon
        .client
        .send(&send("external:nobody/research", None))
        .await
        .expect_err("404");
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 404, .. }
    ));
}

/// 3ehu: `human@machine` is the human on another machine: human authority on human-only
/// routes, its own sender identity, and replies to its messages land back in its inbox.
#[tokio::test]
async fn human_visitor_has_human_authority_and_its_own_identity() {
    let (daemon, _tmp) = start_daemon(None).await;
    let created = daemon
        .client
        .create_token(&TokenCreateRequest {
            name: "human".to_string(),
            machine: Some("nuc".to_string()),
        })
        .await
        .expect("create human visitor token");
    assert_eq!(created.principal, "human@nuc");
    let nuc = Client::new(daemon.running.url.clone(), Some(created.token));

    // A human-only route works, exactly as for the local human.
    let other = nuc
        .create_token(&TokenCreateRequest {
            name: "advisor".to_string(),
            machine: None,
        })
        .await
        .expect("human@nuc may create tokens");
    assert_eq!(other.principal, "external:advisor");

    let sent = nuc
        .send(&SendRequest {
            to: Some("human".to_string()),
            body: "hello from the nuc".to_string(),
            kind: MessageKind::Question,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .expect("send");
    assert_eq!(sent[0].from, "human@nuc");

    let reply = daemon
        .client
        .send(&SendRequest {
            to: Some("human@nuc".to_string()),
            body: "got it".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: Some(sent[0].id.clone()),
            task: None,
        })
        .await
        .expect("reply to human@nuc");
    assert_eq!(reply[0].to, "human@nuc");
    let inbox = nuc
        .list_messages(&MessageQuery {
            to: Some("me".to_string()),
            ..Default::default()
        })
        .await
        .expect("inbox");
    assert_eq!(inbox.len(), 1);

    // The bare human's inbox is separate.
    let err = daemon
        .client
        .send(&SendRequest {
            to: Some("human@elsewhere".to_string()),
            body: "x".to_string(),
            kind: MessageKind::Note,
            when: When::Now,
            reply_to: None,
            task: None,
        })
        .await
        .unwrap_err();
    assert!(
        matches!(err, bridle_api::ClientError::Api { status: 404, .. }),
        "{err:?}"
    );
}
