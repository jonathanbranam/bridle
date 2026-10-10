//! Restart in place (docs/design/agent-host/daemon.md, "Restart in place"): the record of which
//! agents were running when the daemon exec'd itself, and the resume of every one of them after.

use std::path::Path;

use bridle_api::types::{MessageKind, WakeReason, When};
use serde_json::json;

use crate::store::Store;
use crate::supervisor::{AgentManager, SupervisorError, ToTarget, system_principal};
use crate::wake::Wakes;
use crate::worktree;

/// `meta` key holding a JSON array of the agent ids that were running at the restart. Empty or
/// absent means this start is not a restart in place; the new process clears it once read.
const RESUME_KEY: &str = "restart.resume";

/// The integration branch's head, short, for the wakes (`None` if git can't say).
pub async fn head(repo: &Path, integration: &str) -> Option<String> {
    worktree::run_git(
        repo,
        &["rev-parse", "--short", &format!("refs/heads/{integration}")],
    )
    .await
    .ok()
    .map(|s| s.trim().to_string())
}

/// The commit the daemon runs, short: the one the last upgrade built, which can trail the
/// integration head (commits land during a build). Before any upgrade, the head.
pub async fn running_commit(store: &Store, repo: &Path, integration: &str) -> Option<String> {
    if let Some(built) = crate::upgrade::built(store).await
        && let Ok(short) = worktree::run_git(repo, &["rev-parse", "--short", &built]).await
    {
        return Some(short.trim().to_string());
    }
    // A release tag (`self_upgrade = "release"`) needn't be in the checkout.
    if let Some(tag) = crate::upgrade::built(store).await
        && tag.starts_with('v')
    {
        return Some(tag);
    }
    head(repo, integration).await
}

pub async fn record(store: &Store, ids: &[String]) -> Result<(), crate::store::StoreError> {
    store
        .swap_meta(RESUME_KEY, &serde_json::to_string(ids).unwrap_or_default())
        .await
        .map(|_| ())
}

async fn take(store: &Store) -> Vec<String> {
    let raw = store.get_meta(RESUME_KEY).await.ok().flatten();
    let ids: Vec<String> = raw
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or_default();
    if !ids.is_empty() {
        let _ = store.swap_meta(RESUME_KEY, "").await;
    }
    ids
}

/// After startup's own resume of `resume_on_restart` roles: resume every other agent that was
/// running before the restart, tell each to carry on, and wake the orchestrator with what came
/// back. The human hears (inbox) only about agents that failed to.
pub async fn resume_all(
    store: &Store,
    manager: &AgentManager,
    wakes: &Wakes,
    repo: &Path,
    integration: &str,
) {
    let ids = take(store).await;
    if ids.is_empty() {
        return;
    }
    let system = system_principal();
    let commit = running_commit(store, repo, integration).await;
    let (mut resumed, mut failed) = (Vec::new(), Vec::new());
    for id in &ids {
        let Ok(Some(agent)) = store.get_agent(id).await else {
            failed.push(format!("{id}: not found"));
            continue;
        };
        if !agent.state.is_running()
            && let Err(e) = manager.resume(id, false, &system).await
        {
            failed.push(format!("{}: {e}", agent.name));
            continue;
        }
        let body = format!(
            "The daemon restarted for an upgrade (now at {}). You were resumed; carry on where you left off, and re-run any background job you were waiting on.",
            commit.as_deref().unwrap_or("an unknown commit")
        );
        match manager
            .send(
                "system".to_string(),
                ToTarget::Agent(id.clone()),
                MessageKind::Note,
                body,
                When::Idle,
                None,
            )
            .await
        {
            Ok(_) => resumed.push(agent.name),
            Err(SupervisorError::Conflict(_)) | Err(SupervisorError::NotFound(_)) => {
                failed.push(format!("{}: could not be told", agent.name))
            }
            Err(e) => failed.push(format!("{}: {e}", agent.name)),
        }
    }
    let text = format!(
        "daemon restarted at {}: resumed {} of {} agents{}",
        commit.as_deref().unwrap_or("?"),
        resumed.len(),
        ids.len(),
        if failed.is_empty() {
            String::new()
        } else {
            format!("; failed: {}", failed.join("; "))
        }
    );
    wakes
        .push(WakeReason {
            reason: "restart".to_string(),
            text: text.clone(),
            detail: json!({"commit": commit, "resumed": resumed, "failed": failed}),
        })
        .await;
    if !failed.is_empty() {
        let _ = manager
            .send(
                "system".to_string(),
                ToTarget::Human,
                MessageKind::Note,
                text,
                When::Idle,
                None,
            )
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn running_commit_is_the_built_one_not_the_head() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(["-c", "user.name=t", "-c", "user.email=t@t"])
                .args(args)
                .current_dir(repo)
                .output()
                .expect("git");
            assert!(out.status.success(), "git {args:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["commit", "-q", "--allow-empty", "-m", "built"]);
        let built = git(&["rev-parse", "HEAD"]);
        git(&[
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "landed during the build",
        ]);
        let head_short = git(&["rev-parse", "--short", "HEAD"]);
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("store");

        // No upgrade yet: the head.
        assert_eq!(
            running_commit(&store, repo, "main").await.as_deref(),
            Some(head_short.as_str())
        );
        crate::upgrade::record_built(&store, &built).await;
        let got = running_commit(&store, repo, "main").await.expect("commit");
        assert!(built.starts_with(&got));
        assert_ne!(got, head_short);
    }
}
