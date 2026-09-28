//! `bridle renew`: stop the named agent, start its replacement fresh in the
//! same worktree/branch/role/model (docs/design/agent-host/agents.md,
//! Renewing).

mod support;

use bridle_api::types::{AgentState, RenewRequest, SpawnRequest, StopRequest, Workdir};
use bridle_daemon::Overrides;
use support::{
    default_overrides, fake_claude_argv_and_env_dump_wrapper, start_daemon, wait_for_state,
};

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
            extra_env: Vec::new(),
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

/// A spawn's `--allow-tool`/`--env` overrides (k8dw/2ty9) are persisted on
/// the agent, not just applied to the first process: a `renew` must reapply
/// them too, or a worker silently loses them the moment it hits its
/// `wind_down_at` threshold mid-task.
#[tokio::test]
async fn renew_reapplies_the_spawn_s_extra_allowed_tools_and_env() {
    let dump_dir = tempfile::tempdir().expect("dump tempdir");
    let argv_path = dump_dir.path().join("argv.json");
    let env_path = dump_dir.path().join("env.json");
    let wrapper = fake_claude_argv_and_env_dump_wrapper(dump_dir.path(), &argv_path, &env_path);
    let overrides = Overrides {
        claude_program: wrapper.to_string_lossy().into_owned(),
        ..default_overrides()
    };

    let (daemon, _tmp) = support::start_daemon(Some(overrides)).await;

    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Worktree { base: None }),
            model: None,
            extra_allowed_tools: vec!["WebSearch".to_string()],
            extra_env: vec![("PIXELLAB_TOKEN".to_string(), "secret-1".to_string())],
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    let before = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let old_pid = before.pid.expect("pid while idle");

    // The governor also spawns its own short-lived `claude` invocations
    // through the same overridden `claude_program` (usage/rate-limit
    // checks, governor.rs), which dump to these same files too. Filter on
    // `--name w1` (only the agent's own process gets `cmd.name`) so a
    // governor poll racing the read can't be mistaken for it.
    async fn check_dump(argv_path: &std::path::Path, env_path: &std::path::Path) {
        let argv: Vec<String> = support::wait_for_dump(
            "fake-claude argv dump for w1",
            argv_path,
            |argv: &Vec<String>| argv.windows(2).any(|w| w[0] == "--name" && w[1] == "w1"),
        )
        .await;
        let flag_idx = argv
            .iter()
            .position(|a| a == "--allowedTools")
            .unwrap_or_else(|| panic!("--allowedTools flag present, got {argv:?}"));
        assert!(
            argv[flag_idx + 1..].iter().any(|a| a == "WebSearch"),
            "expected WebSearch among allowed tools, got {argv:?}"
        );
        let env: std::collections::HashMap<String, String> = support::wait_for_dump(
            "fake-claude env dump for w1",
            env_path,
            |env: &std::collections::HashMap<String, String>| {
                env.get("BRIDLE_AGENT_NAME").map(String::as_str) == Some("w1")
            },
        )
        .await;
        assert_eq!(
            env.get("PIXELLAB_TOKEN").map(String::as_str),
            Some("secret-1"),
            "expected PIXELLAB_TOKEN in the child's env, got {env:?}"
        );
    }
    check_dump(&argv_path, &env_path).await;

    daemon
        .client
        .stop(&agent.id, &StopRequest { now: true })
        .await
        .expect("stop");
    wait_for_state(&daemon.client, &agent.id, AgentState::Stopped).await;

    // The dump files already hold the original spawn's argv/env; remove them
    // so the next `check_dump` below can't pass on stale content and must
    // observe a fresh write from the renewed process.
    support::clear_dump(&argv_path);
    support::clear_dump(&env_path);

    daemon
        .client
        .renew(
            &agent.id,
            &RenewRequest {
                ignore_budget: false,
            },
        )
        .await
        .expect("renew");

    let after = wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let new_pid = after.pid.expect("pid while idle after renew");
    assert_ne!(new_pid, old_pid, "renew starts a brand-new process");

    // Same dump paths, freshly written by the renewed process at its own
    // startup (before any turn): this now reflects the renewed invocation,
    // not the original spawn.
    check_dump(&argv_path, &env_path).await;

    daemon.running.shutdown();
    daemon.running.join().await.expect("join");
}
