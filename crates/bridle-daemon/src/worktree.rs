//! Git worktree management for agents. Shells out to the `git` CLI via
//! `tokio::process::Command`. See docs/design/agent-host/daemon.md, agents.md.

use std::path::Path;

use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    #[error("git {args:?} failed: {stderr}")]
    Git { args: Vec<String>, stderr: String },
    #[error("branch {branch:?} already exists")]
    BranchExists { branch: String },
    #[error("io error running git: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid agent name {0:?}: expected [a-z0-9][a-z0-9-]{{0,39}}")]
    InvalidName(String),
}

pub(crate) async fn run_git(repo: &Path, args: &[&str]) -> Result<String, WorktreeError> {
    run_git_with_env(repo, args, &[], &[]).await
}

/// Like [`run_git`], but lets the caller set or remove environment variables
/// on the `git` child process, without touching the daemon's own process
/// environment. Used to exercise `ensure_orphan_branch` under an environment
/// with no git identity configured anywhere (see its test).
async fn run_git_with_env(
    repo: &Path,
    args: &[&str],
    set_env: &[(&str, &str)],
    remove_env: &[&str],
) -> Result<String, WorktreeError> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo).args(args);
    for var in remove_env {
        cmd.env_remove(var);
    }
    for (key, value) in set_env {
        cmd.env(key, value);
    }
    let output = cmd.output().await?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(WorktreeError::Git {
            args: args.iter().map(|s| s.to_string()).collect(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

/// Adds a new worktree at `path` on a new branch `branch`, based on `base`.
/// Errors clearly (`BranchExists`) if the branch already exists, rather than
/// surfacing git's raw message.
pub async fn add(repo: &Path, path: &Path, branch: &str, base: &str) -> Result<(), WorktreeError> {
    let path_str = path.to_string_lossy().into_owned();
    let result = run_git(repo, &["worktree", "add", "-b", branch, &path_str, base]).await;
    match result {
        Err(WorktreeError::Git { stderr, .. }) if stderr.contains("already exists") => {
            Err(WorktreeError::BranchExists {
                branch: branch.to_string(),
            })
        }
        other => other.map(|_| ()),
    }
}

/// Adds a new worktree at `path`, checking out `branch`, which must already
/// exist (unlike [`add`], no `-b`: this doesn't create the branch).
pub async fn add_existing(repo: &Path, path: &Path, branch: &str) -> Result<(), WorktreeError> {
    let path_str = path.to_string_lossy().into_owned();
    run_git(repo, &["worktree", "add", &path_str, branch])
        .await
        .map(|_| ())
}

/// Whether `branch` exists as a local branch, without needing a checkout.
pub async fn branch_exists(repo: &Path, branch: &str) -> Result<bool, WorktreeError> {
    Ok(Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ])
        .output()
        .await?
        .status
        .success())
}

/// Git's well-known empty-tree object id (`git hash-object -t tree /dev/null`),
/// the same in every repository, so it can be a constant instead of a shell-out.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// Creates `branch` as a fresh orphan (parentless) branch pointing at an
/// empty commit, if it doesn't already exist. No-op otherwise.
///
/// Deliberately plumbing (`commit-tree` + `update-ref`), not `git checkout
/// --orphan`: those write commit and ref objects directly and never touch
/// any working tree or index, so this is safe to call against the main
/// checkout's `repo` without any risk of disturbing it (docs/design/storage.md,
/// "The state branch").
pub async fn ensure_orphan_branch(
    repo: &Path,
    branch: &str,
    message: &str,
) -> Result<(), WorktreeError> {
    ensure_orphan_branch_with_env(repo, branch, message, &[], &[]).await
}

/// [`ensure_orphan_branch`], with `set_env`/`remove_env` applied to the
/// `commit-tree` child process's environment. Exists so the regression test
/// can prove the branch still gets created when no git identity is
/// configured anywhere, without mutating the daemon's own process
/// environment (see `run_git_with_env`).
async fn ensure_orphan_branch_with_env(
    repo: &Path,
    branch: &str,
    message: &str,
    set_env: &[(&str, &str)],
    remove_env: &[&str],
) -> Result<(), WorktreeError> {
    if branch_exists(repo, branch).await? {
        return Ok(());
    }
    let sha = run_git_with_env(
        repo,
        &[
            "-c",
            "user.email=bridle@localhost",
            "-c",
            "user.name=bridle",
            "commit-tree",
            EMPTY_TREE,
            "-m",
            message,
        ],
        set_env,
        remove_env,
    )
    .await?;
    run_git(
        repo,
        &["update-ref", &format!("refs/heads/{branch}"), sha.trim()],
    )
    .await?;
    Ok(())
}

/// Removes a worktree. `force` matches `git worktree remove --force`, needed
/// when the worktree has uncommitted changes.
pub async fn remove(repo: &Path, path: &Path, force: bool) -> Result<(), WorktreeError> {
    let path_str = path.to_string_lossy().into_owned();
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path_str);
    run_git(repo, &args).await.map(|_| ())
}

/// Whether `path` (a worktree or repo checkout) has any changes, tracked or
/// untracked. Untracked files count as dirty: an agent's worktree with
/// scratch files it hasn't committed is not safe to discard silently.
pub async fn is_dirty(path: &Path) -> Result<bool, WorktreeError> {
    let out = run_git(path, &["status", "--porcelain"]).await?;
    Ok(!out.trim().is_empty())
}

/// Whether any process still has a file open under `path` (a worktree),
/// checked with `lsof +D` (recursive, POSIX; available on macOS and Linux).
/// Returns a short description of the offending process/file for the error
/// message, or `None` if nothing was found or `lsof` isn't installed: a
/// missing `lsof` means the check can't run, not that it's safe to remove.
pub async fn open_file_holder(path: &Path) -> Result<Option<String>, WorktreeError> {
    let path_str = path.to_string_lossy().into_owned();
    let output = match Command::new("lsof").args(["+D", &path_str]).output().await {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    // lsof exits non-zero when it finds nothing under `path`: its normal
    // "no matches" result, not a failure, so only the stdout matters.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let offender = stdout.lines().skip(1).find(|line| !line.trim().is_empty());
    Ok(offender.map(|line| {
        let fields: Vec<&str> = line.split_whitespace().collect();
        match (fields.first(), fields.get(1), fields.last()) {
            (Some(command), Some(pid), Some(name)) => format!("{command} (pid {pid}) has {name} open"),
            _ => line.trim().to_string(),
        }
    }))
}

/// Deletes a branch. `force` matches `git branch -D` vs `-d`.
pub async fn delete_branch(repo: &Path, branch: &str, force: bool) -> Result<(), WorktreeError> {
    let flag = if force { "-D" } else { "-d" };
    run_git(repo, &["branch", flag, branch]).await.map(|_| ())
}

/// Drop git's records of worktrees whose directories no longer exist.
pub async fn prune(repo: &Path) -> Result<(), WorktreeError> {
    run_git(repo, &["worktree", "prune"]).await.map(|_| ())
}

/// Whether `branch` exists and is fully merged into the repo's `HEAD`, i.e.
/// whether `git branch -d` would succeed. A missing branch counts as merged.
pub async fn is_merged(repo: &Path, branch: &str) -> Result<bool, WorktreeError> {
    let exists = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ])
        .output()
        .await?
        .status
        .success();
    if !exists {
        return Ok(true);
    }
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge-base", "--is-ancestor", branch, "HEAD"])
        .output()
        .await?
        .status;
    Ok(status.success())
}

/// Whether `path` is inside a git repository (worktree or main checkout).
pub async fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--git-dir"])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The top-level directory of the repository containing `path`.
pub async fn repo_toplevel(path: &Path) -> Result<std::path::PathBuf, WorktreeError> {
    let out = run_git(path, &["rev-parse", "--show-toplevel"]).await?;
    Ok(std::path::PathBuf::from(out.trim()))
}

/// Agent names double as worktree directory names and branch suffixes, so
/// they're restricted to what's safe in both: lowercase alnum and hyphens,
/// starting with an alnum, at most 40 characters.
pub fn validate_agent_name(name: &str) -> Result<(), WorktreeError> {
    let bytes = name.as_bytes();
    let first_ok = bytes
        .first()
        .is_some_and(|&b| b.is_ascii_lowercase() || b.is_ascii_digit());
    let len_ok = !bytes.is_empty() && bytes.len() <= 40;
    let rest_ok = bytes
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if first_ok && len_ok && rest_ok {
        Ok(())
    } else {
        Err(WorktreeError::InvalidName(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    async fn init_repo(dir: &Path) {
        let run = |args: &'static [&'static str]| {
            let dir = dir.to_path_buf();
            async move {
                let out = Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(args)
                    .output()
                    .await
                    .expect("run git");
                assert!(
                    out.status.success(),
                    "git {args:?} failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        };
        std::fs::create_dir_all(dir).expect("mkdir repo");
        run(&["init", "-q"]).await;
        run(&[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=Test",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            "init",
        ])
        .await;
    }

    #[tokio::test]
    async fn add_dirty_remove_and_delete_branch() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;

        let wt_path = tmp.path().join("wt").join("w1");
        add(&repo, &wt_path, "bridle/w1", "HEAD")
            .await
            .expect("add worktree");
        assert!(wt_path.join(".git").exists());

        assert!(!is_dirty(&wt_path).await.expect("is_dirty clean"));

        std::fs::write(wt_path.join("scratch.txt"), "hello").expect("write scratch file");
        assert!(
            is_dirty(&wt_path)
                .await
                .expect("is_dirty with untracked file")
        );

        // Can't remove a dirty worktree without force.
        assert!(remove(&repo, &wt_path, false).await.is_err());
        remove(&repo, &wt_path, true).await.expect("force remove");
        assert!(!wt_path.exists());

        delete_branch(&repo, "bridle/w1", true)
            .await
            .expect("delete branch");
    }

    #[tokio::test]
    async fn open_file_holder_finds_and_clears_a_held_open_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let wt_path = tmp.path().join("wt").join("w3");
        add(&repo, &wt_path, "bridle/w3", "HEAD")
            .await
            .expect("add worktree");

        assert_eq!(
            open_file_holder(&wt_path).await.expect("no holder yet"),
            None
        );

        let held_path = wt_path.join("held.txt");
        std::fs::write(&held_path, "hold me open").expect("write held file");
        let mut holder = std::process::Command::new("tail")
            .arg("-f")
            .arg(&held_path)
            .spawn()
            .expect("spawn tail -f");

        let found = poll_until_some(|| open_file_holder(&wt_path)).await;
        assert!(found.contains("tail"), "expected tail in {found:?}");

        holder.kill().expect("kill holder");
        let _ = holder.wait();

        poll_until_none(|| open_file_holder(&wt_path)).await;
    }

    /// Polls `check` for up to 5s until it returns `Some`, panicking otherwise.
    /// `lsof` and process teardown aren't instantaneous, so a single call can
    /// race a holder that only just spawned or only just exited.
    async fn poll_until_some<F, Fut>(mut check: F) -> String
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<Option<String>, WorktreeError>>,
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(found) = check().await.expect("lsof") {
                return found;
            }
            assert!(std::time::Instant::now() < deadline, "timed out waiting");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn poll_until_none<F, Fut>(mut check: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<Option<String>, WorktreeError>>,
    {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if check().await.expect("lsof").is_none() {
                return;
            }
            assert!(std::time::Instant::now() < deadline, "timed out waiting");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    #[tokio::test]
    async fn is_merged_tracks_commits_ahead_of_head() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        let wt_path = tmp.path().join("wt").join("w2");
        add(&repo, &wt_path, "bridle/w2", "HEAD")
            .await
            .expect("add worktree");
        assert!(is_merged(&repo, "bridle/w2").await.expect("fresh branch"));
        assert!(
            is_merged(&repo, "no-such-branch")
                .await
                .expect("missing branch")
        );

        let out = Command::new("git")
            .arg("-C")
            .arg(&wt_path)
            .args(["-c", "user.email=t@e.com", "-c", "user.name=T"])
            .args(["commit", "--allow-empty", "-q", "-m", "ahead"])
            .output()
            .await
            .expect("commit");
        assert!(out.status.success());
        assert!(!is_merged(&repo, "bridle/w2").await.expect("branch ahead"));

        // An rm that already removed the directory can still finish.
        std::fs::remove_dir_all(&wt_path).expect("rm dir");
        prune(&repo).await.expect("prune");
        delete_branch(&repo, "bridle/w2", true)
            .await
            .expect("force delete");
    }

    #[tokio::test]
    async fn add_errors_clearly_when_branch_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;

        let wt1 = tmp.path().join("wt1");
        add(&repo, &wt1, "bridle/dup", "HEAD")
            .await
            .expect("first add");

        let wt2 = tmp.path().join("wt2");
        let err = add(&repo, &wt2, "bridle/dup", "HEAD")
            .await
            .expect_err("second add should fail");
        assert!(
            matches!(err, WorktreeError::BranchExists { .. }),
            "expected BranchExists, got {err:?}"
        );
    }

    #[tokio::test]
    async fn is_git_repo_and_toplevel() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;

        assert!(is_git_repo(&repo).await);
        assert!(!is_git_repo(tmp.path()).await || tmp.path() == repo);

        let top = repo_toplevel(&repo).await.expect("toplevel");
        // Resolve symlinks (e.g. macOS /tmp -> /private/tmp) before comparing.
        assert_eq!(
            std::fs::canonicalize(top).expect("canon"),
            std::fs::canonicalize(&repo).expect("canon")
        );
    }

    #[tokio::test]
    async fn ensure_orphan_branch_is_idempotent_and_never_touches_the_checkout() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;

        assert!(!branch_exists(&repo, "bridle").await.expect("check"));
        let status_before = run_git(&repo, &["status", "--porcelain"])
            .await
            .expect("status");
        let index_mtime_before = std::fs::metadata(repo.join(".git/index"))
            .expect("stat index")
            .modified()
            .expect("mtime");

        ensure_orphan_branch(&repo, "bridle", "initial bridle state")
            .await
            .expect("create orphan branch");
        assert!(branch_exists(&repo, "bridle").await.expect("check"));

        // Calling it again with the branch already there is a no-op, not
        // an error, and still leaves the checkout untouched.
        ensure_orphan_branch(&repo, "bridle", "initial bridle state")
            .await
            .expect("idempotent");

        let status_after = run_git(&repo, &["status", "--porcelain"])
            .await
            .expect("status");
        assert_eq!(status_before, status_after);
        assert!(status_after.trim().is_empty());
        let index_mtime_after = std::fs::metadata(repo.join(".git/index"))
            .expect("stat index")
            .modified()
            .expect("mtime");
        assert_eq!(
            index_mtime_before, index_mtime_after,
            "creating the orphan branch must not touch the index"
        );

        // The new branch has no history in common with HEAD.
        let out = run_git(&repo, &["merge-base", "--is-ancestor", "bridle", "HEAD"]).await;
        assert!(
            out.is_err(),
            "orphan branch must not be an ancestor of HEAD"
        );
    }

    #[tokio::test]
    async fn ensure_orphan_branch_succeeds_with_no_git_identity_anywhere() {
        // Reproduces GitHub Actions runners, which have no git author
        // identity configured at all (unlike a developer's machine): no
        // global/system gitconfig with user.name/user.email, and no
        // GIT_AUTHOR_*/GIT_COMMITTER_*/EMAIL env vars.
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;

        let empty_home = tmp.path().join("empty-home");
        std::fs::create_dir_all(&empty_home).expect("mkdir empty home");
        let home = empty_home.to_string_lossy().into_owned();
        let global_config = empty_home.join(".gitconfig").to_string_lossy().into_owned();

        ensure_orphan_branch_with_env(
            &repo,
            "bridle",
            "initial bridle state",
            &[
                ("HOME", home.as_str()),
                ("GIT_CONFIG_GLOBAL", global_config.as_str()),
                ("GIT_CONFIG_NOSYSTEM", "1"),
            ],
            &[
                "GIT_AUTHOR_NAME",
                "GIT_AUTHOR_EMAIL",
                "GIT_COMMITTER_NAME",
                "GIT_COMMITTER_EMAIL",
                "EMAIL",
            ],
        )
        .await
        .expect(
            "ensure_orphan_branch must supply its own identity, not rely on one being configured",
        );
        assert!(branch_exists(&repo, "bridle").await.expect("check"));
    }

    #[tokio::test]
    async fn add_existing_checks_out_a_branch_without_minus_b() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;
        ensure_orphan_branch(&repo, "bridle", "initial bridle state")
            .await
            .expect("create orphan branch");

        let wt_path = tmp.path().join("state");
        add_existing(&repo, &wt_path, "bridle")
            .await
            .expect("add existing");
        assert!(wt_path.join(".git").exists());
        // The orphan branch has no files: nothing but the worktree's `.git` link.
        let entries: Vec<_> = std::fs::read_dir(&wt_path)
            .expect("read dir")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from(".git")]);
    }

    #[test]
    fn agent_name_validation() {
        for ok in ["w1", "worker-2", "a", "abc-def-123"] {
            assert!(validate_agent_name(ok).is_ok(), "{ok:?} should be valid");
        }
        for bad in [
            "",
            "-w1",
            "Worker",
            "has_underscore",
            "has space",
            &"a".repeat(41),
        ] {
            assert!(
                validate_agent_name(bad).is_err(),
                "{bad:?} should be invalid"
            );
        }
    }
}
