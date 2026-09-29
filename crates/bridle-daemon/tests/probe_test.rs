//! `bridle probe` (docs/design/impact-and-conflicts.md): `git merge-tree` of an agent's
//! branch against the integration branch, in a temp repo.

mod support;

use bridle_api::types::{AgentState, ProbeOutcome, ProbeRequest, SpawnRequest, Workdir};
use support::{start_daemon, wait_for_state};

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@bridle.invalid"])
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

#[tokio::test]
async fn probe_reports_clean_then_the_conflicting_paths() {
    let (d, _tmp) = start_daemon(None).await;
    let a = d
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
    wait_for_state(&d.client, &a.id, AgentState::Idle).await;
    let wt = std::path::PathBuf::from(a.worktree.as_deref().expect("worktree"));

    let probe = |target: &str| ProbeRequest {
        target: Some(target.to_string()),
        branch: None,
    };
    let r = d.client.probe(&probe("w1")).await.expect("probe");
    assert_eq!(r.outcome, ProbeOutcome::Clean);
    assert_eq!(r.against, "main");

    std::fs::write(wt.join("f.txt"), "branch\n").expect("w");
    git(&wt, &["add", "."]);
    git(&wt, &["commit", "-qm", "branch adds f"]);
    std::fs::write(d.repo.join("f.txt"), "main\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main adds f"]);

    let r = d.client.probe(&probe("w1")).await.expect("probe");
    assert_eq!(r.outcome, ProbeOutcome::Conflict);
    assert_eq!(r.paths, vec!["f.txt".to_string()]);
    let by_branch = ProbeRequest {
        target: None,
        branch: a.branch.clone(),
    };
    assert_eq!(
        d.client.probe(&by_branch).await.expect("probe").outcome,
        ProbeOutcome::Conflict
    );
    assert!(d.client.probe(&probe("nobody")).await.is_err());
}
