//! Git worktree management for agents. Shells out to the `git` CLI via
//! `tokio::process::Command`. See docs/design/agent-host/daemon.md, agents.md.

use std::path::{Path, PathBuf};

use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    #[error("git {args:?} failed: {stderr}")]
    Git { args: Vec<String>, stderr: String },
    #[error("branch {branch:?} already exists")]
    BranchExists { branch: String },
    #[error("io error running git: {0}")]
    Io(#[from] std::io::Error),
    #[error("worktree setup command {command:?} {outcome}; last output:\n{tail}")]
    Setup {
        command: String,
        outcome: String,
        tail: String,
    },
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

/// Creates one paired member at `path`: a symlink to the sibling's checkout, or
/// a worktree of it on `branch` from the sibling's own HEAD.
pub async fn add_pair(
    member: &crate::config::PairMember,
    path: &Path,
    branch: &str,
) -> Result<(), WorktreeError> {
    match member.mode {
        crate::config::PairMode::Worktree => add(&member.path, path, branch, "HEAD").await,
        crate::config::PairMode::Symlink => {
            if !member.path.is_dir() {
                return Err(WorktreeError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("paired repo {} does not exist", member.path.display()),
                )));
            }
            std::fs::create_dir_all(path.parent().unwrap_or(path))?;
            std::os::unix::fs::symlink(&member.path, path)?;
            Ok(())
        }
    }
}

/// Removes one paired member created by [`add_pair`]. `force` as for [`remove`];
/// `branch` is deleted (`-D`) from the sibling repo when given.
pub async fn remove_pair(
    member: &crate::config::PairMember,
    path: &Path,
    force: bool,
    branch: Option<&str>,
) -> Result<(), WorktreeError> {
    match member.mode {
        crate::config::PairMode::Symlink => {
            if path.symlink_metadata().is_ok() {
                std::fs::remove_file(path)?;
            }
        }
        crate::config::PairMode::Worktree => {
            if path.exists() {
                remove(&member.path, path, force).await?;
            } else {
                prune(&member.path).await?;
            }
            if let Some(branch) = branch {
                delete_branch_if_exists(&member.path, branch).await?;
            }
        }
    }
    Ok(())
}

/// Where `warm_target` copies from: the integration worktree's `target/` (kept fresh by the
/// background warm build) when it exists and no build is running in it, or it is newer than the
/// clone's; else the clone's.
pub fn warm_source(repo: &Path, integration: &Path, building: bool) -> PathBuf {
    let clone = repo.join("target");
    let fresh = integration.join("target");
    if !fresh.is_dir() {
        return clone;
    }
    let newer = || match (mtime(&fresh), mtime(&clone)) {
        (Some(f), Some(c)) => f > c,
        (Some(_), None) => true,
        _ => false,
    };
    if !building || newer() { fresh } else { clone }
}

fn mtime(p: &Path) -> Option<std::time::SystemTime> {
    p.metadata().ok()?.modified().ok()
}

/// Clones a prior build's `target/` (see `warm_source`) into the new worktree so its first
/// build is incremental, not cold (ticket b7cz). `cp -cR` is an APFS copy-on-write clone:
/// near-instant, no extra disk. macOS only; elsewhere a no-op. Best effort: a missing `target/`
/// or a failed copy is logged and never fails the spawn. Never writes to the source.
pub async fn warm_target(repo: &Path, integration: &Path, worktree: &Path) {
    if !cfg!(target_os = "macos") {
        return;
    }
    let src = warm_source(repo, integration, crate::warm_build::building());
    let age_secs = mtime(&src)
        .and_then(|m| m.elapsed().ok())
        .map(|d| d.as_secs());
    tracing::info!(source = %src.display(), ?age_secs, "warming worktree target/");
    let dst = worktree.join("target");
    if !src.is_dir() || dst.exists() {
        return;
    }
    match Command::new("cp")
        .arg("-cR")
        .arg(&src)
        .arg(&dst)
        .output()
        .await
    {
        Ok(out) if out.status.success() => {}
        Ok(out) => tracing::warn!(
            stderr = %String::from_utf8_lossy(&out.stderr).trim(),
            "warming worktree target/ failed; continuing cold"
        ),
        Err(e) => tracing::warn!(error = %e, "warming worktree target/ failed; continuing cold"),
    }
}

/// Copies each `[worktrees] copy` file from the clone into the same path in the worktree
/// (`std::fs::copy` keeps the mode, so a 0600 token file stays 0600). A missing or unsafe
/// entry is skipped with a warning; this never fails the spawn.
pub fn copy_files(repo: &Path, worktree: &Path, files: &[String]) {
    for rel in files {
        let src = repo.join(rel);
        if !crate::config::is_safe_relative(rel) || !src.is_file() {
            tracing::warn!(file = %rel, "worktree copy: not a repo-relative existing file; skipped");
            continue;
        }
        let dst = worktree.join(rel);
        let result = match dst.parent() {
            Some(dir) => std::fs::create_dir_all(dir).and_then(|()| std::fs::copy(&src, &dst)),
            None => std::fs::copy(&src, &dst),
        };
        if let Err(e) = result {
            tracing::warn!(file = %rel, error = %e, "worktree copy failed; skipped");
        }
    }
}

/// Runs the project's `[worktrees] setup` command (`sh -c`, cwd = the worktree). The env is the
/// daemon's minus `BRIDLE_*`, so a setup script never sees an agent token. Fails on non-zero
/// exit or after `timeout`, with the command, status and the last ~20 lines of output.
pub async fn run_setup(
    worktree: &Path,
    command: &str,
    timeout: std::time::Duration,
) -> Result<(), WorktreeError> {
    let started = std::time::Instant::now();
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(command)
        .current_dir(worktree)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("BRIDLE_") {
            cmd.env_remove(key);
        }
    }
    let fail = |outcome: String, tail: String| WorktreeError::Setup {
        command: command.to_string(),
        outcome,
        tail,
    };
    // Dropping the future on timeout kills the child (kill_on_drop).
    let out = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(r) => r?,
        Err(_) => {
            return Err(fail(
                format!("timed out after {}s", timeout.as_secs()),
                String::new(),
            ));
        }
    };
    if !out.status.success() {
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&out.stderr));
        let lines: Vec<&str> = text.lines().collect();
        let tail = lines[lines.len().saturating_sub(20)..].join("\n");
        return Err(fail(format!("failed with {}", out.status), tail));
    }
    tracing::info!(
        command,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "worktree setup done"
    );
    Ok(())
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
///
/// A directory git no longer lists (its record was pruned) can't go through
/// `git worktree remove`; it is deleted directly, and without `force` only when
/// nothing but the `.git` pointer file is left, since there is no index to say
/// what else is safe to lose.
pub async fn remove(repo: &Path, path: &Path, force: bool) -> Result<(), WorktreeError> {
    if !is_registered(repo, path).await? {
        if !force {
            let leftover = std::fs::read_dir(path)?
                .filter_map(Result::ok)
                .any(|e| e.file_name() != ".git");
            if leftover {
                return Err(WorktreeError::Git {
                    args: vec!["worktree".into(), "remove".into(), path.to_string_lossy().into()],
                    stderr: "directory is no longer a registered git worktree and holds files git can't account for; use --force".into(),
                });
            }
        }
        std::fs::remove_dir_all(path)?;
        return Ok(());
    }
    let path_str = path.to_string_lossy().into_owned();
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path_str);
    run_git(repo, &args).await.map(|_| ())
}

/// Whether git's worktree list for `repo` includes `path` (compared canonicalised,
/// since git prints resolved paths).
async fn is_registered(repo: &Path, path: &Path) -> Result<bool, WorktreeError> {
    let want = path.canonicalize()?;
    let out = run_git(repo, &["worktree", "list", "--porcelain"]).await?;
    Ok(out
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .any(|p| Path::new(p).canonicalize().is_ok_and(|p| p == want)))
}

/// Deletes `branch` from `repo` if it exists (`-D`); a branch that is already gone is fine.
pub async fn delete_branch_if_exists(repo: &Path, branch: &str) -> Result<(), WorktreeError> {
    if branch_exists(repo, branch).await? {
        delete_branch(repo, branch, true).await?;
    }
    Ok(())
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
            (Some(command), Some(pid), Some(name)) => {
                format!("{command} (pid {pid}) has {name} open")
            }
            _ => line.trim().to_string(),
        }
    }))
}

/// Deletes a branch. `force` matches `git branch -D` vs `-d`.
pub async fn delete_branch(repo: &Path, branch: &str, force: bool) -> Result<(), WorktreeError> {
    let flag = if force { "-D" } else { "-d" };
    run_git(repo, &["branch", flag, branch]).await.map(|_| ())
}

/// Paths changed by `commit` (against its first parent, for a merge).
pub async fn changed_files(repo: &Path, commit: &str) -> Result<Vec<String>, WorktreeError> {
    let out = run_git(
        repo,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-m",
            "--first-parent",
            commit,
        ],
    )
    .await?;
    Ok(out.lines().map(str::to_string).collect())
}

/// Whether `commit` names a commit reachable from the repo's `HEAD`, i.e. it
/// is on the integration branch. An unknown revision is `false`, not an error.
pub async fn is_on_head(repo: &Path, commit: &str) -> Result<bool, WorktreeError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "merge-base",
            "--is-ancestor",
            &format!("{commit}^{{commit}}"),
            "HEAD",
        ])
        .output()
        .await?;
    Ok(out.status.success())
}

/// The result of `git merge-tree --write-tree` of `a` and `b`: touches no working tree
/// or branch (it writes loose objects only).
pub enum MergeProbe {
    Clean,
    Conflicts(Vec<String>),
    /// git older than 2.38.
    Unsupported,
}

pub async fn merge_probe(repo: &Path, a: &str, b: &str) -> Result<MergeProbe, WorktreeError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge-tree", "--write-tree", "--name-only", "--no-messages"])
        .args([a, b])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    match out.status.code() {
        Some(0) => Ok(MergeProbe::Clean),
        // Exit 1: the first line is the tree id, then one path per conflicted file.
        Some(1) if stdout.lines().count() > 1 => {
            let mut paths: Vec<String> = stdout
                .lines()
                .skip(1)
                .take_while(|l| !l.is_empty())
                .map(str::to_string)
                .collect();
            paths.dedup();
            Ok(MergeProbe::Conflicts(paths))
        }
        // Older git rejects the options as a usage error.
        Some(129) => Ok(MergeProbe::Unsupported),
        _ => Err(WorktreeError::Git {
            args: vec!["merge-tree".into(), a.into(), b.into()],
            stderr: stderr.trim().to_string(),
        }),
    }
}

/// Drop git's records of worktrees whose directories no longer exist.
pub async fn prune(repo: &Path) -> Result<(), WorktreeError> {
    run_git(repo, &["worktree", "prune"]).await.map(|_| ())
}

/// Whether `branch` exists and is fully merged into the repo's `HEAD`, i.e.
/// whether it has landed. A missing branch counts as merged. A branch still at
/// the commit it was created on has no work of its own, so it isn't merged (the
/// creation commit comes from the branch's reflog; with no reflog we can't tell
/// and fall back to ancestry alone). A branch landed by
/// `git merge --squash` isn't an ancestor, so a `Branch: <branch>` trailer on a
/// commit reachable from `HEAD` (the manager's landing commit) counts too,
/// unless the branch has commits newer than that landing (a reused branch).
pub async fn is_merged(repo: &Path, branch: &str) -> Result<bool, WorktreeError> {
    let exists = branch_exists(repo, branch).await?;
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
    if status.success() {
        return has_own_commits(repo, branch).await;
    }
    let landed = run_git(
        repo,
        &[
            "log",
            "-1",
            "--format=%ct",
            "--fixed-strings",
            &format!("--grep=Branch: {branch}"),
            "HEAD",
        ],
    )
    .await?;
    let Ok(landed_at) = landed.trim().parse::<i64>() else {
        return Ok(false);
    };
    // A reused branch keeps its old trailer on HEAD; commits made after that
    // landing are unlanded work.
    let ahead = run_git(
        repo,
        &["log", "--format=%ct", &format!("HEAD..refs/heads/{branch}")],
    )
    .await?;
    let newest = ahead.lines().filter_map(|l| l.parse::<i64>().ok()).max();
    Ok(newest.is_none_or(|t| t <= landed_at))
}

/// False only when the reflog shows `branch` still at the commit it was created on.
async fn has_own_commits(repo: &Path, branch: &str) -> Result<bool, WorktreeError> {
    let tip = run_git(repo, &["rev-parse", &format!("refs/heads/{branch}")]).await?;
    let log = run_git(
        repo,
        &[
            "reflog",
            "show",
            "--format=%H",
            &format!("refs/heads/{branch}"),
        ],
    )
    .await
    .unwrap_or_default();
    Ok(log.lines().last().is_none_or(|first| first != tip.trim()))
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

/// Agent names that would collide with a branch bridle itself owns, not an
/// agent's: `bridle/state` is the state branch (state_branch.rs), so an
/// agent named "state" would fight it for `refs/heads/bridle/state`.
const RESERVED_AGENT_NAMES: [&str; 1] = ["state"];

/// Agent names double as worktree directory names and branch suffixes, so
/// they're restricted to what's safe in both: lowercase alnum and hyphens,
/// starting with an alnum, at most 40 characters, and not a name reserved
/// for one of bridle's own branches ([`RESERVED_AGENT_NAMES`]).
pub fn validate_agent_name(name: &str) -> Result<(), WorktreeError> {
    let bytes = name.as_bytes();
    let first_ok = bytes
        .first()
        .is_some_and(|&b| b.is_ascii_lowercase() || b.is_ascii_digit());
    let len_ok = !bytes.is_empty() && bytes.len() <= 40;
    let rest_ok = bytes
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    let not_reserved = !RESERVED_AGENT_NAMES.contains(&name);
    if first_ok && len_ok && rest_ok && not_reserved {
        Ok(())
    } else {
        Err(WorktreeError::InvalidName(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn warm_target_clones_when_present_and_tolerates_absence() {
        let tmp = tempfile::tempdir().expect("tmp");
        let (repo, wt) = (tmp.path().join("repo"), tmp.path().join("wt"));
        std::fs::create_dir_all(&wt).expect("mk");
        std::fs::create_dir_all(&repo).expect("mk");
        // No target/ in the clone: no-op, no panic.
        let integ = tmp.path().join("integration");
        warm_target(&repo, &integ, &wt).await;
        assert!(!wt.join("target").exists());

        std::fs::create_dir_all(repo.join("target/debug")).expect("mk");
        std::fs::write(repo.join("target/debug/dep"), "x").expect("w");
        warm_target(&repo, &integ, &wt).await;
        assert_eq!(
            wt.join("target/debug/dep").exists(),
            cfg!(target_os = "macos")
        );
        assert!(repo.join("target/debug/dep").exists());
    }

    #[test]
    fn warm_source_prefers_fresh_integration_target() {
        let tmp = tempfile::tempdir().expect("tmp");
        let (repo, integ) = (tmp.path().join("repo"), tmp.path().join("integration"));
        std::fs::create_dir_all(repo.join("target")).expect("mk");
        // No integration target: the clone's.
        assert_eq!(warm_source(&repo, &integ, false), repo.join("target"));
        std::fs::create_dir_all(integ.join("target")).expect("mk");
        // Idle build: the integration target.
        assert_eq!(warm_source(&repo, &integ, false), integ.join("target"));
        // Building and not newer than the clone's: the clone's.
        let old = std::time::SystemTime::now() - Duration::from_secs(3600);
        std::fs::File::open(integ.join("target"))
            .and_then(|f| f.set_modified(old))
            .expect("age");
        assert_eq!(warm_source(&repo, &integ, true), repo.join("target"));
        // Building but newer than the clone's: still the integration target.
        std::fs::File::open(repo.join("target"))
            .and_then(|f| f.set_modified(old - Duration::from_secs(3600)))
            .expect("age");
        assert_eq!(warm_source(&repo, &integ, true), integ.join("target"));
    }

    #[test]
    fn copy_files_copies_nested_keeps_mode_skips_missing_and_unsafe() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().expect("tmp");
        let (repo, wt) = (tmp.path().join("repo"), tmp.path().join("wt"));
        std::fs::create_dir_all(repo.join("a")).expect("mk");
        std::fs::create_dir_all(&wt).expect("mk");
        std::fs::write(repo.join(".env"), "T=1").expect("w");
        std::fs::set_permissions(repo.join(".env"), std::fs::Permissions::from_mode(0o600))
            .expect("chmod");
        std::fs::write(repo.join("a/b.json"), "{}").expect("w");
        std::fs::write(tmp.path().join("outside"), "x").expect("w");
        let files: Vec<String> = [".env", "missing", "a/b.json", "../outside"]
            .map(String::from)
            .into();
        copy_files(&repo, &wt, &files);
        assert_eq!(std::fs::read_to_string(wt.join(".env")).expect("r"), "T=1");
        let mode = std::fs::metadata(wt.join(".env"))
            .expect("m")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        assert_eq!(
            std::fs::read_to_string(wt.join("a/b.json")).expect("r"),
            "{}"
        );
        assert!(!wt.join("missing").exists());
        assert!(!wt.join("outside").exists());
    }

    #[tokio::test]
    async fn run_setup_cwd_env_failure_and_timeout() {
        let tmp = tempfile::tempdir().expect("tmp");
        let wt = tmp.path();
        let long = Duration::from_secs(30);
        // set_var is unsafe (forbidden), so this only proves the filter where the test runner
        // inherits BRIDLE_TOKEN, as it does when run by a bridle agent.
        run_setup(
            wt,
            "pwd -P > marker; echo \"t=${BRIDLE_TOKEN-unset}\" >> marker",
            long,
        )
        .await
        .expect("ok");
        let marker = std::fs::read_to_string(wt.join("marker")).expect("marker");
        assert!(marker.starts_with(wt.canonicalize().expect("canon").to_str().expect("utf8")));
        assert!(marker.contains("t=unset"));

        let err = run_setup(wt, "seq 1 30; echo boom >&2; exit 3", long)
            .await
            .expect_err("fails")
            .to_string();
        assert!(
            err.contains("seq 1 30") && err.contains("exit status: 3"),
            "{err}"
        );
        assert!(
            err.contains("boom") && err.contains("\n30") && !err.contains("\n5\n"),
            "{err}"
        );

        let err = run_setup(wt, "sleep 30", Duration::from_millis(100))
            .await
            .expect_err("times out")
            .to_string();
        assert!(
            err.contains("sleep 30") && err.contains("timed out"),
            "{err}"
        );
    }

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
        run(&["init", "-q", "-b", "main"]).await;
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
    async fn remove_handles_a_worktree_git_no_longer_lists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        init_repo(&repo).await;

        // Record pruned, directory (with the checkout) left.
        let wt_path = tmp.path().join("wt").join("w1");
        add(&repo, &wt_path, "bridle/w1", "HEAD")
            .await
            .expect("add");
        std::fs::remove_dir_all(repo.join(".git/worktrees")).expect("drop record");
        assert!(wt_path.exists());

        // Files git can't account for: refused without force.
        std::fs::write(wt_path.join("scratch.txt"), "x").expect("scratch");
        assert!(remove(&repo, &wt_path, false).await.is_err());
        assert!(wt_path.exists());
        remove(&repo, &wt_path, true).await.expect("force remove");
        assert!(!wt_path.exists());

        // Only the .git pointer left: removed without force.
        let w2 = tmp.path().join("wt").join("w2");
        std::fs::create_dir_all(&w2).expect("mkdir");
        std::fs::write(w2.join(".git"), "gitdir: /nowhere").expect("pointer");
        remove(&repo, &w2, false)
            .await
            .expect("remove bare pointer");
        assert!(!w2.exists());

        // A branch that is already gone doesn't error; an existing one is deleted.
        delete_branch_if_exists(&repo, "bridle/missing")
            .await
            .expect("missing branch");
        delete_branch_if_exists(&repo, "bridle/w1")
            .await
            .expect("existing branch");
        assert!(!branch_exists(&repo, "bridle/w1").await.expect("exists"));
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
        assert!(!is_merged(&repo, "bridle/w2").await.expect("empty branch"));
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

        // A squash landing leaves the branch un-merged in git's eyes; the
        // trailer on the landing commit is what marks it landed.
        let git = |args: &[&str]| {
            let mut c = std::process::Command::new("git");
            c.arg("-C")
                .arg(&repo)
                .args(["-c", "user.email=t@e.com", "-c", "user.name=T"])
                .args(args);
            c
        };
        assert!(
            git(&["merge", "--squash", "bridle/w2"])
                .output()
                .expect("squash")
                .status
                .success()
        );
        assert!(
            !is_merged(&repo, "bridle/w2")
                .await
                .expect("not yet committed")
        );
        assert!(
            git(&[
                "commit",
                "--allow-empty",
                "-q",
                "-m",
                "t: x\n\nBranch: bridle/w2"
            ])
            .output()
            .expect("land")
            .status
            .success()
        );
        assert!(is_merged(&repo, "bridle/w2").await.expect("squash-landed"));

        // Reusing the branch after the landing: the old trailer must not hide the new commit.
        let later = Command::new("git")
            .arg("-C")
            .arg(&wt_path)
            .args(["-c", "user.email=t@e.com", "-c", "user.name=T"])
            .args(["commit", "--allow-empty", "-q", "-m", "more work"])
            .env("GIT_COMMITTER_DATE", "@4102444800 +0000")
            .output()
            .await
            .expect("later commit");
        assert!(later.status.success());
        assert!(
            !is_merged(&repo, "bridle/w2")
                .await
                .expect("reused after landing")
        );

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

    #[test]
    fn agent_name_state_is_reserved() {
        // "state" would collide with the `bridle/state` branch
        // (state_branch.rs): git can't have both `refs/heads/bridle/state`
        // and an agent worktree branch of the same name.
        let err = validate_agent_name("state").expect_err("state should be reserved");
        assert!(matches!(err, WorktreeError::InvalidName(n) if n == "state"));
    }

    fn git_in(dir: &Path, args: &[&str]) {
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
    async fn merge_probe_reports_clean_and_conflicting_branches() {
        let tmp = tempfile::tempdir().expect("tmp");
        let repo = tmp.path();
        git_in(repo, &["init", "-q", "-b", "main"]);
        std::fs::write(repo.join("a.txt"), "one\n").expect("w");
        std::fs::write(repo.join("b.txt"), "one\n").expect("w");
        git_in(repo, &["add", "."]);
        git_in(repo, &["commit", "-qm", "base"]);
        for (branch, file, text) in [("clean", "b.txt", "two\n"), ("clash", "a.txt", "three\n")] {
            git_in(repo, &["checkout", "-q", "-b", branch, "main"]);
            std::fs::write(repo.join(file), text).expect("w");
            git_in(repo, &["commit", "-qam", branch]);
        }
        git_in(repo, &["checkout", "-q", "main"]);
        std::fs::write(repo.join("a.txt"), "main\n").expect("w");
        git_in(repo, &["commit", "-qam", "main moves"]);

        assert!(matches!(
            merge_probe(repo, "main", "clean").await.expect("probe"),
            MergeProbe::Clean
        ));
        match merge_probe(repo, "main", "clash").await.expect("probe") {
            MergeProbe::Conflicts(p) => assert_eq!(p, vec!["a.txt".to_string()]),
            _ => panic!("expected a conflict"),
        }
        assert!(merge_probe(repo, "main", "nope").await.is_err());
        // The probe leaves the working tree and HEAD alone.
        assert_eq!(
            std::fs::read_to_string(repo.join("a.txt")).expect("r"),
            "main\n"
        );
    }
}
