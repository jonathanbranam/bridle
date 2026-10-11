//! End-to-end test of the real `bridle` binary against a real (but
//! fake-claude-backed) daemon: `serve` as a child process, then a sequence
//! of CLI subcommands against it, then `serve --detach` + `daemons`. No
//! real `claude` is ever spawned.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

fn bridle_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bridle"))
}

/// The fake claude, launched as one exec of the real interpreter. `#!/usr/bin/env python3`
/// resolves through the pyenv shim (bash, bash, python: three execs per fake agent), so this
/// asks the interpreter once for its own path and runs a copy of the script whose shebang names
/// it. Falls back to the script itself when no interpreter answers (br-yw8b).
fn fake_claude_path() -> PathBuf {
    static PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../bridle-claude/tests/fake-claude.py")
            .canonicalize()
            .expect("fake-claude.py exists");
        direct_launcher(&script).unwrap_or(script)
    })
    .clone()
}

fn direct_launcher(script: &Path) -> Option<PathBuf> {
    use std::hash::{Hash, Hasher};
    let out = std::process::Command::new("python3")
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .ok()?;
    let interpreter = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if !out.status.success() || !Path::new(&interpreter).is_absolute() {
        return None;
    }
    let body = std::fs::read_to_string(script).ok()?;
    let rest = body.strip_prefix("#!")?.split_once('\n')?.1;
    let content = format!("#!{interpreter}\n{rest}");
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    let dir = std::env::temp_dir().join("bridle-fake-claude");
    std::fs::create_dir_all(&dir).ok()?;
    let target = dir.join(format!("fake-claude-{:016x}.py", hasher.finish()));
    if !target.exists() {
        // Write-then-rename so a parallel test binary never execs a half-written file.
        let tmp = dir.join(format!(".tmp-{}", std::process::id()));
        std::fs::write(&tmp, &content).ok()?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).ok()?;
        std::fs::rename(&tmp, &target).ok()?;
    }
    Some(target)
}

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).expect("mkdir repo");
    let run = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("run git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(&["init", "-q", "-b", "main"]);
    run(&[
        "-c",
        "user.email=test@example.com",
        "-c",
        "user.name=Test",
        "commit",
        "--allow-empty",
        "-q",
        "-m",
        "init",
    ]);
}

/// This test suite may itself be running under bridle (as one of its own
/// agents), which sets these in its own environment. The `bridle` child
/// processes spawned here must not inherit them — otherwise they'd talk to
/// that real daemon instead of the one this test starts, as happened once
/// (see docs/tickets/open/v1-follow-ups-from-the-build-9c6e.md).
/// Mirrors the CLAUDE*/BRIDLE_TOKEN stripping in
/// `bridle_claude::command::env_removal_keys`, extended to every BRIDLE_*
/// var the CLI itself reads for discovery.
fn strip_bridle_env(cmd: &mut Command) -> &mut Command {
    cmd.env_remove("CLAUDECODE")
        .env_remove("BRIDLE_URL")
        .env_remove("BRIDLE_TOKEN")
        .env_remove("BRIDLE_AGENT_ID")
        .env_remove("BRIDLE_AGENT_NAME")
        .env_remove("BRIDLE_PROJECT")
}

/// Hang guard for every wait in this file: a passing test never waits this
/// long, only a genuinely hung one does. Waits are on conditions, never on
/// elapsed time, so a loaded machine can't fail a test that would pass.
const HANG_GUARD: Duration = Duration::from_secs(180);

/// Polls `f` until it returns `Some`, or panics after [`HANG_GUARD`].
fn wait_until<T>(what: &str, mut f: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + HANG_GUARD;
    loop {
        if let Some(v) = f() {
            return v;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Waits for a daemon we spawned to write `daemon.json`, failing at once if it
/// exited instead: a daemon that died at start-up (its stderr is inherited, so
/// nextest shows why) never writes the file, and polling on would just burn
/// the whole hang guard before reporting a misleading timeout.
fn wait_for_daemon(child: &mut Child, daemon_json: &Path) {
    wait_until(&daemon_json.display().to_string(), || {
        if let Some(status) = child.try_wait().expect("poll daemon") {
            panic!(
                "daemon exited ({status}) before writing {}",
                daemon_json.display()
            );
        }
        daemon_json.is_file().then_some(())
    });
}

/// Waits for w1's first turn to complete (polls `agents --all --json`).
fn wait_for_first_turn(cwd: &Path, home: &Path) {
    wait_until("w1's first turn to finish", || {
        let (ok, out, err) = run_cli(cwd, home, &["agents", "--all", "--json"]);
        assert!(ok, "agents failed: {err}");
        let agents: serde_json::Value = serde_json::from_str(&out).expect("agents json");
        let w1 = agents
            .as_array()
            .expect("array")
            .iter()
            .find(|a| a["name"] == "w1")
            .expect("w1 present");
        (w1["turns"].as_u64().unwrap_or(0) >= 1 && w1["state"] == "idle").then_some(())
    });
}

/// Runs the real `bridle` binary and returns (succeeded, stdout, stderr).
fn run_cli(cwd: &Path, home: &Path, args: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new(bridle_bin());
    cmd.args(args)
        .current_dir(cwd)
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", home);
    strip_bridle_env(&mut cmd);
    let out = cmd.output().expect("run bridle");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Kills and reaps the child on drop, so a failed assertion doesn't leak a
/// background daemon process.
struct DaemonGuard(Child);

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Same purpose as [`DaemonGuard`], for `serve --detach`: the CLI process we
/// run is only the launcher, and exits once the real daemon (re-exec'd into
/// its own process group) reports healthy, so there's no `Child` to hold. A
/// panicking assertion between detach and the test's own `stop-daemon` call
/// would otherwise leak that daemon against a tempdir that's about to be
/// deleted.
struct DetachedDaemonGuard(i32);

impl Drop for DetachedDaemonGuard {
    fn drop(&mut self) {
        let _ = kill(Pid::from_raw(self.0), Signal::SIGKILL);
    }
}

#[test]
fn cli_end_to_end_against_a_foreground_daemon() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    // No `--workspace`: it defaults to the repo's parent, so `repo` is a
    // child of the workspace and the CLI's cwd-walk discovery (from `repo`)
    // finds `.bridle/daemon.json` there.
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    // `spawn worker --name w1 --prompt hi --json`
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "spawn", "worker", "--name", "w1", "--prompt", "hi", "--json",
        ],
    );
    assert!(ok, "spawn failed: {err}");
    let agent: serde_json::Value = serde_json::from_str(&out).expect("agent json");
    assert_eq!(agent["name"], "w1");
    let agent_id = agent["id"].as_str().expect("agent id").to_string();

    // Wait for the first turn to complete (poll `agents --all --json`).
    wait_for_first_turn(&repo, &home);

    // `send w1 hello --json`
    let (ok, out, err) = run_cli(&repo, &home, &["send", "w1", "hello", "--json"]);
    assert!(ok, "send failed: {err}");
    let msgs: serde_json::Value = serde_json::from_str(&out).expect("message json");
    assert_eq!(msgs[0]["to"], agent_id);

    // `logs w1` (readable rendering; just check it runs and prints something).
    let (ok, out, err) = run_cli(&repo, &home, &["logs", "w1"]);
    assert!(ok, "logs failed: {err}");
    assert!(!out.trim().is_empty(), "expected some rendered log output");

    // `status --json`
    let (ok, out, err) = run_cli(&repo, &home, &["status", "--json"]);
    assert!(ok, "status failed: {err}");
    let status: serde_json::Value = serde_json::from_str(&out).expect("status json");
    assert_eq!(status["principal"], "human");

    // `stop w1`
    let (ok, out, err) = run_cli(&repo, &home, &["stop", "w1", "--json"]);
    assert!(ok, "stop failed: {err}");
    let stopped: serde_json::Value = serde_json::from_str(&out).expect("stopped agent json");
    assert_eq!(stopped["state"], "stopped");

    // `rm w1`
    let (ok, _out, err) = run_cli(&repo, &home, &["rm", "w1"]);
    assert!(ok, "rm failed: {err}");

    // `token create orchestrator --json`
    let (ok, out, err) = run_cli(&repo, &home, &["token", "create", "orchestrator", "--json"]);
    assert!(ok, "token create failed: {err}");
    let created: serde_json::Value = serde_json::from_str(&out).expect("token created json");
    assert_eq!(created["principal"], "external:orchestrator");

    // `token list --json`
    let (ok, out, err) = run_cli(&repo, &home, &["token", "list", "--json"]);
    assert!(ok, "token list failed: {err}");
    let tokens: serde_json::Value = serde_json::from_str(&out).expect("token list json");
    let orchestrator = tokens
        .as_array()
        .expect("array")
        .iter()
        .find(|t| t["name"] == "orchestrator")
        .expect("orchestrator token listed");
    assert_eq!(orchestrator["revoked"], false);
    assert!(
        orchestrator.get("token").is_none(),
        "token list must never carry the secret: {orchestrator}"
    );

    // `token revoke orchestrator`
    let (ok, _out, err) = run_cli(&repo, &home, &["token", "revoke", "orchestrator"]);
    assert!(ok, "token revoke failed: {err}");

    let (ok, out, err) = run_cli(&repo, &home, &["token", "list", "--json"]);
    assert!(ok, "token list after revoke failed: {err}");
    let tokens: serde_json::Value = serde_json::from_str(&out).expect("token list json");
    let orchestrator = tokens
        .as_array()
        .expect("array")
        .iter()
        .find(|t| t["name"] == "orchestrator")
        .expect("orchestrator token still listed");
    assert_eq!(orchestrator["revoked"], true);

    // `stop-daemon`
    let (ok, out, err) = run_cli(&repo, &home, &["stop-daemon"]);
    assert!(ok, "stop-daemon failed: {err}");
    assert!(
        out.contains("requested shutdown") && out.contains("shutdown complete"),
        "{out}"
    );
    assert!(
        !daemon_json.exists(),
        "daemon.json should be removed on clean shutdown"
    );

    let status = guard
        .0
        .wait_or_kill()
        .expect("daemon process should exit after stop-daemon");
    assert!(status.success(), "daemon exited with {status:?}");
}

/// Ticket 9c63's client-side half: a read command run as a plain Claude Code
/// session (`$CLAUDECODE` set, no `$BRIDLE_TOKEN`) must not error out before
/// even sending a request — it should succeed, unauthenticated, against the
/// daemon's own tolerance for a token-less `GET` (docs/design/agent-host/
/// principals.md, "Read access without a token").
#[test]
fn read_command_succeeds_with_no_token_when_claudecode_is_set() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    // Same as `run_cli`, but with `$CLAUDECODE` set and no `$BRIDLE_TOKEN` —
    // the case that used to fail client-side (crates/bridle-api/src/
    // discovery.rs's `resolve_token`) before any request went out.
    let mut cmd = Command::new(bridle_bin());
    cmd.args(["status", "--json"])
        .current_dir(&repo)
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home);
    strip_bridle_env(&mut cmd);
    cmd.env("CLAUDECODE", "1");
    let out = cmd.output().expect("run bridle status");
    assert!(
        out.status.success(),
        "status with CLAUDECODE set and no token should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let status: serde_json::Value = serde_json::from_slice(&out.stdout).expect("status json");
    assert_eq!(status["principal"], "local");
}

#[test]
fn serve_detach_returns_once_healthy_and_daemons_lists_it() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    // The manager autostarts by default; this test counts agents.
    std::fs::create_dir_all(repo.join(".bridle")).expect("mkdir .bridle");
    std::fs::write(
        repo.join(".bridle/config.toml"),
        "[roles.manager]\nautostart = false\n",
    )
    .expect("write config");
    let workspace = tmp.path().join("ws");
    let home = tmp.path().join("home");

    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "--json",
            "serve",
            "--repo",
            repo.to_str().expect("utf8 path"),
            "--workspace",
            workspace.to_str().expect("utf8 path"),
            "--listen",
            "127.0.0.1:0",
            "--detach",
        ],
    );
    assert!(ok, "serve --detach failed: {err}");
    let info: serde_json::Value = serde_json::from_str(&out).expect("detach json");
    let _guard = DetachedDaemonGuard(info["pid"].as_i64().expect("pid") as i32);
    assert!(
        info["url"].as_str().expect("url").starts_with("http://"),
        "{info}"
    );

    // A second, stale-looking registry entry: alive pid (this test process's
    // own), but nothing listening on its port, so `daemons` can't reach its
    // `/v1/health` and must show `agents: null` rather than hang or fail.
    let dead_registry_dir = home.join("daemons");
    std::fs::create_dir_all(&dead_registry_dir).expect("mkdir registry dir");
    std::fs::write(
        dead_registry_dir.join("unreachable.json"),
        format!(
            r#"{{"project":"unreachable","workspace":"/tmp/nowhere","repo":"/tmp/nowhere",
                "url":"http://127.0.0.1:1","pid":{},"started_at":"2026-01-01T00:00:00Z",
                "version":"0.0.0"}}"#,
            std::process::id()
        ),
    )
    .expect("write stale registry entry");

    let (ok, out, err) = run_cli(&repo, &home, &["daemons", "--json"]);
    assert!(ok, "daemons failed: {err}");
    let list: serde_json::Value = serde_json::from_str(&out).expect("daemons json");
    let ws_str = workspace.to_string_lossy().into_owned();
    let entry = list
        .as_array()
        .expect("array")
        .iter()
        .find(|d| d["workspace"] == ws_str)
        .unwrap_or_else(|| panic!("expected the detached daemon in the registry: {list}"));
    assert_eq!(
        entry["agents"], 0,
        "no agents spawned yet, and health answered: {entry}"
    );

    let unreachable = list
        .as_array()
        .expect("array")
        .iter()
        .find(|d| d["project"] == "unreachable")
        .unwrap_or_else(|| panic!("expected the stale entry in the registry: {list}"));
    assert!(
        unreachable["agents"].is_null(),
        "unreachable daemon should show no agent count: {unreachable}"
    );

    // This daemon's workspace isn't an ancestor of `repo` (explicit
    // `--workspace`), so discover it from inside the workspace instead.
    let (ok, _out, err) = run_cli(&workspace, &home, &["stop-daemon"]);
    assert!(ok, "stop-daemon failed: {err}");
}

/// SIGINT must shut the daemon down cleanly, not panic it: regression test
/// for the `spawn_blocking().await.expect(...)` panics in `Store::open`/
/// `with_conn`, which turned an ordinary `JoinError::Cancelled` (queued
/// blocking store tasks dropped during shutdown) into a real panic.
#[test]
fn sigint_shuts_down_cleanly_while_idle() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    strip_bridle_env(&mut serve_cmd);
    let mut child = serve_cmd.spawn().expect("spawn bridle serve");
    let pid = child.id();
    let mut stderr = child.stderr.take().expect("piped stderr");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    kill(Pid::from_raw(pid as i32), Signal::SIGINT).expect("send SIGINT");

    let status = guard
        .0
        .wait_or_kill()
        .expect("daemon process should exit after SIGINT");

    let mut stderr_text = String::new();
    stderr
        .read_to_string(&mut stderr_text)
        .expect("read daemon stderr");

    assert!(status.success(), "daemon exited with {status:?}");
    assert!(
        !stderr_text.contains("panicked"),
        "daemon panicked on shutdown:\n{stderr_text}"
    );
    assert!(
        !daemon_json.exists(),
        "daemon.json should be removed on clean shutdown"
    );
}

/// Same as above, but sends SIGINT immediately after spawning an agent
/// (before its first turn completes), so a store call has a real chance of
/// being in flight when the shutdown-triggered blocking-task cancellation
/// happens.
#[test]
fn sigint_shuts_down_cleanly_with_a_store_call_in_flight() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");
    // Spawns 8 workers, more than the default max_workers.
    std::fs::create_dir_all(repo.join(".bridle")).expect("mkdir .bridle");
    std::fs::write(
        repo.join(".bridle/config.toml"),
        "[budget]\nmax_workers = 8\n",
    )
    .expect("write config");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    strip_bridle_env(&mut serve_cmd);
    let mut child = serve_cmd.spawn().expect("spawn bridle serve");
    let pid = child.id();
    let mut stderr = child.stderr.take().expect("piped stderr");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    for i in 0..8 {
        let (ok, _out, err) = run_cli(
            &repo,
            &home,
            &[
                "spawn",
                "worker",
                "--name",
                &format!("w{i}"),
                "--prompt",
                "hi",
            ],
        );
        assert!(ok, "spawn failed: {err}");
    }

    kill(Pid::from_raw(pid as i32), Signal::SIGINT).expect("send SIGINT");

    // Stopping 8 agents concurrently is capped at `stop_grace` (30s) + 5s
    // by the daemon itself (see `lib.rs`'s shutdown sequence); give it that
    // much plus real margin for a loaded machine (measured up to ~42s with
    // several full `cargo nextest run --workspace` runs going at once)
    // rather than the idle-daemon 10s used elsewhere in this file, so a
    // busy machine doesn't get SIGKILLed mid-shutdown.
    let status = guard
        .0
        .wait_or_kill()
        .expect("daemon process should exit after SIGINT");

    let mut stderr_text = String::new();
    stderr
        .read_to_string(&mut stderr_text)
        .expect("read daemon stderr");

    assert!(status.success(), "daemon exited with {status:?}");
    assert!(
        !stderr_text.contains("panicked"),
        "daemon panicked on shutdown:\n{stderr_text}"
    );
}

/// Test `bridle inbox show <id>` and `bridle inbox read <id>`.
#[test]
fn inbox_show_and_read_commands() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    // Spawn a worker
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "spawn", "worker", "--name", "w1", "--prompt", "hi", "--json",
        ],
    );
    assert!(ok, "spawn failed: {err}");
    let _: serde_json::Value = serde_json::from_str(&out).expect("agent json");

    // Wait for the first turn to complete
    wait_for_first_turn(&repo, &home);

    // Send a message from w1 to human
    let (ok, out, err) = run_cli(&repo, &home, &["send", "human", "hello", "--json"]);
    assert!(ok, "send failed: {err}");
    let msgs: serde_json::Value = serde_json::from_str(&out).expect("message json");
    let msg_id = msgs[0]["id"].as_str().expect("message id").to_string();

    // Test `bridle inbox show <id> --mark-read`
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "show", &msg_id, "--mark-read"]);
    assert!(ok, "inbox show failed: {err}");
    assert!(
        out.contains("From:"),
        "expected From: header in output: {out}"
    );
    assert!(
        out.contains("hello"),
        "expected message body in output: {out}"
    );
    assert!(
        out.contains("To reply:"),
        "expected reply command in output: {out}"
    );

    // Verify message was marked read (check via json output)
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "--json"]);
    assert!(ok, "inbox failed: {err}");
    let inbox: serde_json::Value = serde_json::from_str(&out).expect("inbox json");
    let messages = inbox["messages"]
        .as_array()
        .expect("messages array")
        .iter()
        .filter(|m| m["id"] == msg_id)
        .collect::<Vec<_>>();
    assert!(
        messages.is_empty(),
        "message should be read and not appear in unread inbox"
    );

    // Send another message
    let (ok, out, err) = run_cli(&repo, &home, &["send", "human", "hello2", "--json"]);
    assert!(ok, "send failed: {err}");
    let msgs: serde_json::Value = serde_json::from_str(&out).expect("message json");
    let msg_id2 = msgs[0]["id"].as_str().expect("message id").to_string();

    // `bridle inbox show <id>` leaves the message unread by default
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "show", &msg_id2]);
    assert!(ok, "inbox show failed: {err}");
    assert!(
        out.contains("hello2"),
        "expected message body in output: {out}"
    );

    // Verify message is still unread
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "--json"]);
    assert!(ok, "inbox failed: {err}");
    let inbox: serde_json::Value = serde_json::from_str(&out).expect("inbox json");
    let messages = inbox["messages"]
        .as_array()
        .expect("messages array")
        .iter()
        .filter(|m| m["id"] == msg_id2)
        .collect::<Vec<_>>();
    assert!(
        !messages.is_empty(),
        "message should still be unread in inbox"
    );

    // Test `bridle inbox read <id>` to mark it read
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "read", &msg_id2]);
    assert!(ok, "inbox read failed: {err}");
    assert!(
        out.contains("marked") && out.contains("read"),
        "expected marked read output: {out}"
    );

    // Verify message is now read
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "--json"]);
    assert!(ok, "inbox failed: {err}");
    let inbox: serde_json::Value = serde_json::from_str(&out).expect("inbox json");
    let messages = inbox["messages"]
        .as_array()
        .expect("messages array")
        .iter()
        .filter(|m| m["id"] == msg_id2)
        .collect::<Vec<_>>();
    assert!(
        messages.is_empty(),
        "message should be read and not appear in unread inbox"
    );

    // Test `bridle inbox read` with multiple ids
    let (ok, out, err) = run_cli(&repo, &home, &["send", "human", "msg3", "--json"]);
    assert!(ok, "send failed: {err}");
    let msgs: serde_json::Value = serde_json::from_str(&out).expect("message json");
    let msg_id3 = msgs[0]["id"].as_str().expect("message id").to_string();

    let (ok, out, err) = run_cli(&repo, &home, &["send", "human", "msg4", "--json"]);
    assert!(ok, "send failed: {err}");
    let msgs: serde_json::Value = serde_json::from_str(&out).expect("message json");
    let msg_id4 = msgs[0]["id"].as_str().expect("message id").to_string();

    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "read", &msg_id3, &msg_id4]);
    assert!(ok, "inbox read multiple failed: {err}");
    assert!(
        out.contains(&msg_id3) && out.contains(&msg_id4),
        "expected both ids in output: {out}"
    );

    // Verify both messages are marked read
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "--json"]);
    assert!(ok, "inbox failed: {err}");
    let inbox: serde_json::Value = serde_json::from_str(&out).expect("inbox json");
    let messages = inbox["messages"]
        .as_array()
        .expect("messages array")
        .iter()
        .filter(|m| m["id"] == msg_id3 || m["id"] == msg_id4)
        .collect::<Vec<_>>();
    assert!(
        messages.is_empty(),
        "both messages should be read and not appear in unread inbox"
    );

    let _ = guard.0.kill();
}

/// A small extension so the foreground-daemon tests can wait for the child
/// to exit after asking it to stop, without pulling in a whole
/// process-management crate for one call. Only [`HANG_GUARD`] bounds it, and
/// a child still running then is killed so it can't outlive the test.
trait WaitOrKill {
    fn wait_or_kill(&mut self) -> std::io::Result<std::process::ExitStatus>;
}

impl WaitOrKill for Child {
    fn wait_or_kill(&mut self) -> std::io::Result<std::process::ExitStatus> {
        let deadline = Instant::now() + HANG_GUARD;
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                let _ = self.kill();
                return self.wait();
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

const GOOD_SPEC: &str = "## Purpose\n\nx\n\n## Requirements\n\n### Requirement: Works {#r-7fa2}\n\nIt SHALL work.\n\n#### Scenario: Works {#s-7fa3}\n\n*Verification*: **executable**\n\n- **WHEN** it runs\n- **THEN** it works\n";
const NO_ID_SPEC: &str = "## Purpose\n\nx\n\n## Requirements\n\n### Requirement: Works\n\nIt SHALL work.\n\n#### Scenario: Works\n\n*Verification*: **executable**\n\n- **WHEN** it runs\n- **THEN** it works\n";

fn spec_dir(name: &str, text: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let specs = dir.path().join("design/specs");
    std::fs::create_dir_all(&specs).expect("mkdir");
    std::fs::write(specs.join(name), text).expect("write spec");
    dir
}

#[test]
fn spec_check_good_spec_passes() {
    let dir = spec_dir("a.md", GOOD_SPEC);
    let home = tempfile::tempdir().expect("home");
    let (ok, out, _) = run_cli(dir.path(), home.path(), &["spec", "check"]);
    assert!(ok, "{out}");
    assert!(
        out.contains("1 file(s) checked: 0 error(s), 0 warning(s)"),
        "{out}"
    );
}

#[test]
fn spec_check_bad_spec_prints_several_diagnostics_and_fails() {
    let dir = spec_dir(
        "bad.md",
        "## Requirements\n\n### Requirement: A {#nope}\n\ntext\n\n### Requirement: B {#r-7fa2 bogus}\n\ntext\n",
    );
    let home = tempfile::tempdir().expect("home");
    let (ok, out, _) = run_cli(dir.path(), home.path(), &["spec", "check"]);
    assert!(!ok);
    assert!(out.contains("bad.md:3:"), "{out}");
    assert!(out.contains("bad.md:7:"), "{out}");
    let errors = out.lines().filter(|l| l.contains("bad.md:")).count();
    assert!(errors >= 2, "{out}");
}

#[test]
fn spec_check_missing_ids_warn_unless_required() {
    let dir = spec_dir("a.md", NO_ID_SPEC);
    let home = tempfile::tempdir().expect("home");
    let (ok, out, _) = run_cli(dir.path(), home.path(), &["spec", "check"]);
    assert!(ok, "{out}");
    assert!(out.contains("warning: requirement"), "{out}");
    assert!(out.contains("0 error(s), 1 warning(s)"), "{out}");

    let (ok, out, _) = run_cli(dir.path(), home.path(), &["spec", "check", "--require-ids"]);
    assert!(!ok);
    assert!(out.contains("1 error(s), 0 warning(s)"), "{out}");
}

#[test]
fn spec_check_json_and_root_override() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("openspec/specs/x")).expect("mkdir");
    std::fs::write(dir.path().join("openspec/specs/x/spec.md"), NO_ID_SPEC).expect("write");
    let home = tempfile::tempdir().expect("home");
    let (ok, out, _) = run_cli(
        dir.path(),
        home.path(),
        &["--json", "spec", "check", "--root", "openspec/specs"],
    );
    assert!(ok, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["files"], 1);
    assert_eq!(v["warnings"], 1);
    assert_eq!(v["diagnostics"][0]["severity"], "warning");
    assert_eq!(v["diagnostics"][0]["column"], 1);
}

fn export_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spec-export")
}

#[test]
fn spec_export_gherkin_matches_golden_and_omits_non_executable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = tempfile::tempdir().expect("home");
    let specs = export_fixture().join("design/specs");
    let (ok, out, err) = run_cli(
        dir.path(),
        home.path(),
        &[
            "spec",
            "export",
            "--format",
            "gherkin",
            specs.to_str().expect("utf8"),
        ],
    );
    assert!(ok, "{out}{err}");
    // No --out: the default cache dir, under the cwd.
    let feature = dir.path().join(".bridle/cache/features/widgets.feature");
    let got = std::fs::read_to_string(&feature).expect("feature written");
    let want = std::fs::read_to_string(export_fixture().join("widgets.feature")).expect("golden");
    assert_eq!(got, want);
    assert!(got.contains("@s-b310") && !got.contains("s-b311"));
}

#[test]
fn spec_export_json_has_ids_and_every_scenario() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = tempfile::tempdir().expect("home");
    let (ok, out, err) = run_cli(
        dir.path(),
        home.path(),
        &[
            "spec",
            "export",
            "--format",
            "json",
            "--root",
            export_fixture()
                .join("design/specs")
                .to_str()
                .expect("utf8"),
        ],
    );
    assert!(ok, "{out}{err}");
    let got: serde_json::Value = serde_json::from_str(&out).expect("json on stdout");
    let mut want: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(export_fixture().join("widgets.json")).expect("golden"),
    )
    .expect("golden json");
    // `file` is the path as given, which differs per machine.
    want["specs"][0]["file"] = got["specs"][0]["file"].clone();
    assert_eq!(got, want);
}

fn exported_scenario_ids(json: &str) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_str(json).expect("json on stdout");
    v["specs"]
        .as_array()
        .expect("specs")
        .iter()
        .flat_map(|s| s["requirements"].as_array().expect("reqs"))
        .flat_map(|r| r["scenarios"].as_array().expect("scenarios"))
        .map(|s| s["id"].as_str().expect("id").to_string())
        .collect()
}

#[test]
fn spec_export_scenario_filter_selects_scenarios_and_requirements() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = tempfile::tempdir().expect("home");
    let specs = export_fixture().join("design/specs");
    let export = |extra: &[&str]| {
        let mut args = vec!["spec", "export", "--format", "json", "--root"];
        args.push(specs.to_str().expect("utf8"));
        args.extend_from_slice(extra);
        run_cli(dir.path(), home.path(), &args)
    };

    let (ok, out, err) = export(&["--scenario", "s-b312"]);
    assert!(ok, "{out}{err}");
    assert_eq!(exported_scenario_ids(&out), ["s-b312"]);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(
        v["specs"][0]["requirements"]
            .as_array()
            .expect("reqs")
            .len(),
        1
    );

    // A requirement id selects all its scenarios; repeats add up.
    let (ok, out, err) = export(&["--scenario", "r-7fa2", "--scenario", "s-b312"]);
    assert!(ok, "{out}{err}");
    assert_eq!(exported_scenario_ids(&out), ["s-b310", "s-b311", "s-b312"]);

    // Gherkin: the requirement with nothing selected is gone.
    let (ok, out, err) = run_cli(
        dir.path(),
        home.path(),
        &[
            "spec",
            "export",
            "--format",
            "gherkin",
            "--scenario",
            "s-b310",
            specs.to_str().expect("utf8"),
        ],
    );
    assert!(ok, "{out}{err}");
    let got = std::fs::read_to_string(dir.path().join(".bridle/cache/features/widgets.feature"))
        .expect("feature");
    assert!(
        got.contains("@s-b310") && !got.contains("Colouring"),
        "{got}"
    );
}

#[test]
fn spec_export_task_selects_the_tasks_impact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_guard, repo, home) = start_daemon(tmp.path());
    let specs = export_fixture().join("design/specs");
    let specs = specs.to_str().expect("utf8");

    let (ok, out, err) = run_cli(&repo, &home, &["task", "new", "t", "-k", "chore", "--json"]);
    assert!(ok, "task new failed: {err}");
    let id = serde_json::from_str::<serde_json::Value>(&out).expect("task json")["id"]
        .as_str()
        .expect("id")
        .to_string();

    let export = [
        "spec", "export", "--format", "json", "--root", specs, "--task",
    ];
    let with_task = |id: &str| {
        let mut args = export.to_vec();
        args.push(id);
        run_cli(&repo, &home, &args)
    };
    let (ok, _, err) = with_task(&id);
    assert!(!ok);
    assert!(err.contains("declares no impact"), "{err}");

    let (ok, _, err) = run_cli(
        &repo,
        &home,
        &[
            "impact",
            "set",
            &id,
            "--modify",
            "s-b312",
            "--add-under",
            "r-7fa2",
        ],
    );
    assert!(ok, "impact set failed: {err}");
    let (ok, out, err) = with_task(&id);
    assert!(ok, "{out}{err}");
    assert_eq!(exported_scenario_ids(&out), ["s-b310", "s-b311", "s-b312"]);
}

#[test]
fn spec_export_refuses_specs_with_errors() {
    let dir = spec_dir(
        "bad.md",
        "## Requirements\n\n### Requirement: A {#nope}\n\ntext\n",
    );
    let home = tempfile::tempdir().expect("home");
    let (ok, out, err) = run_cli(
        dir.path(),
        home.path(),
        &["spec", "export", "--format", "gherkin"],
    );
    assert!(!ok);
    assert!(err.contains("bad.md:3:"), "{out}{err}");
    assert!(!dir.path().join(".bridle/cache/features").exists());
}

fn start_daemon(tmp: &Path) -> (DaemonGuard, PathBuf, PathBuf) {
    let repo = tmp.join("repo");
    init_repo(&repo);
    let home = tmp.join("home");
    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let mut guard = DaemonGuard(serve_cmd.spawn().expect("spawn bridle serve"));
    wait_for_daemon(&mut guard.0, &tmp.join(".bridle/daemon.json"));
    (guard, repo, home)
}

fn spawn_cli(cwd: &Path, home: &Path, args: &[&str]) -> Child {
    let mut cmd = Command::new(bridle_bin());
    cmd.args(args)
        .current_dir(cwd)
        .env("BRIDLE_HOME", home)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    strip_bridle_env(&mut cmd);
    cmd.spawn().expect("spawn bridle")
}

#[test]
fn wait_returns_on_state_change_message_timeout_and_already_in_state() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_guard, repo, home) = start_daemon(tmp.path());

    let (ok, out, err) = run_cli(&repo, &home, &["task", "new", "t", "-k", "chore", "--json"]);
    assert!(ok, "task new failed: {err}");
    let id = serde_json::from_str::<serde_json::Value>(&out).expect("task json")["id"]
        .as_str()
        .expect("id")
        .to_string();

    // Times out: exit 4, distinct from success (1) and unreachable (3).
    let waiter = spawn_cli(&repo, &home, &["wait", &id, "--timeout", "1"]);
    let out = waiter.wait_with_output().expect("wait output");
    assert_eq!(out.status.code(), Some(4));

    // Returns on a state change.
    let waiter = spawn_cli(&repo, &home, &["wait", &id, "--until", "planned"]);
    std::thread::sleep(Duration::from_millis(800));
    let (ok, _, err) = run_cli(&repo, &home, &["task", "ready", &id]);
    assert!(ok, "ready failed: {err}");
    let (ok, _, err) = run_cli(&repo, &home, &["task", "plan", &id]);
    assert!(ok, "plan failed: {err}");
    let out = waiter.wait_with_output().expect("wait output");
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("{id} is planned")
    );

    // Already in the state: returns at once.
    let (ok, out, err) = run_cli(&repo, &home, &["wait", &id, "--until", "planned", "--json"]);
    assert!(ok, "wait failed: {err}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("wait json");
    assert_eq!(v["result"], "state");
    assert_eq!(v["state"], "planned");

    // --or-message returns on a message to the caller.
    let waiter = spawn_cli(&repo, &home, &["wait", &id, "--or-message", "--json"]);
    std::thread::sleep(Duration::from_millis(800));
    let (ok, _, err) = run_cli(&repo, &home, &["send", "human", "ping"]);
    assert!(ok, "send failed: {err}");
    let out = waiter.wait_with_output().expect("wait output");
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("wait json");
    assert_eq!(v["result"], "message");
}

fn arch_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("design/architecture");
    std::fs::create_dir_all(&root).expect("mkdir");
    for (name, text) in files {
        std::fs::write(root.join(name), text).expect("write");
    }
    dir
}

#[test]
fn arch_list_lists_and_filters_invariants() {
    let dir = arch_dir(&[
        ("a.md", "## Engine referees {#a-12cd invariant}\nx\n"),
        (
            "b.md",
            "## Use SQLite {#a-00ff}\ny\n\n**Alternatives rejected:** Postgres\n",
        ),
    ]);
    let home = tempfile::tempdir().expect("home");
    let (ok, out, _) = run_cli(dir.path(), home.path(), &["arch", "list"]);
    assert!(ok);
    assert!(out.contains("a-12cd invariant  Engine referees"), "{out}");
    assert!(out.contains("a-00ff  Use SQLite"), "{out}");
    let (ok, out, _) = run_cli(
        dir.path(),
        home.path(),
        &["--json", "arch", "list", "--invariants"],
    );
    assert!(ok);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v.as_array().map(Vec::len), Some(1));
    assert_eq!(v[0]["id"], "a-12cd");
}

#[test]
fn arch_list_rejects_duplicate_and_missing_ids() {
    let dir = arch_dir(&[
        ("a.md", "## X {#a-12cd}\n## No id\n"),
        ("b.md", "## Y {#a-12cd}\n"),
    ]);
    let home = tempfile::tempdir().expect("home");
    let (ok, _, err) = run_cli(dir.path(), home.path(), &["arch", "list"]);
    assert!(!ok);
    assert!(err.contains("a.md:2:9"), "{err}");
    assert!(err.contains("duplicate id"), "{err}");
}

#[test]
fn explore_new_check_conclude_abandon() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = tempfile::tempdir().expect("home");
    let run = |args: &[&str]| run_cli(dir.path(), home.path(), args);
    assert!(run(&["explore", "new", "tw-e41a"]).0);
    let (ok, _, err) = run(&["explore", "new", "tw-e41a"]);
    assert!(!ok && err.contains("already exists"), "{err}");
    assert!(run(&["explore", "check"]).0);
    let doc = dir.path().join("design/explore/tw-e41a/findings.md");
    let before = std::fs::read_to_string(&doc).expect("read");
    assert!(run(&["explore", "conclude", "tw-e41a"]).0);
    let after = std::fs::read_to_string(&doc).expect("read");
    assert_eq!(after, before.replace("status: open", "status: concluded"));
    assert!(run(&["explore", "abandon", "tw-e41a"]).0);
    std::fs::write(&doc, before.replace("status: open", "status: nope")).expect("write");
    let (ok, out, _) = run(&["explore", "check"]);
    assert!(!ok && out.contains("found 'nope'"), "{out}");
}

fn goals_fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/goals")
        .join(name)
}

/// A temp dir whose `design/goals` holds the named fixtures.
fn goals_dir(names: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let goals = dir.path().join("design/goals");
    std::fs::create_dir_all(&goals).expect("mkdir");
    for n in names {
        std::fs::copy(goals_fixture(n), goals.join(n)).expect("copy fixture");
    }
    dir
}

#[test]
fn goals_list_prints_defaults_and_filters() {
    let dir = goals_dir(&["good.md"]);
    let home = tempfile::tempdir().expect("home");
    let (ok, out, err) = run_cli(dir.path(), home.path(), &["goals", "list"]);
    assert!(ok, "{out}{err}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3, "{out}");
    assert!(
        lines[0].contains("g-03") && lines[0].contains("unaddressed"),
        "{out}"
    );
    assert!(
        lines[1].contains("g-04") && lines[1].contains("build"),
        "{out}"
    );
    assert!(
        lines[2].contains("g-05") && lines[2].contains("keep-open"),
        "{out}"
    );

    let (ok, out, _) = run_cli(
        dir.path(),
        home.path(),
        &["goals", "list", "--stance", "build"],
    );
    assert!(ok);
    assert_eq!(out.lines().count(), 1, "{out}");
    let (_, out, _) = run_cli(
        dir.path(),
        home.path(),
        &["goals", "list", "--priority", "next"],
    );
    assert!(out.contains("g-05") && !out.contains("g-04"), "{out}");

    let (ok, out, _) = run_cli(dir.path(), home.path(), &["--json", "goals", "list"]);
    assert!(ok);
    let v: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(v["goals"][1]["stance"], "build");
    assert_eq!(v["goals"][2]["stance"], "keep-open");
}

#[test]
fn goals_list_parse_error_exits_nonzero() {
    let dir = goals_dir(&["good.md", "bad.md"]);
    let home = tempfile::tempdir().expect("home");
    let (ok, out, err) = run_cli(dir.path(), home.path(), &["goals", "list"]);
    assert!(!ok, "{out}");
    assert!(
        err.contains("bad.md:2:11: unknown firmness \"hard\""),
        "{err}"
    );
    assert!(out.contains("g-04"), "good goals still listed: {out}");
}

#[test]
fn impact_set_and_show() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_guard, repo, home) = start_daemon(tmp.path());

    let (ok, out, err) = run_cli(&repo, &home, &["task", "new", "t", "-k", "chore", "--json"]);
    assert!(ok, "task new failed: {err}");
    let id = serde_json::from_str::<serde_json::Value>(&out).expect("task json")["id"]
        .as_str()
        .expect("id")
        .to_string();

    let (ok, out, _) = run_cli(&repo, &home, &["impact", "show", &id]);
    assert!(ok);
    assert!(out.contains("no impact declared"), "{out}");

    let (ok, _, err) = run_cli(
        &repo,
        &home,
        &[
            "impact",
            "set",
            &id,
            "--modify",
            "s-b310",
            "--add-under",
            "r-7fa2",
            "--remove",
            "s-11c0",
            "--files",
            "client/**",
            "pkg/**",
        ],
    );
    assert!(ok, "impact set failed: {err}");

    let (ok, out, err) = run_cli(&repo, &home, &["impact", "show", &id, "--json"]);
    assert!(ok, "impact show failed: {err}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("impact json");
    assert_eq!(v["modify"][0], "s-b310");
    assert_eq!(v["add_under"][0], "r-7fa2");
    assert_eq!(v["remove"][0], "s-11c0");
    assert_eq!(v["files"][1], "pkg/**");

    // A malformed id is rejected and leaves the declaration alone.
    let (ok, _, err) = run_cli(&repo, &home, &["impact", "set", &id, "--modify", "zzz"]);
    assert!(!ok);
    assert!(err.contains("bad spec id"), "{err}");

    // A task that is no longer open/planned/claimed can't be changed.
    let (ok, _, err) = run_cli(&repo, &home, &["task", "drop", &id, "--reason", "x"]);
    assert!(ok, "drop failed: {err}");
    let (ok, _, err) = run_cli(&repo, &home, &["impact", "set", &id, "--modify", "s-b310"]);
    assert!(!ok);
    assert!(err.contains("impact can only be set"), "{err}");
}

#[test]
fn arch_propose_creates_arch_revision_task() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);

    // Create design/architecture directory with a valid arch element
    let arch_dir = repo.join("design/architecture");
    std::fs::create_dir_all(&arch_dir).expect("mkdir arch");
    std::fs::write(
        arch_dir.join("elements.md"),
        "## Example Element   {#a-1234}\nThis is an example.",
    )
    .expect("write arch file");

    let (_guard, repo, home) = start_daemon(tmp.path());

    // arch propose creates an arch-revision task
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "arch",
            "propose",
            "--title",
            "Change example",
            "--argument",
            "Reasoning",
        ],
    );
    assert!(ok, "arch propose failed: {err}");
    assert!(out.contains("arch-revision"), "{out}");

    // arch propose --json returns task JSON
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "arch",
            "propose",
            "--title",
            "Change again",
            "--argument",
            "More reasoning",
            "--json",
        ],
    );
    assert!(ok, "arch propose --json failed: {err}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("task json");
    assert_eq!(v["kind"], "arch-revision");
    assert!(v["body"].as_str().unwrap().contains("More reasoning"));
}

#[test]
fn goals_propose_creates_question_task() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);

    // Create design/goals directory with a valid goal
    let goals_dir = repo.join("design/goals");
    std::fs::create_dir_all(&goals_dir).expect("mkdir goals");
    std::fs::write(
        goals_dir.join("example.md"),
        "## Example Goal   {#g-01}\nfirmness: soft · priority: later · stance: unaddressed\n\nAn example goal.\n\n**Why unaddressed:** We don't need it yet.",
    )
    .expect("write goals file");

    let (_guard, repo, home) = start_daemon(tmp.path());

    // goals propose creates a question task
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "goals",
            "propose",
            "g-01",
            "--change",
            "priority=now",
            "--why",
            "We need it sooner",
        ],
    );
    assert!(ok, "goals propose failed: {err}");
    assert!(out.contains("question"), "{out}");

    // goals propose --json returns task JSON
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "goals",
            "propose",
            "g-01",
            "--change",
            "stance=build",
            "--why",
            "Time to build it",
            "--json",
        ],
    );
    assert!(ok, "goals propose --json failed: {err}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("task json");
    assert_eq!(v["kind"], "question");
    assert!(v["body"].as_str().unwrap().contains("stance=build"));
    assert!(v["body"].as_str().unwrap().contains("Time to build it"));
}

#[test]
fn task_done_skips_warning_for_human_claimed_tasks() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    // Create a human to-do
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &[
            "task",
            "new",
            "--kind",
            "bug",
            "--for-human",
            "human-task",
            "--json",
        ],
    );
    assert!(ok, "task new failed: {err}");
    let task: serde_json::Value = serde_json::from_str(&out).expect("task json");
    let task_id = task["id"].as_str().expect("task id").to_string();

    // Done the human task and check that no warning is printed
    let (ok, _out, err) = run_cli(&repo, &home, &["task", "done", &task_id]);
    assert!(ok, "task done failed");
    assert!(
        !err.contains("has no summary"),
        "human-claimed task should not warn about missing summary; stderr: {err}"
    );
}

#[test]
fn task_done_warns_for_agent_claimed_tasks_without_summary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let workspace = tmp.path().to_path_buf();
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);

    // Create a regular task that is not claimed by a human
    let (ok, out, err) = run_cli(
        &repo,
        &home,
        &["task", "new", "--kind", "bug", "regular-task", "--json"],
    );
    assert!(ok, "task new failed: {err}");
    let task: serde_json::Value = serde_json::from_str(&out).expect("task json");
    let task_id = task["id"].as_str().expect("task id").to_string();

    // Create a commit to mark the task as done
    let commit_output = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .expect("get HEAD commit");
    let commit = String::from_utf8_lossy(&commit_output.stdout)
        .trim()
        .to_string();

    // Done the unclaimed task and check that a warning IS printed (since it wasn't claimed by human)
    let (ok, _out, err) = run_cli(
        &repo,
        &home,
        &["task", "done", &task_id, "--commit", &commit],
    );
    assert!(ok, "task done failed");
    assert!(
        err.contains("has no summary"),
        "unclaimed task should warn about missing summary; stderr: {err}"
    );
}

/// x56y: an agent runs with both `BRIDLE_URL` (its own daemon) and `BRIDLE_PROJECT` (that
/// daemon's project). `bridle send` must send locally, `--task` included; a different project
/// still goes the cross-project way (the outbox), and no `BRIDLE_PROJECT` is the plain send.
#[test]
fn send_with_an_agents_own_url_and_project_sends_locally() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
    let home = tmp.path().join("home");

    let mut serve_cmd = Command::new(bridle_bin());
    serve_cmd
        .arg("serve")
        .arg("--repo")
        .arg(&repo)
        .arg("--listen")
        .arg("127.0.0.1:0")
        .env("BRIDLE_CLAUDE_BIN", fake_claude_path())
        .env("BRIDLE_HOME", &home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    strip_bridle_env(&mut serve_cmd);
    let mut guard = DaemonGuard(serve_cmd.spawn().expect("spawn bridle serve"));
    let daemon_json = tmp.path().join(".bridle/daemon.json");
    wait_for_daemon(&mut guard.0, &daemon_json);
    let info: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&daemon_json).expect("daemon.json"))
            .expect("daemon.json parses");
    let url = info["url"].as_str().expect("url").to_string();
    let project = info["project"].as_str().expect("project").to_string();

    let (ok, out, err) = run_cli(&repo, &home, &["task", "new", "t", "-k", "chore", "--json"]);
    assert!(ok, "task new failed: {err}");
    let task: serde_json::Value = serde_json::from_str(&out).expect("task json");
    let task_id = task["id"].as_str().expect("task id").to_string();

    // The agent's environment: run from outside the workspace, so only the env finds the daemon.
    let elsewhere = tmp.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("mkdir");
    let (ok, out, err) = run_cli(&repo, &home, &["token", "create", "agent", "--json"]);
    assert!(ok, "token create failed: {err}");
    let created: serde_json::Value = serde_json::from_str(&out).expect("token json");
    let token = created["token"].as_str().expect("token secret").to_string();
    let send = |project_env: Option<&str>, args: &[&str]| {
        let mut cmd = Command::new(bridle_bin());
        cmd.args(args)
            .current_dir(&elsewhere)
            .env("BRIDLE_HOME", &home)
            .env("BRIDLE_URL", &url);
        strip_bridle_env_except_url(&mut cmd);
        if let Some(p) = project_env {
            cmd.env("BRIDLE_PROJECT", p);
        }
        cmd.env("BRIDLE_TOKEN", &token);
        let out = cmd.output().expect("run bridle");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };

    let (ok, _, err) = send(Some(&project), &["send", "human", "own", "--json"]);
    assert!(ok, "send with own project failed: {err}");
    let (ok, _, err) = send(
        Some(&project),
        &["send", "human", "own", "--task", &task_id, "--json"],
    );
    assert!(ok, "send --task with own project failed: {err}");
    let (ok, _, err) = send(None, &["send", "human", "bare", "--json"]);
    assert!(ok, "send without BRIDLE_PROJECT failed: {err}");

    // Another project: not a direct send (`--task` is refused for it, which only the
    // cross-project path does).
    let (ok, _, err) = send(
        Some("some-other-project"),
        &["send", "human", "x", "--task", &task_id],
    );
    assert!(!ok);
    assert!(
        err.contains("another project's daemon"),
        "expected the cross-project path: {err}"
    );
}

fn strip_bridle_env_except_url(cmd: &mut Command) {
    cmd.env_remove("CLAUDECODE")
        .env_remove("BRIDLE_TOKEN")
        .env_remove("BRIDLE_AGENT_ID")
        .env_remove("BRIDLE_AGENT_NAME")
        .env_remove("BRIDLE_PROJECT");
}

#[test]
fn messages_shows_own_bodies_and_headers_only_for_others() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_guard, repo, home) = start_daemon(tmp.path());
    let (ok, _, err) = run_cli(
        &repo,
        &home,
        &[
            "spawn", "worker", "--name", "w1", "--prompt", "hi", "--json",
        ],
    );
    assert!(ok, "spawn failed: {err}");
    wait_for_first_turn(&repo, &home);
    for (to, body) in [
        ("human", "for-the-human"),
        ("w1", "for-w1"),
        ("w1", "second-for-w1"),
    ] {
        let (ok, _, err) = run_cli(&repo, &home, &["send", to, body]);
        assert!(ok, "send failed: {err}");
    }
    let json = |args: &[&str]| -> Vec<serde_json::Value> {
        let (ok, out, err) = run_cli(&repo, &home, args);
        assert!(ok, "{args:?} failed: {err}");
        serde_json::from_str(&out).expect("messages json")
    };

    // Default scope is the caller's own, bodies included.
    let own = json(&["messages", "--json"]);
    assert_eq!(own.len(), 1, "{own:?}");
    assert_eq!(own[0]["body"], "for-the-human");
    assert_eq!(own[0]["sent_via"], "ui");

    // Another principal's: headers only, newest `--last N`.
    let others = json(&["messages", "--for", "w1", "--json"]);
    // The spawn prompt is a message too.
    assert_eq!(others.len(), 3, "{others:?}");
    assert!(others.iter().all(|m| m["body"].is_null()), "{others:?}");
    let last = json(&["messages", "--for", "w1", "--last", "1", "--json"]);
    assert_eq!(last.len(), 1);
    assert_eq!(last[0]["id"], others[2]["id"]);

    // A window that excludes everything, and a bad one.
    assert!(json(&["messages", "--for", "w1", "--since", "0s", "--json"]).is_empty());
    let (ok, _, _) = run_cli(&repo, &home, &["messages", "--since", "soon"]);
    assert!(!ok);
}
