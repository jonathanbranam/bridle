//! `bridle land`: merge a task's branch into the integration branch in a dedicated worktree,
//! run the project's check there, and only then move the integration branch's ref
//! (docs/design/agent-host/roles-and-config.md, "The integrator"). Never pushes.

use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::worktree::{self, MergeProbe, WorktreeError, run_git};

/// Where in the branch's diff an `arch-revision` is required.
const ARCH_PREFIX: &str = "design/architecture/";

/// Lines of check output reported on failure.
const TAIL_LINES: usize = 40;

#[derive(Debug, thiserror::Error)]
pub enum LandError {
    #[error("{0}")]
    Refused(String),
    #[error("merge conflict in: {}", .0.join(", "))]
    Conflict(Vec<String>),
    #[error("check failed ({cmd}); last output:\n{tail}")]
    CheckFailed { cmd: String, tail: String },
    #[error("{0} moved, retry")]
    Moved(String),
    #[error(transparent)]
    Git(#[from] WorktreeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct LandInput<'a> {
    pub repo: &'a Path,
    /// `<workspace>/integration`.
    pub dir: PathBuf,
    pub integration: &'a str,
    pub branch: &'a str,
    pub task: &'a str,
    pub is_arch_revision: bool,
    pub check: Option<&'a str>,
}

pub struct Landed {
    /// The merge commit the integration branch now points at.
    pub commit: String,
    pub notes: Vec<String>,
}

/// The caller holds the daemon's one-landing-at-a-time lock.
pub async fn land(i: &LandInput<'_>) -> Result<Landed, LandError> {
    let mut notes = Vec::new();
    let old = run_git(
        i.repo,
        &["rev-parse", &format!("refs/heads/{}", i.integration)],
    )
    .await?
    .trim()
    .to_string();

    if !i.is_arch_revision {
        let touched = run_git(
            i.repo,
            &["diff", "--name-only", &format!("{old}...{}", i.branch)],
        )
        .await?;
        if let Some(p) = touched.lines().find(|p| p.starts_with(ARCH_PREFIX)) {
            return Err(LandError::Refused(format!(
                "{} touches {p}; only an arch-revision task may change design/architecture/**",
                i.branch
            )));
        }
    }

    match worktree::merge_probe(i.repo, &old, i.branch).await? {
        MergeProbe::Conflicts(paths) => return Err(LandError::Conflict(paths)),
        MergeProbe::Clean => {}
        MergeProbe::Unsupported => notes.push("git < 2.38: skipped the merge probe".to_string()),
    }

    let wt = prepare_worktree(i, &old).await?;
    let msg = format!("Merge branch '{}' ({})", i.branch, i.task);
    if let Err(e) = run_git(&wt, &["merge", "--no-ff", "-m", &msg, i.branch]).await {
        let paths = run_git(&wt, &["diff", "--name-only", "--diff-filter=U"])
            .await
            .unwrap_or_default();
        let _ = run_git(&wt, &["merge", "--abort"]).await;
        let paths: Vec<String> = paths.lines().map(str::to_string).collect();
        return Err(if paths.is_empty() {
            LandError::Git(e)
        } else {
            LandError::Conflict(paths)
        });
    }

    match i.check {
        Some(cmd) => run_check(&wt, cmd).await?,
        None => notes.push("no [integration] check configured: skipped".to_string()),
    }

    let new = run_git(&wt, &["rev-parse", "HEAD"])
        .await?
        .trim()
        .to_string();
    // The old value is the guard: a moved integration branch is refused, not overwritten.
    run_git(
        i.repo,
        &[
            "update-ref",
            &format!("refs/heads/{}", i.integration),
            &new,
            &old,
        ],
    )
    .await
    .map_err(|_| LandError::Moved(i.integration.to_string()))?;
    Ok(Landed { commit: new, notes })
}

/// The integration worktree, created on first use, reset onto a scratch branch at `tip`.
async fn prepare_worktree(i: &LandInput<'_>, tip: &str) -> Result<PathBuf, WorktreeError> {
    let scratch = format!("integrate/{}", i.task);
    if i.dir.join(".git").exists() {
        // A previous landing may have died mid-merge.
        let _ = run_git(&i.dir, &["merge", "--abort"]).await;
        run_git(&i.dir, &["checkout", "-f", "-B", &scratch, tip]).await?;
        run_git(&i.dir, &["clean", "-fdq"]).await?;
    } else {
        worktree::prune(i.repo).await?;
        let dir = i.dir.to_string_lossy();
        run_git(
            i.repo,
            &["worktree", "add", "-f", "-B", &scratch, &dir, tip],
        )
        .await?;
    }
    Ok(i.dir.clone())
}

async fn run_check(wt: &Path, cmd: &str) -> Result<(), LandError> {
    let out = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(wt)
        .output()
        .await?;
    if out.status.success() {
        return Ok(());
    }
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let lines: Vec<&str> = text.lines().collect();
    let tail = lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n");
    Err(LandError::CheckFailed {
        cmd: cmd.to_string(),
        tail,
    })
}
