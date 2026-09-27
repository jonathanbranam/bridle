//! §4-7: interrupt, stop/resume/rm, crash, orphan sweep.

mod support;

use std::time::Duration;

use bridle_api::types::{
    AgentState, InterruptRequest, RemoveQuery, SendRequest, SpawnRequest, Workdir,
};
use support::{start_daemon, wait_for_agent, wait_for_state};

#[tokio::test]
async fn interrupt_during_sleep_ends_the_turn_and_agent_stays_usable() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 10".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;

    let started = std::time::Instant::now();
    let resp = daemon
        .client
        .interrupt(&agent.id, &InterruptRequest { drop_held: false })
        .await
        .expect("interrupt");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "interrupt should be fast"
    );
    assert!(resp.receipt.is_object());

    let idle = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    assert!(idle.turns >= 1);

    // The agent is usable afterwards: send another message and see it
    // delivered.
    let msg = daemon
        .client
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "still there?".to_string(),
                kind: bridle_api::types::MessageKind::Note,
                when: bridle_api::types::When::Now,
                reply_to: None,
            },
        )
        .await
        .expect("send after interrupt");
    support::wait_for("post-interrupt message delivered", || {
        let client = &daemon.client;
        let id = msg.id.clone();
        async move {
            let list = client.list_messages(&Default::default()).await.ok()?;
            list.into_iter()
                .find(|m| m.id == id && m.state == bridle_api::types::MessageState::Delivered)
        }
    })
    .await;
}

#[tokio::test]
async fn stop_then_resume_keeps_the_session_and_answers_new_messages() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let session_id = agent.session_id.clone();

    let stopped = daemon
        .client
        .stop(&agent.id, &bridle_api::types::StopRequest { now: false })
        .await
        .expect("stop");
    assert_eq!(stopped.state, AgentState::Stopped);

    let resumed = daemon
        .client
        .resume(&agent.id, &Default::default())
        .await
        .expect("resume");
    assert_eq!(resumed.session_id, session_id);
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    let msg = daemon
        .client
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "ping".to_string(),
                kind: bridle_api::types::MessageKind::Note,
                when: bridle_api::types::When::Now,
                reply_to: None,
            },
        )
        .await
        .expect("send after resume");
    support::wait_for("resumed agent delivers a message", || {
        let client = &daemon.client;
        let id = msg.id.clone();
        async move {
            let list = client.list_messages(&Default::default()).await.ok()?;
            list.into_iter()
                .find(|m| m.id == id && m.state == bridle_api::types::MessageState::Delivered)
        }
    })
    .await;

    let wt = resumed.worktree.clone().expect("worktree");
    daemon
        .client
        .stop(&agent.id, &bridle_api::types::StopRequest { now: false })
        .await
        .expect("stop before rm");
    daemon
        .client
        .remove(&agent.id, &RemoveQuery::default())
        .await
        .expect("rm");
    assert!(
        !std::path::Path::new(&wt).exists(),
        "worktree should be gone"
    );
    // The branch is kept unless delete_branch was passed.
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&daemon.repo)
        .args(["branch", "--list", "bridle/w1"])
        .output()
        .expect("git branch --list");
    assert!(String::from_utf8_lossy(&out.stdout).contains("bridle/w1"));
}

#[tokio::test]
async fn rm_refuses_a_dirty_worktree_without_force() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let wt = agent.worktree.clone().expect("worktree");
    std::fs::write(std::path::Path::new(&wt).join("dirty.txt"), "scratch")
        .expect("write dirty file");

    let err = daemon
        .client
        .remove(&agent.id, &RemoveQuery::default())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 409, .. }
    ));

    daemon
        .client
        .remove(
            &agent.id,
            &RemoveQuery {
                force: true,
                delete_branch: true,
            },
        )
        .await
        .expect("force rm");
    assert!(!std::path::Path::new(&wt).exists());
}

#[tokio::test]
async fn crash_is_reported_with_a_stderr_tail_and_pending_messages_deliver_on_resume() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("CRASH".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    let crashed = wait_for_state(&daemon.client, &agent.id, AgentState::Crashed).await;
    let exit = crashed.exit.expect("exit info");
    assert!(
        exit.reason.contains("crash") || exit.reason.contains("CRASH"),
        "{}",
        exit.reason
    );

    // Send a message while the agent isn't running: it should be recorded
    // pending.
    let msg = daemon
        .client
        .send_to_agent(
            &agent.id,
            &SendRequest {
                to: None,
                body: "are you back?".to_string(),
                kind: bridle_api::types::MessageKind::Note,
                when: bridle_api::types::When::Now,
                reply_to: None,
            },
        )
        .await
        .expect("send while not running");
    assert_eq!(msg.state, bridle_api::types::MessageState::Pending);

    daemon
        .client
        .resume(&agent.id, &Default::default())
        .await
        .expect("resume after crash");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    support::wait_for("pending message delivered after resume", || {
        let client = &daemon.client;
        let id = msg.id.clone();
        async move {
            let list = client.list_messages(&Default::default()).await.ok()?;
            list.into_iter()
                .find(|m| m.id == id && m.state == bridle_api::types::MessageState::Delivered)
        }
    })
    .await;
}

#[tokio::test]
async fn spawn_child_orphan_is_swept_on_stop() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SPAWN_CHILD".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    let done = wait_for_agent(&daemon.client, &agent.id, |a| a.turns >= 1).await;
    assert_eq!(done.state, AgentState::Idle);

    let text_event = support::wait_for_event(
        &daemon.client,
        bridle_api::types::event_kind::AGENT_TEXT,
        Some(&agent.id),
        |e| {
            e.data
                .get("text")
                .and_then(|v| v.as_str())
                .is_some_and(|t| t.contains("pid="))
        },
    )
    .await;
    let text = text_event.data["text"].as_str().expect("text").to_string();
    let pid_str = text.split("pid=").nth(1).expect("pid present").trim();
    let child_pid: i32 = pid_str.parse().expect("pid parses");

    let stopped = daemon
        .client
        .stop(&agent.id, &bridle_api::types::StopRequest { now: false })
        .await
        .expect("stop");
    assert_eq!(stopped.state, AgentState::Stopped);

    let orphans = support::wait_for_event(
        &daemon.client,
        bridle_api::types::event_kind::AGENT_ORPHANS_KILLED,
        Some(&agent.id),
        |e| {
            e.data
                .get("count")
                .and_then(|v| v.as_u64())
                .is_some_and(|c| c >= 1)
        },
    )
    .await;
    assert!(orphans.data["count"].as_u64().unwrap() >= 1);

    assert!(
        nix::sys::signal::kill(nix::unistd::Pid::from_raw(child_pid), None).is_err(),
        "spawned child pid {child_pid} should be dead after stop"
    );
}
