//! `bridle restart --upgrade` (docs/design/agent-host/daemon.md, "Upgrade"): find the newest commit
//! on the integration branch whose GitHub Actions runs are all green, build it, then the caller
//! restarts in place. The CI lookup is the CI watcher's [`Gh`]; the build command is injectable
//! so tests never run a real `cargo install`.

use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::process::Command;

use crate::ci::Gh;
use crate::paths::Workspace;
use crate::store::Store;
use crate::worktree;

/// `meta` key: the commit the last upgrade built and restarted into. The binary carries no commit
/// of its own, so this is what "newer than the running binary" is measured against; a daemon
/// that has never upgraded builds the newest green commit.
const BUILT_KEY: &str = "upgrade.built";
/// How far back from the tip to look for a green commit.
const LOOKBACK: usize = 30;
const BUILD_TIMEOUT: Duration = Duration::from_secs(60 * 60);
/// Lines of build output kept in a failure report.
const TAIL_LINES: usize = 15;

/// Test hooks: canned CI status and a stand-in build command (program then args, run in the
/// checkout of the commit). `None` means the real `gh` and `cargo install --path crates/bridle`.
#[derive(Clone, Default)]
pub struct UpgradeHooks {
    pub gh: Option<Arc<dyn Gh>>,
    pub build: Option<Vec<String>>,
}

impl fmt::Debug for UpgradeHooks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpgradeHooks")
            .field("gh", &self.gh.is_some())
            .field("build", &self.build)
            .finish()
    }
}

#[derive(Clone)]
pub struct Upgrader {
    gh: Arc<dyn Gh>,
    build: Vec<String>,
    /// One upgrade at a time: it builds for minutes before it restarts.
    busy: Arc<AtomicBool>,
    /// The last commit an upgrade failed on (memory only), so the automatic trigger skips it.
    failed: Arc<Mutex<Option<String>>>,
}

impl Upgrader {
    pub fn new(gh: Arc<dyn Gh>, build: Option<Vec<String>>) -> Self {
        let build = build.unwrap_or_else(|| {
            ["cargo", "install", "--path", "crates/bridle"]
                .map(String::from)
                .to_vec()
        });
        Upgrader {
            gh,
            build,
            busy: Default::default(),
            failed: Default::default(),
        }
    }

    /// Claims the single upgrade slot; `false` if one is under way. Release with [`Self::release`].
    pub fn claim(&self) -> bool {
        !self.busy.swap(true, Ordering::SeqCst)
    }

    pub fn release(&self) {
        self.busy.store(false, Ordering::SeqCst);
    }

    pub fn note_failed(&self, sha: &str) {
        *self.failed.lock().expect("failed-sha lock") = Some(sha.to_string());
    }

    pub fn failed_before(&self, sha: &str) -> bool {
        self.failed.lock().expect("failed-sha lock").as_deref() == Some(sha)
    }

    /// The newest commit on `integration` (first-parent, at most [`LOOKBACK`] back) whose runs
    /// have all finished green. `Ok(None)` when nothing newer than the last upgrade qualifies.
    pub async fn newest_green(
        &self,
        store: &Store,
        repo: &Path,
        integration: &str,
    ) -> Result<Option<String>, String> {
        let log = worktree::run_git(
            repo,
            &[
                "rev-list",
                "--first-parent",
                "-n",
                &LOOKBACK.to_string(),
                &format!("refs/heads/{integration}"),
            ],
        )
        .await
        .map_err(|e| e.to_string())?;
        let built = store.get_meta(BUILT_KEY).await.ok().flatten();
        for sha in log.split_whitespace() {
            if let Some(built) = built.as_deref().filter(|b| !b.is_empty())
                && (built == sha || is_ancestor(repo, sha, built).await)
            {
                return Ok(None);
            }
            let gh = self.gh.clone();
            let s = sha.to_string();
            let runs = tokio::task::spawn_blocking(move || gh.runs(&s))
                .await
                .map_err(|e| e.to_string())??;
            if !runs.is_empty()
                && runs
                    .iter()
                    .all(|r| r.status == "completed" && r.conclusion == "success")
            {
                return Ok(Some(sha.to_string()));
            }
        }
        Ok(None)
    }

    /// Builds `sha` in a throwaway detached worktree (the human's checkout is never touched),
    /// with a target dir of its own kept between upgrades so builds are incremental.
    pub async fn build(&self, ws: &Workspace, sha: &str) -> Result<(), String> {
        let dir = ws.state_dir().join("upgrade-src");
        let _ = worktree::remove(&ws.repo, &dir, true).await;
        let _ = std::fs::remove_dir_all(&dir);
        let _ = worktree::prune(&ws.repo).await;
        let dir_str = dir.to_string_lossy().into_owned();
        worktree::run_git(&ws.repo, &["worktree", "add", "--detach", &dir_str, sha])
            .await
            .map_err(|e| format!("checking out {sha}: {e}"))?;
        let result = self.run_build(ws, &dir).await;
        let _ = worktree::remove(&ws.repo, &dir, true).await;
        result
    }

    async fn run_build(&self, ws: &Workspace, dir: &Path) -> Result<(), String> {
        let (program, args) = self.build.split_first().ok_or("empty build command")?;
        let child = Command::new(program)
            .args(args)
            .current_dir(dir)
            .env("CARGO_TARGET_DIR", ws.state_dir().join("upgrade-target"))
            .kill_on_drop(true)
            .output();
        let out = tokio::time::timeout(BUILD_TIMEOUT, child)
            .await
            .map_err(|_| "the build timed out after an hour".to_string())?
            .map_err(|e| format!("running {program}: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let lines: Vec<&str> = stderr.lines().collect();
        let tail = lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n");
        Err(format!(
            "{} failed ({}):\n{tail}",
            self.build.join(" "),
            out.status
        ))
    }
}

/// Records the commit the daemon is about to restart into.
pub async fn record_built(store: &Store, sha: &str) {
    let _ = store.swap_meta(BUILT_KEY, sha).await;
}

async fn is_ancestor(repo: &Path, ancestor: &str, of: &str) -> bool {
    worktree::run_git(repo, &["merge-base", "--is-ancestor", ancestor, of])
        .await
        .is_ok()
}
