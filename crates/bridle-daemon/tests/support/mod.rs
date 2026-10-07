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

/// The per-poll wait for the `wait_*` helpers.
pub const POLL: Duration = Duration::from_millis(100);

/// Hang guard for every wait on the fake claude, its events, or a condition.
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
    run_git(dir, &["init", "-q", "-b", "main"]).await;
    // Repo-local identity (worktrees share it), so the daemon's own commits work on a machine
    // with no global git config, as on CI.
    run_git(dir, &["config", "user.email", "test@example.com"]).await;
    run_git(dir, &["config", "user.name", "Test"]).await;
    run_git(dir, &["commit", "--allow-empty", "-q", "-m", "init"]).await;
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
                machine: None,
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

/// A wrapper script around fake-claude.py that sets `FAKE_CLAUDE_ARGV_FILE`
/// before exec'ing it, so a test can inspect the real invocation's argv
/// (e.g. `--allowedTools`) without fake-claude dropping a file into the
/// agent's own worktree on every run.
pub fn fake_claude_argv_dump_wrapper(dir: &Path, argv_path: &Path) -> PathBuf {
    let wrapper = dir.join("fake-claude-argv-wrapper.sh");
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nexec env FAKE_CLAUDE_ARGV_FILE={:?} {:?} \"$@\"\n",
            argv_path.display(),
            fake_claude_path().display(),
        ),
    )
    .expect("write wrapper script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
            .expect("chmod wrapper script");
    }
    wrapper
}

/// A wrapper script around fake-claude.py that sets
/// `FAKE_CLAUDE_SESSIONS_DIR`, so `--resume` of a session no process has run
/// dies like real claude's.
pub fn fake_claude_sessions_wrapper(dir: &Path) -> PathBuf {
    let sessions = dir.join("sessions");
    std::fs::create_dir_all(&sessions).expect("create sessions dir");
    let wrapper = dir.join("fake-claude-sessions-wrapper.sh");
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nexec env FAKE_CLAUDE_SESSIONS_DIR={:?} {:?} \"$@\"\n",
            sessions.display(),
            fake_claude_path().display(),
        ),
    )
    .expect("write wrapper script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
            .expect("chmod wrapper script");
    }
    wrapper
}

/// A wrapper script around fake-claude.py that sets `FAKE_CLAUDE_ENV_FILE`
/// before exec'ing it, so a test can inspect the real invocation's own
/// environment (e.g. a per-spawn `--env` secret).
pub fn fake_claude_env_dump_wrapper(dir: &Path, env_path: &Path) -> PathBuf {
    let wrapper = dir.join("fake-claude-env-wrapper.sh");
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nexec env FAKE_CLAUDE_ENV_FILE={:?} {:?} \"$@\"\n",
            env_path.display(),
            fake_claude_path().display(),
        ),
    )
    .expect("write wrapper script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
            .expect("chmod wrapper script");
    }
    wrapper
}

/// A wrapper script around fake-claude.py that sets both
/// `FAKE_CLAUDE_ARGV_FILE` and `FAKE_CLAUDE_ENV_FILE`, so a test can assert
/// on a single invocation's argv and environment together (e.g. that a
/// renew or resume still carries a spawn's `--allow-tool`/`--env`).
pub fn fake_claude_argv_and_env_dump_wrapper(
    dir: &Path,
    argv_path: &Path,
    env_path: &Path,
) -> PathBuf {
    let wrapper = dir.join("fake-claude-argv-env-wrapper.sh");
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nexec env FAKE_CLAUDE_ARGV_FILE={:?} FAKE_CLAUDE_ENV_FILE={:?} {:?} \"$@\"\n",
            argv_path.display(),
            env_path.display(),
            fake_claude_path().display(),
        ),
    )
    .expect("write wrapper script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
            .expect("chmod wrapper script");
    }
    wrapper
}

pub fn default_overrides() -> Overrides {
    Overrides {
        claude_program: fake_claude_path().to_string_lossy().into_owned(),
        write_registry: false,
        bridle_home: None,
        stall_check_interval: Duration::from_secs(3600),
        tracker_interval: Duration::from_millis(200),
        governor_interval: Duration::from_millis(200),
        governor_poll_interval_normal: Duration::ZERO,
        governor_poll_interval_above_hold: Duration::ZERO,
        usage_http: bridle_daemon::usage_http::UsageHttp {
            url: "http://127.0.0.1:9/".to_string(),
            token: Some("test".to_string()),
        },
        task_flush_interval: Duration::from_secs(3600),
        claim_lease_check_interval: Duration::from_secs(3600),
        port_check_interval: Duration::from_secs(3600),
        upgrade: Default::default(),
        ci_tick_interval: Duration::from_secs(3600),
        drain_wake_after: Duration::from_secs(600),
        settle_wake_interval: Duration::from_secs(3600),
        doc_watch_interval: Duration::from_secs(3600),
        queue_nudge_debounce: Duration::from_secs(3600),
        open_watch_debounce: std::time::Duration::from_secs(3600),
        take_over: false,
        host: None,
    }
}

/// [`start_daemon`], with `config_toml` written to `<repo>/.bridle/config.toml`
/// first. The built-in manager autostarts by default, which would put an extra
/// agent in every test's count, so it's switched off here unless the config
/// has its own `[roles.manager]`; [`start_daemon_verbatim_config`] keeps the
/// defaults.
pub async fn start_daemon_with_config(
    overrides: Option<Overrides>,
    config_toml: Option<&str>,
) -> (TestDaemon, tempfile::TempDir) {
    let mut text = config_toml.unwrap_or("").to_string();
    if !text.contains("[roles.manager]") {
        text.push_str("\n[roles.manager]\nautostart = false\n");
    }
    // Tests that build and start tasks at once don't wait out the settle
    // period; the ones that exercise it set `[tasks] settle` themselves.
    if !text.contains("settle") {
        match text.find("[tasks]") {
            Some(at) => text.insert_str(at + "[tasks]".len(), "\nsettle = \"0\""),
            None => text.push_str("\n[tasks]\nsettle = \"0\"\n"),
        }
    }
    start_daemon_verbatim_config(overrides, Some(&text)).await
}

/// [`start_daemon_with_config`] without the manager opt-out: the config is
/// written as given, so the built-in defaults apply.
pub async fn start_daemon_verbatim_config(
    overrides: Option<Overrides>,
    config_toml: Option<&str>,
) -> (TestDaemon, tempfile::TempDir) {
    start_daemon_named(None, overrides, config_toml).await
}

/// [`start_daemon_verbatim_config`] with an explicit project name, for tests that run two daemons.
pub async fn start_daemon_named(
    project: Option<&str>,
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
        project: project.map(str::to_string),
        listen: Some("127.0.0.1:0".parse().expect("valid addr")),
    };
    let mut overrides = overrides.unwrap_or_else(default_overrides);
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

/// Polls `f` until it returns `Some`, or panics after [`HANG_GUARD_TIMEOUT`].
pub async fn wait_for<T, F, Fut>(what: &str, mut f: F) -> T
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let deadline = Instant::now() + HANG_GUARD_TIMEOUT;
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

/// Reads a fake-claude dump written by [`fake_claude_argv_dump_wrapper`],
/// [`fake_claude_env_dump_wrapper`], or [`fake_claude_argv_and_env_dump_wrapper`].
/// Each invocation (including the daemon's own governor probes, which reuse
/// the same overridden `claude_program`) writes its own `<path>.<pid>` file
/// rather than sharing one, so a probe can never overwrite the dump a test is
/// waiting to read; this scans every such file for one whose parsed contents
/// satisfy `pred`, retrying (via [`wait_for`]) until one shows up.
pub async fn wait_for_dump<T, F>(what: &str, path: &Path, pred: F) -> T
where
    T: serde::de::DeserializeOwned,
    F: Fn(&T) -> bool,
{
    wait_for(what, || async { read_dump(path, &pred) }).await
}

fn read_dump<T, F>(path: &Path, pred: &F) -> Option<T>
where
    T: serde::de::DeserializeOwned,
    F: Fn(&T) -> bool,
{
    let dir = path.parent().expect("dump path has a parent dir");
    let prefix = format!(
        "{}.",
        path.file_name()
            .expect("dump path has a file name")
            .to_string_lossy()
    );
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        if !entry.file_name().to_string_lossy().starts_with(&prefix) {
            continue;
        }
        if let Some(v) = std::fs::read_to_string(entry.path())
            .ok()
            .and_then(|s| serde_json::from_str::<T>(&s).ok())
            .filter(|v| pred(v))
        {
            return Some(v);
        }
    }
    None
}

/// Removes every per-invocation dump file left by earlier `claude`
/// invocations under `path` (see [`wait_for_dump`]), so a later
/// [`wait_for_dump`] on the same `path` can't pass on a stale file from
/// before a stop/renew and must observe a fresh invocation's dump.
pub fn clear_dump(path: &Path) {
    let dir = path.parent().expect("dump path has a parent dir");
    let prefix = format!(
        "{}.",
        path.file_name()
            .expect("dump path has a file name")
            .to_string_lossy()
    );
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let _ = std::fs::remove_file(entry.path());
        }
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

/// Tasks are created `pending`; most tests want one the PM could plan.
pub trait ClientExt {
    async fn new_open_task(
        &self,
        req: &bridle_api::types::NewTaskRequest,
    ) -> Result<bridle_api::types::Task, bridle_api::ClientError>;
}

impl ClientExt for Client {
    async fn new_open_task(
        &self,
        req: &bridle_api::types::NewTaskRequest,
    ) -> Result<bridle_api::types::Task, bridle_api::ClientError> {
        let task = self.new_task(req).await?;
        if req.for_human {
            return Ok(task);
        }
        self.ready_task(&task.id).await
    }
}
