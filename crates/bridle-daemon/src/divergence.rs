//! Divergence from origin: on a timer (and in `bridle doctor`), `git fetch origin` in the
//! owner's clone and compare `origin/<integration>` with the local integration branch. A
//! clone silently far ahead of or behind origin hid for hours in incident br-2y3m (postmortem
//! j7r4); a plain `[ahead 23]` did not stand out. Any difference is reported as
//! "N ahead, M behind" and reaches the orchestrator as a `git.diverged` event and a message,
//! once per change of the counts (ticket k6jd).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bridle_api::types::{MessageKind, When, event_kind};
use serde_json::json;

use crate::events::Emitter;
use crate::supervisor::{AgentManager, ToTarget};

pub const INTERVAL: Duration = Duration::from_secs(10 * 60);
const FETCH_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Divergence {
    pub ahead: u32,
    pub behind: u32,
}

impl std::fmt::Display for Divergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ahead, {} behind", self.ahead, self.behind)
    }
}

fn git(repo: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo)
        .args(args)
        // Fail rather than wait on a credential prompt nobody will answer.
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null());
    cmd
}

fn git_ok(repo: &Path, args: &[&str]) -> bool {
    git(repo, args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// `git fetch origin`, killed after [`FETCH_TIMEOUT`]. The error is git's own last line.
fn fetch(repo: &Path) -> Result<(), String> {
    let mut child = git(repo, &["fetch", "origin"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("running git: {e}"))?;
    let deadline = Instant::now() + FETCH_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) => {
                let out = child.wait_with_output().map_err(|e| e.to_string())?;
                let text = String::from_utf8_lossy(&out.stderr);
                let line = text.lines().rev().find(|l| !l.trim().is_empty());
                return Err(line.unwrap_or("git fetch origin failed").trim().to_string());
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "git fetch origin timed out after {}s",
                    FETCH_TIMEOUT.as_secs()
                ));
            }
            Err(e) => return Err(format!("waiting for git: {e}")),
        }
    }
}

/// Fetches, then compares. `Ok(None)` when there is no `origin` remote or no
/// `origin/<integration>` (nothing to compare); `Ok(Some(..))` with zeros when in step.
/// Blocking.
pub fn check(repo: &Path, integration: &str) -> Result<Option<Divergence>, String> {
    if !git_ok(repo, &["remote", "get-url", "origin"]) {
        return Ok(None);
    }
    fetch(repo)?;
    let remote = format!("refs/remotes/origin/{integration}");
    if !git_ok(repo, &["rev-parse", "--verify", "--quiet", &remote]) {
        return Ok(None);
    }
    let range = format!("refs/heads/{integration}...{remote}");
    let out = git(repo, &["rev-list", "--left-right", "--count", &range])
        .output()
        .map_err(|e| format!("running git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "comparing {integration} with origin/{integration}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut n = text.split_whitespace().map(|w| w.parse::<u32>());
    match (n.next(), n.next()) {
        (Some(Ok(ahead)), Some(Ok(behind))) => Ok(Some(Divergence { ahead, behind })),
        _ => Err(format!("unexpected rev-list output: {}", text.trim())),
    }
}

#[derive(Clone)]
pub struct DivergenceWatch(Arc<Inner>);

struct Inner {
    repo: PathBuf,
    integration: String,
    emitter: Emitter,
    manager: AgentManager,
    /// The counts last reported, so the orchestrator hears again only when they change;
    /// `None` while in step.
    last: Mutex<Option<Divergence>>,
}

impl DivergenceWatch {
    pub fn new(
        repo: PathBuf,
        integration: String,
        emitter: Emitter,
        manager: AgentManager,
    ) -> Self {
        DivergenceWatch(Arc::new(Inner {
            repo,
            integration,
            emitter,
            manager,
            last: Mutex::new(None),
        }))
    }

    pub async fn tick(&self) {
        let (repo, integration) = (self.0.repo.clone(), self.0.integration.clone());
        let found = match tokio::task::spawn_blocking(move || check(&repo, &integration)).await {
            Ok(Ok(found)) => found,
            Ok(Err(e)) => return tracing::warn!(error = %e, "origin divergence check failed"),
            Err(e) => return tracing::warn!(error = %e, "origin divergence check panicked"),
        };
        let now = found.filter(|d| d.ahead > 0 || d.behind > 0);
        let before = std::mem::replace(&mut *self.0.last.lock().expect("divergence lock"), now);
        if now == before {
            return;
        }
        let d = now.unwrap_or(Divergence {
            ahead: 0,
            behind: 0,
        });
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::GIT_DIVERGED,
                "system".to_string(),
                None,
                json!({"integration": self.0.integration, "ahead": d.ahead, "behind": d.behind}),
            )
            .await;
        if now.is_none() {
            return;
        }
        let body = format!(
            "This clone's {0} has diverged from origin/{0}: {d}. Find out why before landing or \
             pushing more (`git log --left-right --oneline {0}...origin/{0}`); a clone that is \
             far ahead or behind is how br-2y3m went unnoticed.",
            self.0.integration
        );
        let _ = self
            .0
            .manager
            .send(
                "system".to_string(),
                ToTarget::External(crate::wake::ORCHESTRATOR.to_string()),
                MessageKind::Note,
                body,
                When::Now,
                None,
            )
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .stdout(Stdio::null())
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    /// A bare origin and a clone with one commit on `main`, pushed.
    fn pair() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let origin = tmp.path().join("origin.git");
        let clone = tmp.path().join("clone");
        std::fs::create_dir(&origin).unwrap();
        run(&origin, &["init", "--bare", "-b", "main"]);
        run(tmp.path(), &["clone", origin.to_str().unwrap(), "clone"]);
        run(&clone, &["checkout", "-b", "main"]);
        run(&clone, &["commit", "--allow-empty", "-m", "one"]);
        run(&clone, &["push", "origin", "main"]);
        (tmp, clone)
    }

    #[test]
    fn in_step_ahead_and_behind() {
        let (tmp, clone) = pair();
        let zero = Divergence {
            ahead: 0,
            behind: 0,
        };
        assert_eq!(check(&clone, "main").unwrap(), Some(zero));

        run(&clone, &["commit", "--allow-empty", "-m", "two"]);
        run(&clone, &["commit", "--allow-empty", "-m", "three"]);
        let d = check(&clone, "main").unwrap().unwrap();
        assert_eq!(d.to_string(), "2 ahead, 0 behind");

        // Another clone pushes one commit: diverged.
        let other = tmp.path().join("other");
        run(
            tmp.path(),
            &[
                "clone",
                tmp.path().join("origin.git").to_str().unwrap(),
                "other",
            ],
        );
        run(&other, &["commit", "--allow-empty", "-m", "theirs"]);
        run(&other, &["push", "origin", "HEAD:main"]);
        let d = check(&clone, "main").unwrap().unwrap();
        assert_eq!(d.to_string(), "2 ahead, 1 behind");
    }

    #[test]
    fn silent_without_remote_or_remote_branch() {
        let tmp = tempfile::tempdir().unwrap();
        run(tmp.path(), &["init", "-b", "main"]);
        run(tmp.path(), &["commit", "--allow-empty", "-m", "one"]);
        assert_eq!(check(tmp.path(), "main").unwrap(), None);

        let (_t, clone) = pair();
        assert_eq!(check(&clone, "nope").unwrap(), None);
    }

    #[test]
    fn fetch_failure_is_an_error() {
        let (_t, clone) = pair();
        run(
            &clone,
            &["remote", "set-url", "origin", "/nonexistent/origin.git"],
        );
        assert!(check(&clone, "main").is_err());
    }
}
