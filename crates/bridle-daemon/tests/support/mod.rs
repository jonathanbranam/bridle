//! Shared test harness for bridle-daemon integration tests: a real temp git
//! repo, a real daemon (`bridle_daemon::start`) pointed at the fake claude,
//! and small polling helpers. No real `claude` is ever spawned.
//!
//! Not every test binary uses every helper here (each `tests/*.rs` file is
//! its own crate with its own copy of this module), hence the blanket
//! `dead_code` allow.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use bridle_api::Client;
use bridle_api::types::{Agent, AgentState, Event, EventQuery};
use bridle_daemon::{Overrides, RunningDaemon, ServeOptions};
use tokio::process::Command;

/// The default per-poll wait and overall timeout for the `wait_*` helpers.
/// Generous: the fake claude runs through a pyenv shim (~0.9s startup).
pub const POLL: Duration = Duration::from_millis(100);
pub const TIMEOUT: Duration = Duration::from_secs(20);

/// Hang guard timeout for waiting on the fake claude process or its events.
/// A passing test should never be slowed by this; only a genuinely hung one waits longer.
pub const HANG_GUARD_TIMEOUT: Duration = Duration::from_secs(60);

/// A fake `~/.bridle` for [`Overrides::bridle_home`], under a test's own
/// tempdir, so a daemon started in-process never reads the real machine's
/// `~/.bridle/config.toml` (it's read directly, not via `$BRIDLE_HOME`, so
/// tests can't isolate it by setting an env var without `unsafe`).
pub fn machine_home_dir(tmp: &Path) -> PathBuf {
    tmp.join("machine-home")
}

pub fn fake_claude_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../bridle-claude/tests/fake-claude.py")
        .canonicalize()
        .expect("fake-claude.py exists")
}

/// `git init` plus one commit, so worktrees can branch from `HEAD`.
pub async fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).expect("mkdir repo");
    run_git(dir, &["init", "-q"]).await;
    run_git(
        dir,
        &[
            "-c",
            "user.email=test@example.com",
            "-c",
            "user.name=Test",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            "init",
        ],
    )
    .await;
}

async fn run_git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
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

pub struct TestDaemon {
    pub running: RunningDaemon,
    pub client: Client,
    pub workspace: PathBuf,
    pub repo: PathBuf,
}

impl TestDaemon {
    /// A client authenticated with a freshly minted `external` token, to
    /// exercise non-human principals against the human-only endpoints.
    pub async fn external_client(&self, name: &str) -> Client {
        let created = self
            .client
            .create_token(&bridle_api::types::TokenCreateRequest {
                name: name.to_string(),
            })
            .await
            .expect("create external token");
        Client::new(self.running.url.clone(), Some(created.token))
    }

    /// A client authenticated as `agent_id`'s own token, to exercise its
    /// role's principal against role-gated endpoints (e.g. agent lifecycle).
    pub fn agent_client(&self, agent_id: &str) -> Client {
        let token_path = self
            .workspace
            .join(".bridle/agents")
            .join(agent_id)
            .join("token");
        let token = std::fs::read_to_string(token_path)
            .expect("agent token")
            .trim()
            .to_string();
        Client::new(self.running.url.clone(), Some(token))
    }
}

/// Starts a fresh temp repo + workspace + daemon, pointed at the fake
/// claude, with the registry disabled (each test is isolated by its own
/// tempdir; no need to touch `~/.bridle`).
pub async fn start_daemon(overrides: Option<Overrides>) -> (TestDaemon, tempfile::TempDir) {
    start_daemon_with_config(overrides, None).await
}

/// [`start_daemon`], with `config_toml` written to `<repo>/.bridle/config.toml`
/// first.
pub async fn start_daemon_with_config(
    overrides: Option<Overrides>,
    config_toml: Option<&str>,
) -> (TestDaemon, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo).await;
    if let Some(text) = config_toml {
        std::fs::create_dir_all(repo.join(".bridle")).expect("mkdir .bridle");
        std::fs::write(repo.join(".bridle/config.toml"), text).expect("write config");
    }
    let workspace = tmp.path().to_path_buf();

    let opts = ServeOptions {
        repo: repo.clone(),
        workspace: Some(workspace.clone()),
        project: None,
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = overrides.unwrap_or_else(|| Overrides {
        claude_program: fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        bridle_home: None,
        stall_check_interval: Duration::from_secs(3600),
        tracker_interval: Duration::from_millis(200),
        governor_interval: Duration::from_millis(200),
        governor_poll_interval_normal: Duration::ZERO,
        governor_poll_interval_above_hold: Duration::ZERO,
        task_flush_interval: Duration::from_secs(3600),
        claim_lease_check_interval: Duration::from_secs(3600),
    });
    // Never read the real machine's ~/.bridle/config.toml: every test
    // daemon gets its own fake machine home, regardless of what the caller
    // passed in.
    overrides.bridle_home = Some(machine_home_dir(tmp.path()));
    let running = bridle_daemon::start(opts, overrides)
        .await
        .expect("start daemon");

    let token_path = workspace.join(".bridle/tokens/human");
    let token = std::fs::read_to_string(&token_path)
        .expect("human token file")
        .trim()
        .to_string();
    let client = Client::new(running.url.clone(), Some(token));

    (
        TestDaemon {
            running,
            client,
            workspace,
            repo,
        },
        tmp,
    )
}

/// Polls `f` until it returns `Some`, or panics after [`TIMEOUT`].
pub async fn wait_for<T, F, Fut>(what: &str, mut f: F) -> T
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(v) = f().await {
            return v;
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for {what}");
        }
        tokio::time::sleep(POLL).await;
    }
}

pub async fn wait_for_state(client: &Client, id: &str, state: AgentState) -> Agent {
    wait_for(&format!("agent {id} to reach {state:?}"), || async {
        let a = client.get_agent(id).await.ok()?;
        (a.state == state).then_some(a)
    })
    .await
}

pub async fn wait_for_agent(client: &Client, id: &str, pred: impl Fn(&Agent) -> bool) -> Agent {
    wait_for("agent predicate", || async {
        let a = client.get_agent(id).await.ok()?;
        pred(&a).then_some(a)
    })
    .await
}

/// Waits for an event of `kind` for `agent` (or any agent, if `None`)
/// matching `pred`, polling `GET /v1/events`.
pub async fn wait_for_event(
    client: &Client,
    kind: &str,
    agent: Option<&str>,
    pred: impl Fn(&Event) -> bool,
) -> Event {
    wait_for(&format!("event {kind}"), || async {
        let events = client
            .events(&EventQuery {
                since: None,
                agent: agent.map(str::to_string),
                kind: Some(kind.to_string()),
                limit: None,
            })
            .await
            .ok()?;
        events.into_iter().find(|e| pred(e))
    })
    .await
}
