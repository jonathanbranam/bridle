//! `bridle gateway` reads its own config section; a bad one fails only that command.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::{Command, Output};

fn bridle(home: &std::path::Path, args: &[&str]) -> Output {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .env("BRIDLE_HOME", home)
        .args(args)
        .output()
        .unwrap()
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
