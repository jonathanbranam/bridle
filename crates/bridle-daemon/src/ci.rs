//! CI watcher: when the integration branch's tip changes on `origin`, polls
//! GitHub Actions (through `gh`) until every run for that commit finishes,
//! emits `ci.completed`, and tells the manager if it failed. Opt-in with
//! `[ci] github = true`; off, `tick` does nothing (ticket c8qw).

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bridle_api::types::{CiStatus, MessageKind, When, event_kind};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::events::Emitter;
use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget};

/// One tick per minute; the counts below are in ticks, so the cadence is
/// deterministic to test.
pub const TICK_INTERVAL: Duration = Duration::from_secs(60);
/// Look for a new tip every third tick while nothing is pending.
const TIP_CHECK_EVERY: u32 = 3;
/// Give up on a commit's runs after about an hour.
const MAX_POLLS: u32 = 60;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub database_id: u64,
    pub status: String,
    #[serde(default)]
    pub conclusion: String,
    #[serde(default)]
    pub url: String,
}

/// The two external tools the watcher shells out to, behind a trait so tests
/// inject canned output. Blocking; the watcher calls it from `spawn_blocking`.
pub trait Gh: Send + Sync + 'static {
    /// The tip of `branch` on `origin`, `None` if it doesn't exist there.
    fn remote_tip(&self, branch: &str) -> Result<Option<String>, String>;
    fn runs(&self, sha: &str) -> Result<Vec<Run>, String>;
    /// Names of the failed jobs of one run.
    fn failed_jobs(&self, run_id: u64) -> Result<Vec<String>, String>;
}

pub struct RealGh {
    cwd: PathBuf,
}

impl RealGh {
    pub fn new(cwd: PathBuf) -> Self {
        RealGh { cwd }
    }

    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let out = Command::new(program)
            .args(args)
            .current_dir(&self.cwd)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| format!("running {program}: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "{program} {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

impl Gh for RealGh {
    fn remote_tip(&self, branch: &str) -> Result<Option<String>, String> {
        let out = self.run(
            "git",
            &["ls-remote", "origin", &format!("refs/heads/{branch}")],
        )?;
        Ok(out.split_whitespace().next().map(str::to_string))
    }

    fn runs(&self, sha: &str) -> Result<Vec<Run>, String> {
        let out = self.run(
            "gh",
            &[
                "run",
                "list",
                "--commit",
                sha,
                "--json",
                "databaseId,status,conclusion,url,name",
            ],
        )?;
        serde_json::from_str(&out).map_err(|e| format!("parsing gh run list: {e}"))
    }

    fn failed_jobs(&self, run_id: u64) -> Result<Vec<String>, String> {
        #[derive(Deserialize)]
        struct Jobs {
            jobs: Vec<Job>,
        }
        #[derive(Deserialize)]
        struct Job {
            name: String,
            #[serde(default)]
            conclusion: String,
        }
        let out = self.run(
            "gh",
            &["run", "view", &run_id.to_string(), "--json", "jobs"],
        )?;
        let jobs: Jobs =
            serde_json::from_str(&out).map_err(|e| format!("parsing gh run view: {e}"))?;
        Ok(jobs
            .jobs
            .into_iter()
            .filter(|j| j.conclusion == "failure")
            .map(|j| j.name)
            .collect())
    }
}

#[derive(Default)]
struct State {
    /// The tip last seen on origin; a different one starts a watch.
    last_tip: Option<String>,
    /// The commit being polled and how many polls it has had.
    pending: Option<(String, u32)>,
    /// Ticks to skip before the next tip check.
    tip_countdown: u32,
    /// A `gh`/`git` failure was already logged; cleared by the next success,
    /// so an outage is one warning, not one per tick.
    warned: bool,
}

struct Inner {
    enabled: bool,
    branch: String,
    gh: Arc<dyn Gh>,
    store: Store,
    manager: AgentManager,
    emitter: Emitter,
    state: tokio::sync::Mutex<State>,
    last: Mutex<Option<CiStatus>>,
}

#[derive(Clone)]
pub struct CiWatcher(Arc<Inner>);

impl CiWatcher {
    pub fn new(
        enabled: bool,
        branch: String,
        gh: Arc<dyn Gh>,
        store: Store,
        manager: AgentManager,
        emitter: Emitter,
    ) -> Self {
        CiWatcher(Arc::new(Inner {
            enabled,
            branch,
            gh,
            store,
            manager,
            emitter,
            state: tokio::sync::Mutex::new(State::default()),
            last: Mutex::new(None),
        }))
    }

    /// The last finished result, for `bridle status`.
    pub fn last(&self) -> Option<CiStatus> {
        self.0.last.lock().expect("ci last lock").clone()
    }

    pub async fn tick(&self) {
        if !self.0.enabled {
            return;
        }
        let mut st = self.0.state.lock().await;
        if st.pending.is_none() {
            if st.tip_countdown > 0 {
                st.tip_countdown -= 1;
                return;
            }
            st.tip_countdown = TIP_CHECK_EVERY - 1;
            let branch = self.0.branch.clone();
            let tip = match self.call(&mut st, move |gh| gh.remote_tip(&branch)).await {
                Some(Some(tip)) if st.last_tip.as_deref() != Some(&tip) => tip,
                _ => return,
            };
            st.last_tip = Some(tip.clone());
            st.pending = Some((tip, 0));
        }
        self.poll(&mut st).await;
    }

    /// Runs one blocking `gh`/`git` call; a failure is logged once until the
    /// next success and yields `None`.
    async fn call<T: Send + 'static>(
        &self,
        st: &mut State,
        f: impl FnOnce(&dyn Gh) -> Result<T, String> + Send + 'static,
    ) -> Option<T> {
        let gh = self.0.gh.clone();
        let res = tokio::task::spawn_blocking(move || f(gh.as_ref()))
            .await
            .unwrap_or_else(|e| Err(e.to_string()));
        match res {
            Ok(v) => {
                st.warned = false;
                Some(v)
            }
            Err(e) => {
                if !st.warned {
                    tracing::warn!(error = %e, "CI watch: gh/git failed; retrying next tick");
                    st.warned = true;
                }
                None
            }
        }
    }

    async fn poll(&self, st: &mut State) {
        let Some((sha, polls)) = st.pending.as_mut() else {
            return;
        };
        *polls += 1;
        let give_up = *polls >= MAX_POLLS;
        let sha = sha.clone();
        let runs = {
            let sha = sha.clone();
            self.call(st, move |gh| gh.runs(&sha)).await
        };
        // No runs yet is not done: GitHub takes a moment to create them.
        let done = runs.filter(|r| !r.is_empty() && r.iter().all(|r| r.status == "completed"));
        match done {
            Some(runs) => {
                st.pending = None;
                self.finish(&sha, runs).await;
            }
            None if give_up => {
                tracing::warn!(%sha, "CI watch: giving up, runs never finished");
                st.pending = None;
            }
            None => {}
        }
    }

    async fn finish(&self, sha: &str, runs: Vec<Run>) {
        let failed: Vec<&Run> = runs
            .iter()
            .filter(|r| {
                matches!(
                    r.conclusion.as_str(),
                    "failure" | "timed_out" | "startup_failure"
                )
            })
            .collect();
        let conclusion = if !failed.is_empty() {
            "failure"
        } else if runs.iter().any(|r| r.conclusion == "cancelled") {
            "cancelled"
        } else {
            "success"
        };
        let url = failed
            .first()
            .copied()
            .or(runs.first())
            .map(|r| r.url.clone());

        *self.0.last.lock().expect("ci last lock") = Some(CiStatus {
            sha: sha.to_string(),
            conclusion: conclusion.to_string(),
            url: url.clone(),
            completed_at: Utc::now(),
        });
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::CI_COMPLETED,
                "system".to_string(),
                None,
                json!({"sha": sha, "conclusion": conclusion, "url": url}),
            )
            .await;
        if conclusion != "failure" {
            return;
        }

        let mut jobs = Vec::new();
        for run in &failed {
            let id = run.database_id;
            match tokio::task::spawn_blocking({
                let gh = self.0.gh.clone();
                move || gh.failed_jobs(id)
            })
            .await
            {
                Ok(Ok(names)) => jobs.extend(names),
                _ => tracing::warn!(run = id, "CI watch: couldn't list failed jobs"),
            }
        }
        let body = format!(
            "CI failed on {sha}: {}; {}. Don't merge until it's green.",
            if jobs.is_empty() {
                "(job names unavailable)".to_string()
            } else {
                jobs.join(", ")
            },
            url.as_deref().unwrap_or("no url"),
        );
        self.notify_manager(body).await;
    }

    /// To the first running manager; the human if there is none.
    async fn notify_manager(&self, body: String) {
        let manager = match self.0.store.list_agents(false).await {
            Ok(agents) => agents
                .into_iter()
                .find(|a| a.role == "manager" && a.state.is_running()),
            Err(_) => None,
        };
        let to = match manager {
            Some(a) => ToTarget::Agent(a.id),
            None => ToTarget::Human,
        };
        let _ = self
            .0
            .manager
            .send(
                "system".to_string(),
                to,
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
    use std::collections::VecDeque;

    use super::*;
    use crate::config::Config;
    use crate::paths::Workspace;
    use crate::store::{ListMessages, NewAgent};

    /// Scripted `gh`: each call pops the next canned answer (the last one
    /// repeats) and is counted.
    #[derive(Default)]
    struct FakeGh {
        tips: Mutex<VecDeque<Result<Option<String>, String>>>,
        runs: Mutex<VecDeque<Result<Vec<Run>, String>>>,
        calls: Mutex<u32>,
    }

    fn pop<T: Clone>(q: &Mutex<VecDeque<T>>) -> T {
        let mut q = q.lock().unwrap();
        if q.len() > 1 {
            q.pop_front().unwrap()
        } else {
            q.front().expect("scripted answer").clone()
        }
    }

    impl Gh for FakeGh {
        fn remote_tip(&self, _: &str) -> Result<Option<String>, String> {
            *self.calls.lock().unwrap() += 1;
            pop(&self.tips)
        }
        fn runs(&self, _: &str) -> Result<Vec<Run>, String> {
            *self.calls.lock().unwrap() += 1;
            pop(&self.runs)
        }
        fn failed_jobs(&self, _: u64) -> Result<Vec<String>, String> {
            Ok(vec!["test (ubuntu-latest)".to_string(), "lint".to_string()])
        }
    }

    fn run(status: &str, conclusion: &str) -> Run {
        Run {
            database_id: 7,
            status: status.to_string(),
            conclusion: conclusion.to_string(),
            url: "https://gh/run/7".to_string(),
        }
    }

    struct Fixture {
        watcher: CiWatcher,
        gh: Arc<FakeGh>,
        store: Store,
        _dir: tempfile::TempDir,
    }

    async fn fixture(enabled: bool, gh: FakeGh) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("bridle.db")).await.unwrap();
        let emitter = Emitter::new(store.clone());
        let manager = AgentManager::new(
            store.clone(),
            Workspace::new(dir.path().join("repo"), None),
            Config::default(),
            "claude".to_string(),
            "http://127.0.0.1:0".to_string(),
            "test".to_string(),
            emitter.clone(),
            Default::default(),
        );
        let gh = Arc::new(gh);
        let watcher = CiWatcher::new(
            enabled,
            "main".to_string(),
            gh.clone(),
            store.clone(),
            manager,
            emitter,
        );
        Fixture {
            watcher,
            gh,
            store,
            _dir: dir,
        }
    }

    fn script(
        tips: Vec<Result<Option<String>, String>>,
        runs: Vec<Result<Vec<Run>, String>>,
    ) -> FakeGh {
        FakeGh {
            tips: Mutex::new(tips.into()),
            runs: Mutex::new(runs.into()),
            calls: Mutex::new(0),
        }
    }

    fn tip(s: &str) -> Result<Option<String>, String> {
        Ok(Some(s.to_string()))
    }

    async fn ci_events(store: &Store) -> Vec<bridle_api::types::Event> {
        store
            .list_events(bridle_api::types::EventQuery {
                kind: Some("ci.".to_string()),
                ..Default::default()
            })
            .await
            .unwrap()
    }

    async fn human_messages(store: &Store) -> Vec<String> {
        store
            .list_messages(ListMessages {
                to: Some("human".to_string()),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.body)
            .collect()
    }

    #[tokio::test]
    async fn disabled_makes_no_calls() {
        let f = fixture(false, script(vec![tip("abc")], vec![Ok(vec![])])).await;
        for _ in 0..5 {
            f.watcher.tick().await;
        }
        assert_eq!(*f.gh.calls.lock().unwrap(), 0);
        assert!(f.watcher.last().is_none());
    }

    #[tokio::test]
    async fn success_emits_event_and_sends_nothing() {
        let f = fixture(
            true,
            script(
                vec![tip("abc")],
                vec![Ok(vec![run("completed", "success")])],
            ),
        )
        .await;
        f.watcher.tick().await;
        let last = f.watcher.last().unwrap();
        assert_eq!(
            (last.sha.as_str(), last.conclusion.as_str()),
            ("abc", "success")
        );
        assert_eq!(ci_events(&f.store).await.len(), 1);
        assert!(human_messages(&f.store).await.is_empty());
    }

    #[tokio::test]
    async fn failure_messages_the_manager_with_failed_jobs() {
        let f = fixture(
            true,
            script(
                vec![tip("abc")],
                vec![Ok(vec![run("completed", "failure")])],
            ),
        )
        .await;
        let manager = f
            .store
            .insert_agent(NewAgent {
                name: "mgr".to_string(),
                role: "manager".to_string(),
                model: "sonnet".to_string(),
                session_id: "s".to_string(),
                workdir_kind: "repo".to_string(),
                cwd: "/".to_string(),
                worktree: None,
                branch: None,
                created_by: "human".to_string(),
                extra_allowed_tools: vec![],
                extra_env: vec![],
                components: vec![],
            })
            .await
            .unwrap();
        f.store
            .set_agent_state(&manager.id, bridle_api::types::AgentState::Idle)
            .await
            .unwrap();
        f.watcher.tick().await;
        assert_eq!(f.watcher.last().unwrap().conclusion, "failure");
        let msgs = f
            .store
            .list_messages(ListMessages {
                to: Some(manager.id.clone()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].body.contains("CI failed on abc"));
        assert!(msgs[0].body.contains("test (ubuntu-latest), lint"));
        assert!(msgs[0].body.contains("https://gh/run/7"));
    }

    #[tokio::test]
    async fn failure_without_a_manager_goes_to_the_human() {
        let f = fixture(
            true,
            script(
                vec![tip("abc")],
                vec![Ok(vec![run("completed", "failure")])],
            ),
        )
        .await;
        f.watcher.tick().await;
        assert_eq!(human_messages(&f.store).await.len(), 1);
    }

    #[tokio::test]
    async fn waits_through_in_progress_and_no_runs() {
        let f = fixture(
            true,
            script(
                vec![tip("abc")],
                vec![
                    Ok(vec![]),
                    Ok(vec![run("in_progress", "")]),
                    Ok(vec![run("completed", "success")]),
                ],
            ),
        )
        .await;
        f.watcher.tick().await;
        f.watcher.tick().await;
        assert!(f.watcher.last().is_none());
        f.watcher.tick().await;
        assert_eq!(f.watcher.last().unwrap().conclusion, "success");
        // Done: no more polling of that sha (tip checks are skipped for a couple of ticks).
        let calls = *f.gh.calls.lock().unwrap();
        f.watcher.tick().await;
        assert_eq!(*f.gh.calls.lock().unwrap(), calls);
    }

    #[tokio::test]
    async fn gh_error_keeps_going_next_tick() {
        let f = fixture(
            true,
            script(
                vec![tip("abc")],
                vec![
                    Err("gh: not found".to_string()),
                    Err("gh: not found".to_string()),
                    Ok(vec![run("completed", "success")]),
                ],
            ),
        )
        .await;
        f.watcher.tick().await;
        assert!(f.watcher.0.state.lock().await.warned);
        f.watcher.tick().await;
        f.watcher.tick().await;
        assert_eq!(f.watcher.last().unwrap().conclusion, "success");
        assert!(!f.watcher.0.state.lock().await.warned);
    }

    #[tokio::test]
    async fn unchanged_tip_is_not_rewatched() {
        let f = fixture(
            true,
            script(
                vec![tip("abc")],
                vec![Ok(vec![run("completed", "success")])],
            ),
        )
        .await;
        for _ in 0..(TIP_CHECK_EVERY * 3) {
            f.watcher.tick().await;
        }
        assert_eq!(ci_events(&f.store).await.len(), 1);
    }
}
