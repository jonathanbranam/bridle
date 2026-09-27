//! End-to-end test of the real `bridle` binary against a real (but
//! fake-claude-backed) daemon: `serve` as a child process, then a sequence
//! of CLI subcommands against it, then `serve --detach` + `daemons`. No
//! real `claude` is ever spawned.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

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
    run(&["init", "-q"]);
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
    let msg: serde_json::Value = serde_json::from_str(&out).expect("message json");
    assert_eq!(msg["to"], agent_id);

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

    // `stop-daemon`
    let (ok, out, err) = run_cli(&repo, &home, &["stop-daemon"]);
    assert!(ok, "stop-daemon failed: {err}");
    assert!(out.contains("daemon stopped"), "{out}");
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

#[test]
fn serve_detach_returns_once_healthy_and_daemons_lists_it() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    init_repo(&repo);
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
    assert!(
        info["url"].as_str().expect("url").starts_with("http://"),
        "{info}"
    );

    let (ok, out, err) = run_cli(&repo, &home, &["daemons", "--json"]);
    assert!(ok, "daemons failed: {err}");
    let list: serde_json::Value = serde_json::from_str(&out).expect("daemons json");
    let ws_str = workspace.to_string_lossy().into_owned();
    assert!(
        list.as_array()
            .expect("array")
            .iter()
            .any(|d| d["workspace"] == ws_str),
        "expected the detached daemon in the registry: {list}"
    );

    // This daemon's workspace isn't an ancestor of `repo` (explicit
    // `--workspace`), so discover it from inside the workspace instead.
    let (ok, _out, err) = run_cli(&workspace, &home, &["stop-daemon"]);
    assert!(ok, "stop-daemon failed: {err}");
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
