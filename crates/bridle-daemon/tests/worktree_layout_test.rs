//! `[worktrees] layout = "root"` puts a worker's worktree at the configured path, and
//! `bridle rm` still removes it (docs/design/worktrees-and-ports.md).

mod support;

use bridle_api::types::{RemoveQuery, SpawnRequest};
use support::start_daemon_verbatim_config;

fn spawn_req(name: &str) -> SpawnRequest {
    SpawnRequest {
        role: "worker".to_string(),
        name: Some(name.to_string()),
        prompt: None,
        workdir: None,
        model: None,
        extra_allowed_tools: Vec::new(),
        extra_env: Vec::new(),
        ignore_budget: false,
        components: Vec::new(),
    }
}

#[tokio::test]
async fn root_layout_creates_the_worktree_at_the_resolved_path_and_rm_removes_it() {
    let elsewhere = tempfile::tempdir().expect("tempdir");
    let root = elsewhere.path().join("{project}-wt/{task}");
    let cfg = format!(
        "[roles.manager]\nautostart = false\n[worktrees]\nlayout = \"root\"\nroot = {:?}\n",
        root.to_string_lossy()
    );
    let (daemon, tmp) = start_daemon_verbatim_config(None, Some(&cfg)).await;
    let agent = daemon.client.spawn(&spawn_req("w1")).await.expect("spawn");

    let expected = elsewhere.path().join("repo-wt/w1");
    assert_eq!(
        agent.worktree.as_deref(),
        Some(expected.to_str().expect("utf8"))
    );
    assert!(expected.join(".git").exists());
    assert!(!tmp.path().join("wt/w1").exists());

    daemon
        .client
        .stop(&agent.id, &bridle_api::types::StopRequest { now: false })
        .await
        .expect("stop");
    daemon
        .client
        .remove(&agent.id, &RemoveQuery::default())
        .await
        .expect("rm");
    assert!(!expected.exists());
}

#[tokio::test]
async fn default_layout_stays_under_the_workspace() {
    let (daemon, tmp) =
        start_daemon_verbatim_config(None, Some("[roles.manager]\nautostart = false\n")).await;
    let agent = daemon.client.spawn(&spawn_req("w1")).await.expect("spawn");
    let expected = tmp.path().join("wt/w1");
    assert_eq!(
        agent.worktree.as_deref(),
        Some(expected.to_str().expect("utf8"))
    );
}
