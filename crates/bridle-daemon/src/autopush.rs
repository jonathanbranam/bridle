//! Pushes the integration branch when the owner's clone is ahead of origin. Direct commits
//! on it (tickets, notes, made by the orchestrator, advisors and aides) never go through
//! `land`, so nothing pushed them and origin lagged for hours in incident br-2y3m
//! (postmortem j7r4, ticket 8ay6). Fast-forward only: a clone that is behind or diverged is
//! the divergence watch's business (k6jd), never a push, never a force.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::types::{MessageKind, When, event_kind};
use serde_json::json;
use tokio::process::Command;

use crate::events::Emitter;
use crate::supervisor::{AgentManager, ToTarget};

pub const INTERVAL: Duration = Duration::from_secs(30);
const PUSH_TIMEOUT: Duration = Duration::from_secs(60);

async fn git(repo: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let run = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        // Fail rather than wait on a credential prompt nobody will answer.
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true)
        .output();
    match tokio::time::timeout(PUSH_TIMEOUT, run).await {
        Ok(Ok(out)) => Ok(out),
        Ok(Err(e)) => Err(format!("running git: {e}")),
        Err(_) => Err(format!("git {} timed out", args.first().unwrap_or(&""))),
    }
}

/// What a rejected push says: git's first non-empty stderr line.
pub fn first_line(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("git push failed")
        .to_string()
}

/// Records a failed push of the integration branch (`push.failed`) and tells the human and the
/// orchestrator. Shared by `bridle push` and the automatic push.
pub async fn report_failure(
    emitter: &Emitter,
    manager: &AgentManager,
    actor: String,
    project: &str,
    integration: &str,
    error: &str,
    body: String,
) {
    let _ = emitter
        .emit(
            event_kind::PUSH_FAILED,
            actor,
            None,
            json!({"project": project, "branch": integration, "error": error}),
        )
        .await;
    for to in [
        ToTarget::Human,
        ToTarget::External(crate::wake::ORCHESTRATOR.to_string()),
    ] {
        let _ = manager
            .send(
                "system".to_string(),
                to,
                MessageKind::Note,
                body.clone(),
                When::Now,
                None,
            )
            .await;
    }
}

/// One look: with the branch strictly ahead of `origin/<integration>` (by the last fetch or
/// push; no fetch here, only a `rev-list`), pushes it. `Ok(true)` when it pushed, `Ok(false)`
/// when there was nothing to do (in step, behind, diverged, or no origin), `Err` with git's
/// first line when the push was rejected or failed.
pub async fn push_if_ahead(repo: &Path, integration: &str) -> Result<bool, String> {
    let range = format!("refs/heads/{integration}...refs/remotes/origin/{integration}");
    let out = git(repo, &["rev-list", "--left-right", "--count", &range]).await?;
    if !out.status.success() {
        // No origin, or no such branch there yet: nothing to compare.
        return Ok(false);
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut n = text.split_whitespace().map(|w| w.parse::<u32>());
    let (Some(Ok(ahead)), Some(Ok(behind))) = (n.next(), n.next()) else {
        return Ok(false);
    };
    if ahead == 0 || behind > 0 {
        return Ok(false);
    }
    let out = git(repo, &["push", "origin", integration]).await?;
    if out.status.success() {
        Ok(true)
    } else {
        Err(first_line(&out.stderr))
    }
}

#[derive(Clone)]
pub struct AutoPush(Arc<Inner>);

struct Inner {
    repo: std::path::PathBuf,
    project: String,
    integration: String,
    emitter: Emitter,
    manager: AgentManager,
    /// The error last alarmed, so a push that keeps failing the same way alarms once; it is
    /// still retried every tick. Cleared by a success.
    last_error: Mutex<Option<String>>,
}

impl AutoPush {
    pub fn new(
        repo: std::path::PathBuf,
        project: String,
        integration: String,
        emitter: Emitter,
        manager: AgentManager,
    ) -> Self {
        AutoPush(Arc::new(Inner {
            repo,
            project,
            integration,
            emitter,
            manager,
            last_error: Mutex::new(None),
        }))
    }

    pub async fn tick(&self) {
        let i = &self.0;
        match push_if_ahead(&i.repo, &i.integration).await {
            Ok(pushed) => {
                if pushed {
                    tracing::info!(branch = %i.integration, "pushed direct commits to origin");
                }
                *i.last_error.lock().expect("autopush lock") = None;
            }
            Err(e) => {
                let before = i
                    .last_error
                    .lock()
                    .expect("autopush lock")
                    .replace(e.clone());
                if before.as_deref() == Some(e.as_str()) {
                    return;
                }
                let body = format!(
                    "The automatic push of {} to origin failed ({}): {e}. Direct commits \
                     (tickets, notes) are on the local branch only; find out why.",
                    i.integration, i.project
                );
                report_failure(
                    &i.emitter,
                    &i.manager,
                    "system".to_string(),
                    &i.project,
                    &i.integration,
                    &e,
                    body,
                )
                .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    fn pair() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let origin = tmp.path().join("origin.git");
        let clone = tmp.path().join("clone");
        std::fs::create_dir(&origin).unwrap();
        run(&origin, &["init", "--bare", "-b", "main"]);
        run(tmp.path(), &["clone", origin.to_str().unwrap(), "clone"]);
        run(&clone, &["checkout", "-b", "main"]);
        run(&clone, &["commit", "--allow-empty", "-m", "one"]);
        run(&clone, &["push", "origin", "main"]);
        (tmp, origin, clone)
    }

    fn origin_tip(origin: &Path) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(origin)
            .args(["rev-parse", "main"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    }

    #[tokio::test]
    async fn a_direct_commit_is_pushed_and_in_step_is_left_alone() {
        let (_tmp, origin, clone) = pair();
        assert_eq!(push_if_ahead(&clone, "main").await, Ok(false));
        run(&clone, &["commit", "--allow-empty", "-m", "ticket"]);
        assert_ne!(origin_tip(&origin), {
            let o = std::process::Command::new("git")
                .arg("-C")
                .arg(&clone)
                .args(["rev-parse", "main"])
                .output()
                .unwrap();
            String::from_utf8(o.stdout).unwrap()
        });
        assert_eq!(push_if_ahead(&clone, "main").await, Ok(true));
        let o = std::process::Command::new("git")
            .arg("-C")
            .arg(&clone)
            .args(["rev-parse", "main"])
            .output()
            .unwrap();
        assert_eq!(origin_tip(&origin), String::from_utf8(o.stdout).unwrap());
        assert_eq!(push_if_ahead(&clone, "main").await, Ok(false));
    }

    #[tokio::test]
    async fn a_behind_or_diverged_clone_is_never_pushed() {
        let (tmp, origin, clone) = pair();
        let other = tmp.path().join("other");
        run(tmp.path(), &["clone", origin.to_str().unwrap(), "other"]);
        run(&other, &["commit", "--allow-empty", "-m", "theirs"]);
        run(&other, &["push", "origin", "HEAD:main"]);
        run(&clone, &["fetch", "origin"]);
        run(&clone, &["commit", "--allow-empty", "-m", "mine"]);
        let before = origin_tip(&origin);
        assert_eq!(push_if_ahead(&clone, "main").await, Ok(false));
        assert_eq!(origin_tip(&origin), before);
    }

    #[tokio::test]
    async fn a_rejecting_origin_is_an_error_with_gits_line() {
        let (_tmp, origin, clone) = pair();
        let hook = origin.join("hooks/pre-receive");
        std::fs::write(&hook, "#!/bin/sh\necho 'nope: not the owner' >&2\nexit 1\n").unwrap();
        run(&origin, &["config", "core.hooksPath", "hooks"]);
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(&hook)
            .status()
            .unwrap();
        run(&clone, &["commit", "--allow-empty", "-m", "ticket"]);
        let e = push_if_ahead(&clone, "main").await.unwrap_err();
        assert!(!e.is_empty(), "{e}");
    }
}
