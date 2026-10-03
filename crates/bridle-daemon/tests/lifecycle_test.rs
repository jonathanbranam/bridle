//! §4-7: interrupt, stop/resume/rm, crash, orphan sweep.

mod support;

use bridle_api::types::{
    AgentState, InterruptRequest, RemoveQuery, RenewRequest, ResumeRequest, SendRequest,
    SpawnRequest, StopRequest, Workdir,
};
use support::{start_daemon, wait_for_agent, wait_for_state};

#[tokio::test]
async fn interrupt_during_sleep_ends_the_turn_and_agent_stays_usable() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SLEEP 10".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Working).await;

    let resp = tokio::time::timeout(
        support::HANG_GUARD_TIMEOUT,
        daemon
            .client
            .interrupt(&agent.id, &InterruptRequest { drop_held: false }),
    )
    .await
    .expect("interrupt hung")
    .expect("interrupt");
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
                task: None,
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
                .find(|m| m.id == id && m.state == bridle_api::types::MessageState::Read)
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
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("hi".to_string()),
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    // A session only exists to resume once its first turn has run.
    support::wait_for_event(
        &daemon.client,
        bridle_api::types::event_kind::TURN_ENDED,
        Some(&agent.id),
        |_| true,
    )
    .await;
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
                task: None,
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
                .find(|m| m.id == id && m.state == bridle_api::types::MessageState::Read)
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
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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
async fn rm_delete_branch_refuses_an_empty_branch_without_force() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    // No commit of its own: an ancestor of HEAD, but "no work", not "merged" (z4hd).
    let err = daemon
        .client
        .remove(
            &agent.id,
            &RemoveQuery {
                force: false,
                delete_branch: true,
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 409, .. }
    ));
}

#[tokio::test]
async fn rm_refuses_a_worktree_with_open_files_without_force() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    daemon
        .client
        .stop(&agent.id, &bridle_api::types::StopRequest { now: false })
        .await
        .expect("stop before rm");
    let wt = agent.worktree.clone().expect("worktree");

    // A file committed in the worktree, so opening it doesn't also make the
    // worktree dirty and fail for the wrong reason.
    let held_path = std::path::Path::new(&wt).join("held.txt");
    std::fs::write(&held_path, "hold me open").expect("write held file");
    for args in [
        vec!["add", "held.txt"],
        vec![
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=Test",
            "commit",
            "-q",
            "-m",
            "add held.txt",
        ],
    ] {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&wt)
            .args(&args)
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?} failed");
    }

    let mut holder = std::process::Command::new("tail")
        .arg("-f")
        .arg(&held_path)
        .spawn()
        .expect("spawn tail -f to hold the file open");

    // lsof isn't necessarily instantaneous to see a freshly opened fd.
    support::wait_for("a 409 conflict for the open file", || async {
        let result = daemon
            .client
            .remove(&agent.id, &RemoveQuery::default())
            .await;
        matches!(
            result,
            Err(bridle_api::ClientError::Api { status: 409, .. })
        )
        .then_some(())
    })
    .await;

    holder.kill().expect("kill holder");
    let _ = holder.wait();

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
async fn a_not_logged_in_result_crashes_the_agent_with_a_clear_reason() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("NOT_LOGGED_IN".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    let crashed = wait_for_state(&daemon.client, &agent.id, AgentState::Crashed).await;
    assert_eq!(
        crashed.exit.expect("exit info").reason,
        "claude is not logged in in this daemon's session"
    );
}

#[tokio::test]
async fn crash_is_reported_with_a_stderr_tail_and_pending_messages_deliver_on_resume() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("CRASH".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
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
                task: None,
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
                .find(|m| m.id == id && m.state == bridle_api::types::MessageState::Read)
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
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("SPAWN_CHILD".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");

    // `end_turn` (which bumps `turns`) commits before the agent's state
    // flips back to `idle` (supervisor.rs's Result handling), so waiting on
    // `turns >= 1` alone can catch it still `working`; wait for both.
    let done = wait_for_agent(&daemon.client, &agent.id, |a| {
        a.turns >= 1 && a.state == AgentState::Idle
    })
    .await;
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

/// v4nk: worker-role agents get no agent lifecycle authority
/// (docs/design/agent-host/roles-and-config.md) — a worker's own token must
/// be refused on spawn/interrupt/stop/resume/renew/rm of any agent,
/// including agents it didn't spawn itself.
#[tokio::test]
async fn worker_principal_is_refused_agent_lifecycle_endpoints() {
    let (daemon, _tmp) = start_daemon(None).await;
    let actor = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("actor".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn actor");
    wait_for_state(&daemon.client, &actor.id, AgentState::Idle).await;
    let victim = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("victim".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn victim");
    wait_for_state(&daemon.client, &victim.id, AgentState::Idle).await;

    let worker_client = daemon.agent_client(&actor.id);

    let err = worker_client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("victim2".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 403, .. }
    ));

    let err = worker_client
        .interrupt(&victim.id, &InterruptRequest { drop_held: false })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 403, .. }
    ));

    let err = worker_client
        .stop(&victim.id, &StopRequest { now: false })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 403, .. }
    ));

    let err = worker_client
        .resume(&victim.id, &ResumeRequest::default())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 403, .. }
    ));

    let err = worker_client
        .renew(&victim.id, &RenewRequest::default())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 403, .. }
    ));

    let err = worker_client
        .remove(&victim.id, &RemoveQuery::default())
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        bridle_api::ClientError::Api { status: 403, .. }
    ));
}

/// The other side of v4nk: manager and orchestrator principals keep full
/// lifecycle authority, matching roles-and-config.md.
#[tokio::test]
async fn manager_and_orchestrator_principals_keep_agent_lifecycle_authority() {
    let (daemon, _tmp) = start_daemon(None).await;

    for (role, name) in [("manager", "mgr"), ("orchestrator", "orc")] {
        let actor = daemon
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
            .expect("spawn actor");
        wait_for_state(&daemon.client, &actor.id, AgentState::Idle).await;
        let actor_client = daemon.agent_client(&actor.id);

        let victim = actor_client
            .spawn(&SpawnRequest {
                components: Vec::new(),
                role: "worker".to_string(),
                name: Some(format!("{name}-victim")),
                prompt: None,
                workdir: Some(Workdir::Worktree { base: None }),
                model: None,
                extra_allowed_tools: Vec::new(),
                extra_env: Vec::new(),
                ignore_budget: false,
            })
            .await
            .unwrap_or_else(|e| panic!("{role} spawn should succeed: {e}"));
        wait_for_state(&daemon.client, &victim.id, AgentState::Idle).await;

        actor_client
            .interrupt(&victim.id, &InterruptRequest { drop_held: false })
            .await
            .unwrap_or_else(|e| panic!("{role} interrupt should succeed: {e}"));

        let stopped = actor_client
            .stop(&victim.id, &StopRequest { now: false })
            .await
            .unwrap_or_else(|e| panic!("{role} stop should succeed: {e}"));
        assert_eq!(stopped.state, AgentState::Stopped);

        actor_client
            .resume(&victim.id, &ResumeRequest::default())
            .await
            .unwrap_or_else(|e| panic!("{role} resume should succeed: {e}"));
        wait_for_state(&daemon.client, &victim.id, AgentState::Idle).await;

        actor_client
            .renew(&victim.id, &RenewRequest::default())
            .await
            .unwrap_or_else(|e| panic!("{role} renew should succeed: {e}"));

        actor_client
            .stop(&victim.id, &StopRequest { now: false })
            .await
            .unwrap_or_else(|e| panic!("{role} stop before rm should succeed: {e}"));
        actor_client
            .remove(
                &victim.id,
                &RemoveQuery {
                    // The victim's branch is empty, so it isn't "merged"; this test is about authority.
                    force: true,
                    delete_branch: true,
                },
            )
            .await
            .unwrap_or_else(|e| panic!("{role} rm should succeed: {e}"));
    }
}
