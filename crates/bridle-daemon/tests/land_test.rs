//! `bridle land`: the integrator merges a branch in its own worktree, checks it, and only then
//! moves the integration branch (docs/design/agent-host/roles-and-config.md).

mod support;

use bridle_api::types::{LandRequest, NewTaskRequest, TaskKind, TaskState};
use support::{TestDaemon, start_daemon};

fn git(dir: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@bridle.invalid"])
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .trim()
        .to_string()
}

/// A branch off main's tip with one commit adding `file`.
fn branch_with(d: &TestDaemon, branch: &str, file: &str, content: &str) {
    git(&d.repo, &["branch", branch, "main"]);
    let wt = d.workspace.join(format!("scratch-{branch}"));
    git(
        &d.repo,
        &["worktree", "add", wt.to_str().expect("utf8"), branch],
    );
    let path = wt.join(file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, content).expect("write");
    git(&wt, &["add", "."]);
    git(&wt, &["commit", "-qm", "work"]);
    git(
        &d.repo,
        &["worktree", "remove", "--force", wt.to_str().expect("utf8")],
    );
}

async fn task(d: &TestDaemon, kind: TaskKind) -> String {
    d.client
        .new_task(&NewTaskRequest {
            components: Vec::new(),
            title: "t".to_string(),
            kind,
            body: String::new(),
            size: None,
        })
        .await
        .expect("task")
        .id
}

fn req(branch: &str, check: Option<&str>) -> LandRequest {
    LandRequest {
        branch: Some(branch.to_string()),
        check_cmd: check.map(str::to_string),
    }
}

async fn land_err(d: &TestDaemon, id: &str, r: &LandRequest) -> String {
    d.client
        .land_task(id, r)
        .await
        .expect_err("land refused")
        .to_string()
}

#[tokio::test]
async fn clean_land_moves_main_and_integrates_the_task() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    let before = git(&d.repo, &["rev-parse", "main"]);
    let tip = git(&d.repo, &["rev-parse", "b1"]);
    let r = d
        .client
        .land_task(&id, &req("b1", None))
        .await
        .expect("land");
    let after = git(&d.repo, &["rev-parse", "main"]);
    assert_ne!(before, after);
    assert_eq!(r.commit, after);
    assert_eq!(git(&d.repo, &["rev-parse", "main^2"]), tip);
    assert_eq!(r.task.state, TaskState::Integrated);
    assert!(
        r.notes.iter().any(|n| n.contains("skipped")),
        "{:?}",
        r.notes
    );
}

#[tokio::test]
async fn conflict_lands_nothing() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "branch\n");
    std::fs::write(d.repo.join("f.txt"), "main\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main adds f"]);
    let id = task(&d, TaskKind::Feature).await;
    let before = git(&d.repo, &["rev-parse", "main"]);
    let e = land_err(&d, &id, &req("b1", None)).await;
    assert!(e.contains("f.txt"), "{e}");
    assert_eq!(git(&d.repo, &["rev-parse", "main"]), before);
    assert_ne!(
        d.client.get_task(&id).await.expect("task").state,
        TaskState::Integrated
    );
}

#[tokio::test]
async fn failing_check_lands_nothing_and_reports_the_tail() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    let before = git(&d.repo, &["rev-parse", "main"]);
    let e = land_err(&d, &id, &req("b1", Some("echo boom; exit 1"))).await;
    assert!(e.contains("boom"), "{e}");
    assert_eq!(git(&d.repo, &["rev-parse", "main"]), before);
}

#[tokio::test]
async fn a_moved_main_is_refused() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    // The check moves main out from under the landing.
    let repo = d.repo.to_str().expect("utf8");
    let mover = format!(
        "export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@x GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@x; \
         c=$(git -C {repo} commit-tree main^{{tree}} -p main -m moved); \
         git -C {repo} update-ref refs/heads/main $c"
    );
    let e = land_err(&d, &id, &req("b1", Some(&mover))).await;
    assert!(e.contains("main moved, retry"), "{e}");
    assert_ne!(
        d.client.get_task(&id).await.expect("task").state,
        TaskState::Integrated
    );
}

#[tokio::test]
async fn architecture_changes_need_an_arch_revision() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "design/architecture/a.md", "x\n");
    let feature = task(&d, TaskKind::Feature).await;
    let before = git(&d.repo, &["rev-parse", "main"]);
    let e = land_err(&d, &feature, &req("b1", None)).await;
    assert!(e.contains("arch-revision"), "{e}");
    assert_eq!(git(&d.repo, &["rev-parse", "main"]), before);
    let arch = task(&d, TaskKind::ArchRevision).await;
    d.client
        .land_task(&arch, &req("b1", None))
        .await
        .expect("arch land");
    assert_ne!(git(&d.repo, &["rev-parse", "main"]), before);
}
