//! `bridle token pair` through the real binary, with a fake `ssh` first on the child's PATH
//! (never the real one) and a throwaway BRIDLE_HOME (never the real ~/.bridle).

#![allow(clippy::unwrap_used)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

/// Answers the helpers the way a remote bridle would. Records every ssh argv in `argv.log` and
/// every token it was asked to store in `stored.log`.
const FAKE_SSH: &str = r#"#!/bin/sh
dir=$(dirname "$0")
echo "$@" >> "$dir/argv.log"
if [ -f "$dir/down" ]; then echo "ssh: connect to host refused" >&2; exit 255; fi
case "$*" in
  *pair-projects*) echo "notes 7402" ;;
  *pair-check*) exit 1 ;;
  *pair-mint*) echo "tok-SECRET-$$" ;;
  *pair-store*) cat >> "$dir/stored.log" ;;
  *) echo "unexpected: $*" >&2; exit 2 ;;
esac
"#;

struct Rig {
    home: tempfile::TempDir,
    fake: tempfile::TempDir,
}

fn rig() -> Rig {
    let home = tempfile::tempdir().unwrap();
    let fake = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[machine]\nname = \"mbp\"\n[machines]\nmbp = \"mbp-host\"\nnuc = \"nuc-host\"\n",
    )
    .unwrap();
    let ssh = fake.path().join("ssh");
    std::fs::write(&ssh, FAKE_SSH).unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
    Rig { home, fake }
}

fn pair(r: &Rig, args: &[&str]) -> Output {
    let path = format!(
        "{}:{}",
        r.fake.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["token", "pair"])
        .args(args)
        .env("BRIDLE_HOME", r.home.path())
        .env("PATH", path)
        .env_remove("BRIDLE_AS")
        .env_remove("BRIDLE_AGENT_ID")
        .env_remove("CLAUDECODE")
        .env_remove("BRIDLE_TOKEN")
        .output()
        .unwrap()
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap_or_default()
}

// s-f12f: no token in argv or output, only on the store helper's stdin.
#[test]
fn pairs_over_ssh_and_no_token_is_in_argv_or_output() {
    let r = rig();
    let out = pair(&r, &["--machines", "nuc", "--roles", "aide,mail"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}{stderr}");
    assert!(
        stdout.contains("machine nuc (role tokens): minted 2, kept 0"),
        "{stdout}"
    );
    assert!(!stdout.contains("tok-SECRET") && !stderr.contains("tok-SECRET"));
    let argv = read(r.fake.path(), "argv.log");
    assert!(argv.contains("-o BatchMode=yes nuc-host bridle"), "{argv}");
    assert!(!argv.contains("tok-SECRET"), "{argv}");
    assert_eq!(
        read(r.fake.path(), "stored.log")
            .matches("tok-SECRET")
            .count(),
        2
    );
    // Nothing was written locally: the fake ssh stands in for nuc's credentials file.
    assert!(!r.home.path().join("credentials.toml").exists());
}

// s-d9c0
#[test]
fn dry_run_stores_and_mints_nothing() {
    let r = rig();
    let out = pair(&r, &["--machines", "nuc", "--roles", "aide", "--dry-run"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert!(stdout.contains("would mint 1"), "{stdout}");
    let argv = read(r.fake.path(), "argv.log");
    assert!(
        !argv.contains("pair-mint") && !argv.contains("pair-store"),
        "{argv}"
    );
}

// s-e443
#[test]
fn an_unreachable_machine_is_reported_and_the_exit_is_non_zero() {
    let r = rig();
    std::fs::write(r.fake.path().join("down"), "").unwrap();
    let out = pair(&r, &["--machines", "nuc"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success());
    assert!(stdout.contains("machine nuc: unreachable"), "{stdout}");
}

// s-9267
#[test]
fn roles_with_peer_only_is_a_usage_error() {
    let r = rig();
    let out = pair(&r, &["--roles", "aide", "--tokens", "peer"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--roles"));
}

#[test]
fn an_agent_may_not_pair() {
    let r = rig();
    let out = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["token", "pair"])
        .env("BRIDLE_HOME", r.home.path())
        .env("BRIDLE_AS", "orchestrator")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("human-only"));
}
