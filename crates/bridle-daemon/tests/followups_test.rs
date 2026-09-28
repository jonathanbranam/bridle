//! Fixes from the v1 follow-ups (docs/questions/open/v1-follow-ups-from-the-build-9c6e.md):
//! held messages across an exit, events by agent name, `rm` ordering and
//! usage history, caller provenance, the spend cap, Claude Code version
//! tracking and autostart prompts.

mod support;

use bridle_api::types::{
    AgentState, InterruptRequest, MessageKind, MessageState, RemoveQuery, SendRequest,
    SpawnRequest, StopRequest, When, Workdir,
};
use support::{
    start_daemon, start_daemon_with_config, wait_for_agent, wait_for_event, wait_for_state,
};

fn spawn_req(name: &str, prompt: Option<&str>, workdir: Workdir) -> SpawnRequest {
    SpawnRequest {
        role: "worker".to_string(),
        name: Some(name.to_string()),
        prompt: prompt.map(str::to_string),
        workdir: Some(workdir),
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
    }
}

#[tokio::test]
async fn a_held_message_survives_an_exit_and_is_delivered_on_resume() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let agent = c
        .spawn(&spawn_req("w1", Some("SLEEP 10"), Workdir::Repo))
        .await
        .expect("spawn");
    wait_for_state(c, &agent.id, AgentState::Working).await;

    let held = c
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "after the turn".to_string(),
                kind: MessageKind::Note,
                when: When::Idle,
                reply_to: None,
            },
        )
        .await
        .expect("send");
    assert_eq!(held.state, MessageState::Held);

    // SIGTERM mid-turn: no result, so the held message is never written.
    c.stop(&agent.id, &StopRequest { now: true })
        .await
        .expect("stop");
    c.resume(&agent.id, &Default::default())
        .await
        .expect("resume");

    wait_for_event(c, "message.delivered", Some(&agent.id), |e| {
        e.data["message"] == held.id.as_str()
    })
    .await;
}

#[tokio::test]
async fn events_can_be_filtered_by_agent_name() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    c.spawn(&spawn_req("named", Some("hi"), Workdir::Repo))
        .await
        .expect("spawn");
    wait_for_event(c, "turn.ended", Some("named"), |_| true).await;
}

#[tokio::test]
async fn rm_refuses_a_dirty_worktree_without_stopping_the_agent() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let agent = c
        .spawn(&spawn_req("dirty", None, Workdir::Worktree { base: None }))
        .await
        .expect("spawn");
    let wt = agent.worktree.clone().expect("worktree");
    std::fs::write(std::path::Path::new(&wt).join("scratch.txt"), "x").expect("write");

    let err = c
        .remove(&agent.id, &RemoveQuery::default())
        .await
        .expect_err("rm of a dirty worktree must be refused");
    assert!(err.to_string().contains("uncommitted"), "{err}");
    let after = c.get_agent(&agent.id).await.expect("get");
    assert!(
        after.state.is_running(),
        "refused rm stopped it: {:?}",
        after.state
    );

    c.remove(
        &agent.id,
        &RemoveQuery {
            force: true,
            delete_branch: true,
        },
    )
    .await
    .expect("forced rm");
}

#[tokio::test]
async fn rm_keeps_the_agents_usage() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let agent = c
        .spawn(&spawn_req("gone", Some("hi"), Workdir::Repo))
        .await
        .expect("spawn");
    wait_for_agent(c, &agent.id, |a| a.turns >= 1).await;
    c.remove(&agent.id, &RemoveQuery::default())
        .await
        .expect("rm");

    let usage = c.usage().await.expect("usage");
    let row = usage
        .agents
        .iter()
        .find(|a| a.agent == agent.id)
        .expect("removed agent still in usage");
    assert!(row.removed);
    assert_eq!(row.name, "gone");
    assert_eq!(row.turns, 1);
    assert!(row.tokens.input > 0);
    assert_eq!(usage.total_turns, 1);
}

#[tokio::test]
async fn interrupt_stop_and_resume_record_the_caller() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    let agent = c
        .spawn(&spawn_req("w1", Some("SLEEP 10"), Workdir::Repo))
        .await
        .expect("spawn");
    wait_for_state(c, &agent.id, AgentState::Working).await;

    c.interrupt(&agent.id, &InterruptRequest { drop_held: false })
        .await
        .expect("interrupt");
    wait_for_event(c, "agent.interrupted", Some(&agent.id), |e| {
        e.actor == "human"
    })
    .await;

    c.stop(&agent.id, &StopRequest { now: false })
        .await
        .expect("stop");
    wait_for_event(c, "agent.stop_requested", Some(&agent.id), |e| {
        e.actor == "human"
    })
    .await;

    c.resume(&agent.id, &Default::default())
        .await
        .expect("resume");
    wait_for_event(c, "agent.resumed", Some(&agent.id), |e| e.actor == "human").await;
}

#[tokio::test]
async fn a_spent_budget_stops_the_agent_and_resume_grants_another() {
    let config = "[roles.worker]\nmax_budget_usd = 0.0005\n";
    let (daemon, _tmp) = start_daemon_with_config(None, Some(config)).await;
    let c = &daemon.client;
    let agent = c
        .spawn(&spawn_req("capped", Some("hi"), Workdir::Repo))
        .await
        .expect("spawn");

    let stopped = wait_for_state(c, &agent.id, AgentState::Stopped).await;
    assert_eq!(stopped.exit.expect("exit").reason, "budget_exhausted");
    wait_for_event(c, "agent.budget_exhausted", Some(&agent.id), |_| true).await;

    let resumed = c
        .resume(&agent.id, &Default::default())
        .await
        .expect("resume");
    assert!(resumed.state.is_running());
}

#[tokio::test]
async fn a_new_claude_code_version_is_recorded_and_announced() {
    let (daemon, _tmp) = start_daemon(None).await;
    let c = &daemon.client;
    c.spawn(&spawn_req("a", Some("hi"), Workdir::Repo))
        .await
        .expect("spawn a");
    wait_for_event(c, "claude.version", None, |e| {
        e.data["version"] == "fake" && e.data["previous"].is_null()
    })
    .await;
    assert_eq!(
        c.status().await.expect("status").claude_version.as_deref(),
        Some("fake")
    );

    std::fs::write(daemon.repo.join(".fake-claude-version"), "9.9.9").expect("write version");
    c.spawn(&spawn_req("b", Some("hi"), Workdir::Repo))
        .await
        .expect("spawn b");
    wait_for_event(c, "claude.version", None, |e| {
        e.data["version"] == "9.9.9" && e.data["previous"] == "fake"
    })
    .await;
}

#[tokio::test]
async fn an_autostarted_role_gets_its_start_prompt() {
    let config = "[roles.manager]\nautostart = true\nstart_prompt = \"hello manager\"\n";
    let (daemon, _tmp) = start_daemon_with_config(None, Some(config)).await;
    wait_for_agent(&daemon.client, "manager", |a| a.turns >= 1).await;
}
