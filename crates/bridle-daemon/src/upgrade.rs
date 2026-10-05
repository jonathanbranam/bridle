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
    /// Stand-in pre-flight (program then args). With `build` set and this not, none runs, so a
    /// test's stand-in build never execs the test binary.
    pub preflight: Option<Vec<String>>,
}

impl fmt::Debug for UpgradeHooks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpgradeHooks")
            .field("gh", &self.gh.is_some())
            .field("build", &self.build)
            .field("preflight", &self.preflight)
            .finish()
    }
}

#[derive(Clone)]
pub struct Upgrader {
    gh: Arc<dyn Gh>,
    build: Vec<String>,
    /// Sign the installed binary after the build: only for the real build, never a test's stand-in.
    sign: bool,
    preflight: Preflight,
    /// One upgrade at a time: it builds for minutes before it restarts.
    busy: Arc<AtomicBool>,
    /// The last commit an upgrade failed on (memory only), so the automatic trigger skips it.
    failed: Arc<Mutex<Option<String>>>,
    /// The commit the automatic upgrade first found no quiet point for, and when (memory only):
    /// it retries silently until this is [`GIVE_UP_AFTER`] old.
    waiting: Arc<Mutex<Option<(String, std::time::Instant)>>>,
}

/// How long the automatic upgrade keeps retrying a commit that never finds a quiet point.
pub const GIVE_UP_AFTER: Duration = Duration::from_secs(3 * 60 * 60);

/// How a freshly built binary is checked before the daemon execs it.
#[derive(Clone)]
enum Preflight {
    /// `<installed bridle> serve --check` with the daemon's own repo and workspace.
    Real,
    Command(Vec<String>),
    Skip,
}

impl Upgrader {
    pub fn new(
        gh: Arc<dyn Gh>,
        build: Option<Vec<String>>,
        preflight: Option<Vec<String>>,
    ) -> Self {
        let preflight = match (preflight, build.is_some()) {
            (Some(c), _) => Preflight::Command(c),
            (None, true) => Preflight::Skip,
            (None, false) => Preflight::Real,
        };
        let sign = build.is_none();
        let build = build.unwrap_or_else(|| {
            ["cargo", "install", "--path", "crates/bridle"]
                .map(String::from)
                .to_vec()
        });
        Upgrader {
            gh,
            build,
            sign,
            preflight,
            busy: Default::default(),
            failed: Default::default(),
            waiting: Default::default(),
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

    /// Notes a no-quiet-point give-up on `sha`; how long it has been giving up on it.
    pub fn note_waiting(&self, sha: &str) -> Duration {
        let mut w = self.waiting.lock().expect("waiting lock");
        match &*w {
            Some((s, since)) if s == sha => since.elapsed(),
            _ => {
                *w = Some((sha.to_string(), std::time::Instant::now()));
                Duration::ZERO
            }
        }
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
        // The build replaces the installed binary; keep the running one to roll back to.
        let result = match crate::exe_path()
            .map_err(anyhow::Error::from)
            .and_then(|exe| crate::rollback::stash_previous(ws, &exe))
        {
            Ok(()) => self.run_build(ws, &dir).await,
            Err(e) => Err(format!("keeping the previous binary: {e:#}")),
        };
        let _ = worktree::remove(&ws.repo, &dir, true).await;
        result
    }

    /// Runs the new binary's self-check. `Err` means the daemon must stay as it is.
    pub async fn check_built(&self, ws: &Workspace) -> Result<(), String> {
        let (program, args): (String, Vec<String>) = match &self.preflight {
            Preflight::Skip => return Ok(()),
            Preflight::Command(c) => {
                let (p, a) = c.split_first().ok_or("empty preflight command")?;
                (p.clone(), a.to_vec())
            }
            Preflight::Real => (
                crate::exe_path()
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .into_owned(),
                vec![
                    "serve".into(),
                    "--check".into(),
                    "--repo".into(),
                    ws.repo.to_string_lossy().into_owned(),
                    "--workspace".into(),
                    ws.workspace.to_string_lossy().into_owned(),
                ],
            ),
        };
        let out = tokio::time::timeout(
            Duration::from_secs(60),
            Command::new(&program)
                .args(&args)
                .kill_on_drop(true)
                .output(),
        )
        .await
        .map_err(|_| "the new binary's self-check timed out".to_string())?
        .map_err(|e| format!("running the new binary {program}: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let lines: Vec<&str> = stderr.lines().collect();
        let tail = lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n");
        Err(format!(
            "the new binary's self-check failed ({}):\n{tail}",
            out.status
        ))
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
            // A failed sign leaves the working ad-hoc build in place; don't fail the upgrade.
            if self.sign
                && let Err(e) = crate::exe_path()
                    .map_err(anyhow::Error::from)
                    .and_then(|exe| crate::signing::sign(&exe))
            {
                tracing::warn!("signing the upgraded binary: {e:#}");
            }
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

/// Paths the binary is built from. The `include_*!` uses in the crates all sit in tests, so
/// nothing under `workflow/` or `docs/` reaches the binary; if one is added outside tests, add
/// its path here.
fn feeds_binary(path: &str) -> bool {
    path.starts_with("crates/")
        || path.starts_with(".cargo/")
        || matches!(
            path,
            "Cargo.toml" | "Cargo.lock" | "rust-toolchain" | "rust-toolchain.toml"
        )
}

/// The commit the last upgrade built, if any.
pub async fn built(store: &Store) -> Option<String> {
    store
        .get_meta(BUILT_KEY)
        .await
        .ok()
        .flatten()
        .filter(|b| !b.is_empty())
}

/// Whether the diff from the last built commit to `sha` changes anything the binary is built
/// from. `true` (build) whenever that can't be told: nothing built yet, or git can't diff.
pub async fn needs_build(store: &Store, repo: &Path, sha: &str) -> bool {
    let Some(built) = built(store).await else {
        return true;
    };
    match worktree::run_git(repo, &["diff", "--name-only", &format!("{built}..{sha}")]).await {
        Ok(out) => out.lines().any(feeds_binary),
        Err(_) => true,
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn git(repo: &Path, args: &[&str]) -> String {
        let out = tokio::process::Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .current_dir(repo)
            .output()
            .await
            .expect("git");
        assert!(out.status.success(), "git {args:?}");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    async fn commit(repo: &Path, file: &str) -> String {
        let p = repo.join(file);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(&p, file).expect("write");
        git(repo, &["add", "-A"]).await;
        git(repo, &["commit", "-m", file]).await;
        git(repo, &["rev-parse", "HEAD"]).await
    }

    #[tokio::test]
    async fn only_files_the_binary_is_built_from_need_a_build() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).expect("mkdir");
        git(&repo, &["init", "-q"]).await;
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("store");
        let base = commit(&repo, "crates/a/src/lib.rs").await;
        // Nothing built yet: build.
        assert!(needs_build(&store, &repo, &base).await);
        record_built(&store, &base).await;

        let docs = commit(&repo, "docs/tickets/open/x.md").await;
        let flow = commit(&repo, "workflow/base/rules/x.md").await;
        assert!(!needs_build(&store, &repo, &docs).await);
        assert!(!needs_build(&store, &repo, &flow).await);

        for (i, file) in ["crates/a/src/main.rs", "Cargo.lock", "Cargo.toml"]
            .into_iter()
            .enumerate()
        {
            let sha = commit(&repo, file).await;
            assert!(needs_build(&store, &repo, &sha).await, "{file} (#{i})");
            // A mixed range still builds: the docs commit before it doesn't hide it.
            record_built(&store, &docs).await;
            assert!(needs_build(&store, &repo, &sha).await);
            record_built(&store, &base).await;
        }
    }
}
