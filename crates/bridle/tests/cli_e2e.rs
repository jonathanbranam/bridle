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

fn fake_claude_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../bridle-claude/tests/fake-claude.py")
        .canonicalize()
        .expect("fake-claude.py exists")
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
/// (see docs/questions/open/v1-follow-ups-from-the-build-9c6e.md).
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

fn wait_for_file(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while !path.is_file() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(100));
    }
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
        .stderr(Stdio::null());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_file(&daemon_json, Duration::from_secs(20));

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
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let (ok, out, err) = run_cli(&repo, &home, &["agents", "--all", "--json"]);
        assert!(ok, "agents failed: {err}");
        let agents: serde_json::Value = serde_json::from_str(&out).expect("agents json");
        let w1 = agents
            .as_array()
            .expect("array")
            .iter()
            .find(|a| a["name"] == "w1")
            .expect("w1 present");
        if w1["turns"].as_u64().unwrap_or(0) >= 1 && w1["state"] == "idle" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "w1 never finished its first turn: {agents}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }

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
        out.contains("received; shutting down gracefully, may take up to 30s"),
        "{out}"
    );
    assert!(
        !daemon_json.exists(),
        "daemon.json should be removed on clean shutdown"
    );

    let status = guard
        .0
        .wait_timeout_or_kill(Duration::from_secs(10))
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
        .stderr(Stdio::null());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let _guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_file(&daemon_json, Duration::from_secs(20));

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
    wait_for_file(&daemon_json, Duration::from_secs(20));

    kill(Pid::from_raw(pid as i32), Signal::SIGINT).expect("send SIGINT");

    let status = guard
        .0
        .wait_timeout_or_kill(Duration::from_secs(10))
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
    wait_for_file(&daemon_json, Duration::from_secs(20));

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
        .wait_timeout_or_kill(Duration::from_secs(60))
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
        .stderr(Stdio::null());
    strip_bridle_env(&mut serve_cmd);
    let child = serve_cmd.spawn().expect("spawn bridle serve");
    let mut guard = DaemonGuard(child);

    let daemon_json = workspace.join(".bridle/daemon.json");
    wait_for_file(&daemon_json, Duration::from_secs(20));

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
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let (ok, out, err) = run_cli(&repo, &home, &["agents", "--all", "--json"]);
        assert!(ok, "agents failed: {err}");
        let agents: serde_json::Value = serde_json::from_str(&out).expect("agents json");
        let w1 = agents
            .as_array()
            .expect("array")
            .iter()
            .find(|a| a["name"] == "w1")
            .expect("w1 present");
        if w1["turns"].as_u64().unwrap_or(0) >= 1 && w1["state"] == "idle" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "w1 never finished its first turn: {agents}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // Send a message from w1 to human
    let (ok, out, err) = run_cli(&repo, &home, &["send", "human", "hello", "--json"]);
    assert!(ok, "send failed: {err}");
    let msgs: serde_json::Value = serde_json::from_str(&out).expect("message json");
    let msg_id = msgs[0]["id"].as_str().expect("message id").to_string();

    // Test `bridle inbox show <id>` (marks read by default)
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "show", &msg_id]);
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

    // Test `bridle inbox show <id> --no-mark-read`
    let (ok, out, err) = run_cli(&repo, &home, &["inbox", "show", &msg_id2, "--no-mark-read"]);
    assert!(ok, "inbox show --no-mark-read failed: {err}");
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

/// A small extension so the foreground-daemon test can bound how long it
/// waits for the child to exit after asking it to stop, without pulling in
/// a whole process-management crate for one call.
trait WaitTimeoutOrKill {
    fn wait_timeout_or_kill(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<std::process::ExitStatus>;
}

impl WaitTimeoutOrKill for Child {
    fn wait_timeout_or_kill(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<std::process::ExitStatus> {
        let deadline = Instant::now() + timeout;
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
        .stderr(Stdio::null());
    strip_bridle_env(&mut serve_cmd);
    let guard = DaemonGuard(serve_cmd.spawn().expect("spawn bridle serve"));
    wait_for_file(&tmp.join(".bridle/daemon.json"), Duration::from_secs(20));
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
