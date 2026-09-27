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

async fn run_git(repo: &Path, args: &[&str]) -> Result<String, WorktreeError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .await?;
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
