//! `bridle land`: the integrator merges a branch in its own worktree, checks it, and only then
//! moves the integration branch (docs/design/agent-host/roles-and-config.md).

mod support;

use bridle_api::types::{LandRequest, NewTaskRequest, TaskKind, TaskState};
use support::{TestDaemon, start_daemon};

fn git(dir: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
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
            for_human: false,
            priority: None,
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
        checked_commit: None,
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
    // One single-parent squash commit carrying the trailers; the branch then reads as merged.
    assert_eq!(
        git(&d.repo, &["rev-list", "--parents", "-1", "main"])
            .split_whitespace()
            .count(),
        2,
        "single parent"
    );
    let msg = git(&d.repo, &["log", "-1", "--format=%B", "main"]);
    assert!(msg.starts_with(&format!("{id}: t\n")), "{msg}");
    assert!(msg.contains(&format!("Task: {id}")), "{msg}");
    assert!(msg.contains("Branch: b1"), "{msg}");
    assert_eq!(git(&d.repo, &["show", "main:f.txt"]), "x");
    // Landing removes the branch; bring it back to show the trailer alone marks it merged.
    git(&d.repo, &["branch", "-f", "b1", &tip]);
    assert!(
        bridle_daemon::worktree::is_merged(&d.repo, "b1")
            .await
            .expect("is_merged")
    );
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
    // Main has moved past the branch's base, so the check runs.
    std::fs::write(d.repo.join("other.txt"), "y\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main moves"]);
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
    // Main has moved past the branch's base, so the check runs.
    std::fs::write(d.repo.join("other.txt"), "y\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main moves"]);
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

#[tokio::test]
async fn landing_into_a_checked_out_branch_leaves_it_clean() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    assert_eq!(git(&d.repo, &["symbolic-ref", "--short", "HEAD"]), "main");
    let r = d
        .client
        .land_task(&id, &req("b1", None))
        .await
        .expect("land");
    assert_eq!(git(&d.repo, &["rev-parse", "HEAD"]), r.commit);
    assert_eq!(git(&d.repo, &["status", "--porcelain", "-uno"]), "");
    assert_eq!(
        std::fs::read_to_string(d.repo.join("f.txt")).expect("landed file"),
        "x\n"
    );
}

#[tokio::test]
async fn a_dirty_checked_out_branch_refuses_the_landing() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    std::fs::write(d.repo.join("tracked.txt"), "clean\n").expect("write");
    git(&d.repo, &["add", "tracked.txt"]);
    git(&d.repo, &["commit", "-qm", "track"]);
    std::fs::write(d.repo.join("tracked.txt"), "dirty\n").expect("dirty");
    let before = git(&d.repo, &["rev-parse", "main"]);
    let e = land_err(&d, &id, &req("b1", None)).await;
    assert!(e.contains("uncommitted"), "{e}");
    assert_eq!(git(&d.repo, &["rev-parse", "main"]), before);
    assert_ne!(
        d.client.get_task(&id).await.expect("task").state,
        TaskState::Integrated
    );
}

fn checked(branch: &str, sha: Option<String>) -> LandRequest {
    LandRequest {
        checked_commit: sha,
        ..req(branch, Some("exit 1"))
    }
}

#[tokio::test]
async fn a_fast_forward_of_the_reported_checked_commit_skips_the_check() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    let tip = git(&d.repo, &["rev-parse", "b1"]);
    let r = d
        .client
        .land_task(&id, &checked("b1", Some(tip)))
        .await
        .expect("a failing check must not run");
    assert!(
        r.notes
            .iter()
            .any(|n| n.contains("check skipped: fast-forward of an unchanged base")),
        "{:?}",
        r.notes
    );
}

#[tokio::test]
async fn a_newer_commit_than_the_reported_one_runs_the_check() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let reported = git(&d.repo, &["rev-parse", "b1"]);
    let wt = d.workspace.join("scratch-more");
    git(
        &d.repo,
        &["worktree", "add", wt.to_str().expect("utf8"), "b1"],
    );
    std::fs::write(wt.join("g.txt"), "z\n").expect("write");
    git(&wt, &["add", "."]);
    git(&wt, &["commit", "-qm", "later"]);
    git(
        &d.repo,
        &["worktree", "remove", "--force", wt.to_str().expect("utf8")],
    );
    let id = task(&d, TaskKind::Feature).await;
    let e = land_err(&d, &id, &checked("b1", Some(reported))).await;
    assert!(e.contains("check failed"), "{e}");
}

#[tokio::test]
async fn no_reported_commit_runs_the_check() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    let id = task(&d, TaskKind::Feature).await;
    let e = land_err(&d, &id, &checked("b1", None)).await;
    assert!(e.contains("check failed"), "{e}");
}

#[tokio::test]
async fn a_moved_base_runs_the_check_even_with_the_tip_reported() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    std::fs::write(d.repo.join("other.txt"), "y\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main moves"]);
    let id = task(&d, TaskKind::Feature).await;
    let tip = git(&d.repo, &["rev-parse", "b1"]);
    let e = land_err(&d, &id, &checked("b1", Some(tip))).await;
    assert!(e.contains("check failed"), "{e}");
}

#[tokio::test]
async fn a_moved_base_runs_the_check_and_says_so() {
    let (d, _tmp) = start_daemon(None).await;
    branch_with(&d, "b1", "f.txt", "x\n");
    std::fs::write(d.repo.join("other.txt"), "y\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main moves"]);
    let id = task(&d, TaskKind::Feature).await;
    let r = d
        .client
        .land_task(&id, &req("b1", Some("true")))
        .await
        .expect("land");
    assert!(
        r.notes.iter().any(|n| n.contains("check ran")),
        "{:?}",
        r.notes
    );
}

/// A check whose output ends with nextest's summary line for `n` tests.
fn counting(n: u64) -> String {
    format!("echo '     Summary [   1.0s] {n} tests run: {n} passed, 0 skipped'")
}

fn count_file(d: &TestDaemon) -> std::path::PathBuf {
    d.workspace.join("last-full-test-count")
}

/// Land a fresh branch whose base has moved, so the check runs.
async fn land_moved(d: &TestDaemon, branch: &str, check: &str) -> Result<(), String> {
    branch_with(d, branch, &format!("{branch}.txt"), "x\n");
    std::fs::write(d.repo.join(format!("m-{branch}.txt")), "y\n").expect("w");
    git(&d.repo, &["add", "."]);
    git(&d.repo, &["commit", "-qm", "main moves"]);
    let id = task(d, TaskKind::Feature).await;
    d.client
        .land_task(&id, &req(branch, Some(check)))
        .await
        .map(drop)
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn a_passing_full_run_records_its_test_count_and_bands_the_next() {
    let (d, _tmp) = start_daemon(None).await;
    land_moved(&d, "b1", &counting(100)).await.expect("land");
    assert_eq!(
        std::fs::read_to_string(count_file(&d))
            .expect("count")
            .trim(),
        "100"
    );
    // Well inside the band: lands and updates the count.
    land_moved(&d, "b2", &counting(120)).await.expect("land");
    // Far above, far below and zero all fail, leaving the count alone.
    for n in [500, 10, 0] {
        let e = land_moved(&d, &format!("c{n}"), &counting(n))
            .await
            .expect_err("out of band");
        assert!(e.contains("sane band"), "{e}");
    }
    assert_eq!(
        std::fs::read_to_string(count_file(&d))
            .expect("count")
            .trim(),
        "120"
    );
}

#[tokio::test]
async fn zero_tests_fail_even_without_a_last_count() {
    let (d, _tmp) = start_daemon(None).await;
    let e = land_moved(&d, "b1", &counting(0)).await.expect_err("zero");
    assert!(e.contains("sane band"), "{e}");
}
