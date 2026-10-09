//! `bridle gateway` reads its own config section; a bad one fails only that command.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::{Command, Output};

/// Fake `launchctl` and `systemctl` that record every call in `<home>/service-calls` and fail
/// (succeed once `<home>/shim-ok` exists),
/// first on PATH, so no test run can reach the real service manager (rztb).
fn shim_path(home: &std::path::Path) -> std::ffi::OsString {
    let dir = home.join("shim");
    std::fs::create_dir_all(&dir).unwrap();
    for tool in ["launchctl", "systemctl"] {
        let path = dir.join(tool);
        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\necho \"{tool} $*\" >> \"{}\"\n[ -e \"{ok}\" ] && exit 0\nexit 1\n",
                home.join("service-calls").display(),
                ok = home.join("shim-ok").display(),
            ),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut paths = vec![dir];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(paths).unwrap()
}

fn bridle_with(home: &std::path::Path, args: &[&str], unmanaged: bool) -> Output {
    // The pid file, log and port all come from BRIDLE_HOME: it must be a temp dir.
    assert!(
        home.starts_with(std::env::temp_dir().canonicalize().unwrap())
            || home.starts_with(std::env::temp_dir()),
        "{} is not under the temp dir",
        home.display()
    );
    let mut cmd = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")));
    cmd.env("BRIDLE_HOME", home)
        .env("PATH", shim_path(home))
        .args(args);
    if unmanaged {
        // Never let a test restart the developer's real launchd/systemd gateway.
        cmd.env("BRIDLE_GATEWAY_UNMANAGED", "1");
    }
    cmd.output().unwrap()
}

fn bridle(home: &std::path::Path, args: &[&str]) -> Output {
    bridle_with(home, args, true)
}

#[test]
fn tests_never_reach_the_real_service_manager() {
    let home = tempfile::tempdir().unwrap();
    // Disabled, so restart only detects the manager and then starts nothing.
    std::fs::write(
        home.path().join("config.toml"),
        "[gateway]\nenabled = false\n",
    )
    .unwrap();
    // Without the opt-out the binary does ask launchctl/systemctl: the shim catches it, which
    // proves the shim is what a test run would reach.
    bridle_with(home.path(), &["gateway", "restart"], false);
    let calls = std::fs::read_to_string(home.path().join("service-calls")).unwrap();
    assert!(
        calls.contains("print") || calls.contains("is-enabled"),
        "{calls}"
    );
    // With the opt-out it asks nobody.
    let _ = std::fs::remove_file(home.path().join("service-calls"));
    bridle(home.path(), &["gateway", "restart"]);
    bridle(home.path(), &["gateway", "status"]);
    bridle(home.path(), &["gateway", "stop"]);
    assert!(!home.path().join("service-calls").exists());
}

#[test]
fn detach_is_refused_when_a_unit_runs_the_gateway() {
    let (home, _) = gateway_home();
    std::fs::write(home.path().join("shim-ok"), "").unwrap();
    let out = bridle_with(home.path(), &["gateway", "--detach"], false);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("is run by"), "{err}");
    assert!(pid_file(home.path()).is_none());
}

#[test]
fn stop_and_start_log_who_asked() {
    let (home, _) = gateway_home();
    let out = bridle(home.path(), &["gateway", "--detach", "--json"]);
    assert!(out.status.success());
    bridle(home.path(), &["gateway", "stop"]);
    let log = std::fs::read_to_string(home.path().join("gateway.log")).unwrap();
    assert!(
        log.contains("gateway start (detached) via cli: agent="),
        "{log}"
    );
    assert!(log.contains("gateway stop (pid "), "{log}");
    assert!(log.contains("ppid="), "{log}");
}

#[test]
fn bad_gateway_config_fails_only_the_gateway() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[gateway]\nbind = \"0.0.0.0:7878\"\n",
    )
    .unwrap();

    let out = bridle(home.path(), &["gateway"]);
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("[gateway] bind"), "{err}");

    let out = bridle(home.path(), &["daemons"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("[gateway]"), "{err}");
}

#[test]
fn malformed_config_fails_only_the_gateway() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join("config.toml"), "[gateway\n").unwrap();
    let out = bridle(home.path(), &["gateway"]);
    assert_eq!(out.status.code(), Some(1));
    let out = bridle(home.path(), &["daemons"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn kill(pid: &str) {
    let _ = Command::new("kill").arg(pid).status();
}

#[test]
fn detach_starts_logs_answers_health_and_refuses_a_second() {
    let home = tempfile::tempdir().unwrap();
    let port = free_port();
    std::fs::write(
        home.path().join("config.toml"),
        format!("[gateway]\nbind = \"127.0.0.1:{port}\"\n"),
    )
    .unwrap();

    let out = bridle(home.path(), &["gateway", "--detach", "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let pid = v["pid"].to_string();

    let second = bridle(home.path(), &["gateway", "--detach"]);
    let err = String::from_utf8_lossy(&second.stderr).to_string();
    let log = std::fs::read_to_string(home.path().join("gateway.log")).unwrap_or_default();
    kill(&pid);
    assert_eq!(second.status.code(), Some(1), "{err}");
    assert!(err.contains("already answers"), "{err}");
    assert!(log.contains("gateway listening"), "{log}");
}

#[test]
fn disabled_gateway_exits_cleanly_without_starting() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[gateway]\nenabled = false\n",
    )
    .unwrap();
    let out = bridle(home.path(), &["gateway", "--detach"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("disabled"));
}

#[test]
fn a_replaced_binary_is_re_executed() {
    let home = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let port = free_port();
    std::fs::write(
        home.path().join("config.toml"),
        format!("[gateway]\nbind = \"127.0.0.1:{port}\"\n"),
    )
    .unwrap();
    let exe = bin_dir.path().join("bridle");
    std::fs::copy(env!("CARGO_BIN_EXE_bridle"), &exe).unwrap();
    let mut child = Command::new(&exe)
        .env("BRIDLE_HOME", home.path())
        .env("BRIDLE_GATEWAY_BINARY_CHECK_SECS", "1")
        .arg("gateway")
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    // Wait until it listens: the change check starts from the binary as it was by then.
    for _ in 0..300 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    // Replace like `cargo install`: a new file renamed over the path. The fake records the call.
    let marker = home.path().join("restarted");
    let fake = bin_dir.path().join("new");
    std::fs::write(
        &fake,
        format!("#!/bin/sh\necho \"$@\" > {}\n", marker.display()),
    )
    .unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    std::fs::rename(&fake, &exe).unwrap();
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "the fake ran in place of the gateway: {status}"
    );
    assert_eq!(std::fs::read_to_string(&marker).unwrap().trim(), "gateway");
}

#[test]
fn hash_password_with_piped_input() {
    let home = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .env("BRIDLE_HOME", home.path())
        .arg("gateway")
        .arg("hash-password")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(mut stdin) = cmd.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(b"test_password\n");
    }

    let out = cmd.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!hash.is_empty(), "hash should not be empty");
    assert!(hash.len() > 20, "hash should be a valid argon2 hash");
}

#[test]
fn hash_password_with_piped_empty_input_fails() {
    let home = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .env("BRIDLE_HOME", home.path())
        .arg("gateway")
        .arg("hash-password")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(mut stdin) = cmd.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(b"\n");
    }

    let out = cmd.wait_with_output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no password"), "{err}");
}

fn gateway_home() -> (tempfile::TempDir, u16) {
    let home = tempfile::tempdir().unwrap();
    let port = free_port();
    std::fs::write(
        home.path().join("config.toml"),
        format!("[gateway]\nbind = \"127.0.0.1:{port}\"\n"),
    )
    .unwrap();
    (home, port)
}

fn pid_file(home: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(home.join("gateway.pid")).ok()
}

fn wait_until(what: &str, f: impl Fn() -> bool) {
    for _ in 0..100 {
        if f() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("timed out waiting for {what}");
}

fn alive(pid: &str) -> bool {
    Command::new("kill")
        .args(["-0", pid])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success()
}

#[test]
fn pid_file_is_written_and_removed_on_sigterm_and_stop_ends_the_gateway() {
    let (home, _) = gateway_home();
    let out = bridle(home.path(), &["gateway", "--detach", "--json"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let pid = v["pid"].to_string();
    // The child writes it, once listening.
    wait_until("pid file", || pid_file(home.path()).is_some());
    assert_eq!(pid_file(home.path()).unwrap(), format!("{pid}\n"));

    let status = bridle(home.path(), &["gateway", "status"]);
    let text = String::from_utf8_lossy(&status.stdout).to_string();
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(
        text.starts_with(&format!("running pid {pid} http://127.0.0.1:")),
        "{text}"
    );
    assert!(!text.contains("stale binary"), "{text}");

    let stop = bridle(home.path(), &["gateway", "stop"]);
    assert!(stop.status.success());
    assert_eq!(String::from_utf8_lossy(&stop.stdout).trim(), "stopped");
    assert!(!alive(&pid));
    assert!(pid_file(home.path()).is_none());

    let status = bridle(home.path(), &["gateway", "status"]);
    assert_eq!(status.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&status.stdout).trim(),
        "not running"
    );
}

#[test]
fn a_stale_pid_file_is_ignored_overwritten_and_cleaned() {
    let (home, _) = gateway_home();
    // A pid that is gone: a child we ran to completion.
    let mut done = Command::new("true").spawn().unwrap();
    done.wait().unwrap();
    std::fs::write(home.path().join("gateway.pid"), format!("{}\n", done.id())).unwrap();

    let status = bridle(home.path(), &["gateway", "status"]);
    assert_eq!(status.status.code(), Some(1));
    let text = String::from_utf8_lossy(&status.stdout).to_string();
    assert!(
        text.contains("not running") && text.contains("stale pid file removed"),
        "{text}"
    );
    assert!(pid_file(home.path()).is_none());

    std::fs::write(home.path().join("gateway.pid"), format!("{}\n", done.id())).unwrap();
    let out = bridle(home.path(), &["gateway", "--detach", "--json"]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let pid = v["pid"].to_string();
    wait_until("pid file overwritten", || {
        pid_file(home.path()) == Some(format!("{pid}\n"))
    });
    bridle(home.path(), &["gateway", "stop"]);
}

#[test]
fn stop_refuses_a_pid_that_is_not_a_gateway() {
    let (home, _) = gateway_home();
    let mut other = Command::new("sleep").arg("30").spawn().unwrap();
    std::fs::write(home.path().join("gateway.pid"), format!("{}\n", other.id())).unwrap();

    let out = bridle(home.path(), &["gateway", "stop"]);
    let still_alive = other.try_wait().unwrap().is_none();
    let file = pid_file(home.path());
    let _ = other.kill();
    let _ = other.wait();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a bridle gateway"));
    assert!(still_alive);
    assert!(file.is_some());
}

#[test]
fn stop_with_no_pid_file_says_not_running() {
    let (home, _) = gateway_home();
    let out = bridle(home.path(), &["gateway", "stop"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "not running");
}

#[test]
fn restart_starts_when_not_running_and_gives_a_new_pid() {
    let (home, _) = gateway_home();
    let first = bridle(home.path(), &["gateway", "restart"]);
    let text = String::from_utf8_lossy(&first.stdout).to_string();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(text.contains("running pid"), "{text}");
    wait_until("pid file", || pid_file(home.path()).is_some());
    let before = pid_file(home.path()).unwrap();

    let second = bridle(home.path(), &["gateway", "restart"]);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    wait_until("new pid", || {
        pid_file(home.path()).is_some_and(|p| p != before)
    });
    let now = pid_file(home.path()).unwrap();
    assert!(!alive(before.trim()));
    bridle(home.path(), &["gateway", "stop"]);
    assert!(!alive(now.trim()));
}
