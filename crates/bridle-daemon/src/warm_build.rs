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
    let out = Command::new("nice")
        .args(["-n", "10", "sh", "-c", cmd])
        .current_dir(dir)
        .kill_on_drop(true)
        .output()
        .await;
    match out {
        Ok(o) if o.status.success() => {
            tracing::info!(secs = started.elapsed().as_secs(), "warm build finished");
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
