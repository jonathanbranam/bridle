//! `bridle renew`: stop the named agent, start its replacement fresh in the
//! same worktree/branch/role/model (docs/design/agent-host/agents.md,
//! Renewing).

mod support;

use bridle_api::types::{AgentState, RenewRequest, SpawnRequest, StopRequest, Workdir};
use support::{start_daemon, wait_for_state};

#[tokio::test]
async fn renew_replaces_the_process_in_the_same_worktree_and_branch() {
    let (daemon, _tmp) = start_daemon(None).await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    let before = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let old_pid = before.pid.expect("pid while idle");
    let old_session = before.session_id.clone();
    let worktree = before.worktree.clone().expect("worktree path");
    let branch = before.branch.clone().expect("branch");

    daemon
        .client
        .stop(&agent.id, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for_state(&daemon.client, &agent.id, AgentState::Stopped).await;

    let renewed = daemon
        .client
        .renew(
            &agent.id,
            &RenewRequest {
                ignore_budget: false,
            },
        )
        .await
        .expect("renew");

    // Same identity: nothing else addressing this agent needs to know it
    // was renewed.
    assert_eq!(renewed.id, agent.id, "renew keeps the same agent id");
    assert_eq!(renewed.name, agent.name, "renew keeps the same agent name");
    // Same worktree/branch: no new worktree created, no BranchExists error.
    assert_eq!(
        renewed.worktree.as_deref(),
        Some(worktree.as_str()),
        "renew reuses the existing worktree path"
    );
    assert_eq!(
        renewed.branch.as_deref(),
        Some(branch.as_str()),
        "renew reuses the existing branch"
    );
    // Fresh session, unlike `resume` which keeps the old one.
    assert_ne!(
        renewed.session_id, old_session,
        "renew starts a brand-new session, not --resume of the old one"
    );

    let after = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let new_pid = after.pid.expect("pid while idle after renew");
    assert_ne!(new_pid, old_pid, "renew starts a brand-new process");

    // Only one git worktree is registered for the branch: renew attached a
    // fresh process to the one that was already there, distinct from a
    // plain spawn (which would create a second worktree on a fresh branch,
    // or fail with BranchExists reusing this one).
    let list = tokio::process::Command::new("git")
        .arg("-C")
        .arg(&daemon.repo)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .await
        .expect("git worktree list");
    let out = String::from_utf8_lossy(&list.stdout);
    let count = out
        .matches(&format!("branch refs/heads/{branch}\n"))
        .count();
    assert_eq!(
        count, 1,
        "renew must not create a second worktree for the branch"
    );

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
