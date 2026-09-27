//! §2-3: spawning a worker with a prompt, and message delivery timing.

mod support;

use bridle_api::types::{
    AgentState, MessageKind, MessageQuery, MessageState, SendRequest, When, Workdir,
};
use bridle_api::types::{SpawnRequest, event_kind};
use support::{start_daemon, wait_for_agent, wait_for_event, wait_for_state};

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

async fn wait_for(client: &bridle_api::Client, id: &str) -> bridle_api::types::Message {
    support::wait_for(&format!("message {id} delivered"), || async {
        let m = client.list_messages(&MessageQuery::default()).await.ok()?;
        m.into_iter()
            .find(|m| m.id == id && m.state == MessageState::Delivered)
    })
    .await
}
