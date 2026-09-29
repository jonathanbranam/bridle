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

fn sibling_repo(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).expect("mkdir");
    git(dir, &["init", "-q", "-b", "main"]);
    std::fs::write(dir.join("f"), "x").expect("write");
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "init"]);
}

#[tokio::test]
async fn paired_layout_creates_members_and_rm_cleans_them_refusing_dirty() {
    let elsewhere = tempfile::tempdir().expect("tempdir");
    let web = elsewhere.path().join("web-src");
    let docs = elsewhere.path().join("docs-src");
    sibling_repo(&web);
    sibling_repo(&docs);
    let root = elsewhere.path().join("wt/{task}");
    let cfg = format!(
        "[roles.manager]\nautostart = false\n[worktrees]\nlayout = \"paired\"\nroot = {:?}\n\
         [worktrees.pair.web]\npath = {:?}\n[worktrees.pair.docs]\npath = {:?}\nmode = \"symlink\"\n",
        root.to_string_lossy(),
        web.to_string_lossy(),
        docs.to_string_lossy()
    );
    let (daemon, _tmp) = start_daemon_verbatim_config(None, Some(&cfg)).await;
    let agent = daemon.client.spawn(&spawn_req("w1")).await.expect("spawn");

    let dir = elsewhere.path().join("wt/w1");
    assert_eq!(
        agent.worktree.as_deref(),
        Some(dir.join("repo").to_str().expect("utf8"))
    );
    assert!(dir.join("repo/.git").exists());
    assert!(dir.join("web/.git").exists());
    assert_eq!(std::fs::read_link(dir.join("docs")).expect("symlink"), docs);

    daemon
        .client
        .stop(&agent.id, &bridle_api::types::StopRequest { now: false })
        .await
        .expect("stop");
    std::fs::write(dir.join("web/scratch"), "dirty").expect("write");
    let err = daemon
        .client
        .remove(&agent.id, &RemoveQuery::default())
        .await
        .expect_err("dirty sibling refuses");
    assert!(err.to_string().contains("web"), "{err}");
    assert!(dir.join("repo").exists() && dir.join("web").exists());

    std::fs::remove_file(dir.join("web/scratch")).expect("clean");
    daemon
        .client
        .remove(&agent.id, &RemoveQuery::default())
        .await
        .expect("rm");
    assert!(!dir.join("repo").exists());
    assert!(!dir.join("web").exists());
    assert!(dir.join("docs").symlink_metadata().is_err());
    assert!(docs.join("f").exists(), "symlink target untouched");
}
