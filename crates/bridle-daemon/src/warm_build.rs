//! The background warm build: after each land, one incremental build in the integration
//! worktree keeps its `target/` fresh, so `worktree::warm_target` copies a current cache
//! (ticket b7cz). Off land's critical path, niced, one at a time; a trigger during a build
//! queues exactly one more. Failures are logged and never fail a land.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::process::Command;

/// Whether a warm build is running. Process-wide so `warm_target` can read it without the
/// supervisor holding a handle (one daemon per process).
static BUILDING: AtomicBool = AtomicBool::new(false);

pub fn building() -> bool {
    BUILDING.load(Ordering::SeqCst)
}

/// `target/debug` subdirectories that grow without bound as dependency hashes change.
const GROWING: [&str; 3] = ["deps", "build", "incremental"];

/// Entries in `target/debug/deps` past which the warm source is pruned after a build, and
/// `worktree::warm_target` refuses to clone it (a clone of a bloated target stalls the spawn and
/// feeds syspolicyd one assessment per binary; ticket j4bk). A healthy target has ~12K.
pub const BLOAT_ENTRIES: usize = 100_000;

/// Entries in `target/debug/deps` past which `prune_after_land` prunes. Lower than
/// `BLOAT_ENTRIES` so the folder stays near a clean build (~12K), not just under the clone limit
/// (ticket mhfx).
pub const PRUNE_ENTRIES: usize = 30_000;

/// How many entries `target/debug/deps` holds (flat, so cheap to count).
pub fn deps_entries(target: &std::path::Path) -> usize {
    std::fs::read_dir(target.join("debug/deps")).map_or(0, |d| d.count())
}

/// Removes entries of `target/debug/{deps,build,incremental}` last modified before `cutoff`,
/// when `deps` holds more than `max_entries`. Run with the start of a build as the cutoff, so
/// what that build produced survives; what it merely reused is rebuilt next time, a one-off cost
/// that keeps the directory bounded. Returns how many entries were removed.
pub fn prune_stale(
    target: &std::path::Path,
    cutoff: std::time::SystemTime,
    max_entries: usize,
) -> usize {
    if deps_entries(target) <= max_entries {
        return 0;
    }
    let mut removed = 0;
    for sub in GROWING {
        let Ok(rd) = std::fs::read_dir(target.join("debug").join(sub)) else {
            continue;
        };
        for e in rd.flatten() {
            let stale = e
                .metadata()
                .and_then(|m| m.modified())
                .is_ok_and(|m| m < cutoff);
            if !stale {
                continue;
            }
            let p = e.path();
            let ok = if p.is_dir() {
                std::fs::remove_dir_all(&p)
            } else {
                std::fs::remove_file(&p)
            };
            removed += usize::from(ok.is_ok());
        }
    }
    removed
}

#[derive(Default)]
struct Phase {
    running: bool,
    queued: bool,
}

#[derive(Clone)]
pub struct WarmBuild {
    cmd: Option<Arc<str>>,
    dir: PathBuf,
    phase: Arc<Mutex<Phase>>,
}

impl WarmBuild {
    /// `cmd` is `[integration] warm_build`; `None` makes `trigger` a no-op.
    pub fn new(cmd: Option<String>, dir: PathBuf) -> Self {
        Self {
            cmd: cmd.map(Arc::from),
            dir,
            phase: Default::default(),
        }
    }

    /// Starts a build, or queues one more if one is running. Returns at once.
    pub fn trigger(&self) {
        let Some(cmd) = self.cmd.clone() else { return };
        {
            let mut p = self.phase.lock().expect("warm build phase lock");
            if p.running {
                p.queued = true;
                return;
            }
            p.running = true;
        }
        let (dir, phase) = (self.dir.clone(), self.phase.clone());
        tokio::spawn(async move {
            loop {
                BUILDING.store(true, Ordering::SeqCst);
                run(&cmd, &dir).await;
                let mut p = phase.lock().expect("warm build phase lock");
                if p.queued {
                    p.queued = false;
                    continue;
                }
                p.running = false;
                BUILDING.store(false, Ordering::SeqCst);
                break;
            }
        });
    }

    /// Prunes the integration `target/` of everything older than now, independent of whether a
    /// warm build is configured (it was off in bridle's own config, so the prune after a warm
    /// build never ran and the folder reached 90 GB; ticket mhfx). The caller holds the landing
    /// lock, so no check is running in the integration worktree.
    pub async fn prune_after_land(&self) {
        let target = self.dir.join("target");
        let removed = tokio::task::spawn_blocking(move || {
            prune_stale(&target, std::time::SystemTime::now(), PRUNE_ENTRIES)
        })
        .await
        .unwrap_or(0);
        if removed > 0 {
            tracing::info!(removed, "integration target/ pruned after land");
        }
    }

    #[cfg(test)]
    fn idle(&self) -> bool {
        !self.phase.lock().expect("warm build phase lock").running
    }
}

async fn run(cmd: &str, dir: &std::path::Path) {
    if !dir.is_dir() {
        tracing::warn!(dir = %dir.display(), "warm build: no integration worktree; skipped");
        return;
    }
    let started = std::time::Instant::now();
    let started_at = std::time::SystemTime::now();
    let out = Command::new("nice")
        .args(["-n", "10", "sh", "-c", cmd])
        .current_dir(dir)
        .kill_on_drop(true)
        .output()
        .await;
    match out {
        Ok(o) if o.status.success() => {
            tracing::info!(secs = started.elapsed().as_secs(), "warm build finished");
            let target = dir.join("target");
            let removed = tokio::task::spawn_blocking(move || {
                prune_stale(&target, started_at, BLOAT_ENTRIES)
            })
            .await
            .unwrap_or(0);
            if removed > 0 {
                tracing::info!(removed, "warm build: pruned stale target/ artifacts");
            }
        }
        Ok(o) => tracing::warn!(
            status = %o.status,
            stderr = %String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or(""),
            "warm build failed; ignored"
        ),
        Err(e) => tracing::warn!(error = %e, "warm build could not start; ignored"),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    async fn wait_idle(w: &WarmBuild) {
        for _ in 0..200 {
            if w.idle() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("warm build never went idle");
    }

    fn runs(dir: &std::path::Path) -> usize {
        std::fs::read_to_string(dir.join("runs"))
            .map(|s| s.lines().count())
            .unwrap_or(0)
    }

    #[test]
    fn prune_removes_stale_artifacts_and_keeps_current_ones() {
        let tmp = tempfile::tempdir().expect("tmp");
        let target = tmp.path();
        let deps = target.join("debug/deps");
        std::fs::create_dir_all(target.join("debug/build/old-crate")).expect("mk");
        std::fs::create_dir_all(&deps).expect("mk");
        std::fs::write(deps.join("old"), "x").expect("w");
        std::fs::write(target.join("debug/build/old-crate/out"), "x").expect("w");
        std::thread::sleep(Duration::from_millis(50));
        let cutoff = std::time::SystemTime::now();
        std::thread::sleep(Duration::from_millis(50));
        std::fs::write(deps.join("new"), "x").expect("w");

        // Under the threshold nothing is touched.
        assert_eq!(prune_stale(target, cutoff, 10), 0);
        assert!(deps.join("old").exists());

        // Over it, the artifact older than the run goes and the current one stays.
        assert_eq!(prune_stale(target, cutoff, 1), 2);
        assert!(!deps.join("old").exists());
        assert!(!target.join("debug/build/old-crate").exists());
        assert!(deps.join("new").exists());
    }

    #[tokio::test]
    async fn prune_after_land_clears_a_bloated_target_without_a_warm_build() {
        let tmp = tempfile::tempdir().expect("tmp");
        let deps = tmp.path().join("target/debug/deps");
        std::fs::create_dir_all(&deps).expect("mk");
        for n in 0..PRUNE_ENTRIES + 5 {
            std::fs::write(deps.join(format!("d{n}")), "x").expect("w");
        }
        std::thread::sleep(Duration::from_millis(50));
        let w = WarmBuild::new(None, tmp.path().to_path_buf());
        w.prune_after_land().await;
        assert_eq!(deps_entries(&tmp.path().join("target")), 0);
    }

    #[tokio::test]
    async fn trigger_returns_before_the_build_and_coalesces() {
        let tmp = tempfile::tempdir().expect("tmp");
        let w = WarmBuild::new(
            Some("echo x >> runs; sleep 0.5".to_string()),
            tmp.path().to_path_buf(),
        );
        let t = std::time::Instant::now();
        w.trigger();
        assert!(t.elapsed() < Duration::from_millis(300), "trigger blocked");
        assert!(!w.idle());
        // Three triggers during one build make one more build, not three.
        w.trigger();
        w.trigger();
        w.trigger();
        wait_idle(&w).await;
        assert_eq!(runs(tmp.path()), 2);
    }

    #[tokio::test]
    async fn failure_is_ignored_and_unset_command_does_nothing() {
        let tmp = tempfile::tempdir().expect("tmp");
        let w = WarmBuild::new(
            Some("echo x >> runs; exit 1".to_string()),
            tmp.path().to_path_buf(),
        );
        w.trigger();
        wait_idle(&w).await;
        w.trigger();
        wait_idle(&w).await;
        assert_eq!(runs(tmp.path()), 2);

        let none = WarmBuild::new(None, tmp.path().to_path_buf());
        none.trigger();
        assert!(none.idle());
    }
}
