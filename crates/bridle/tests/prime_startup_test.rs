//! `[startup.<role>]` (br-r8td): the command's full output is appended to that role's
//! `bridle prime` text only; a failing command leaves a visible line.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn prime(role: &str, config: &str) -> String {
    prime_args(&[role], config)
}

fn prime_args(args: &[&str], config: &str) -> String {
    let workflow = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflow")
        .canonicalize()
        .unwrap();
    let repo = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(repo.path().join(".bridle")).unwrap();
    fs::write(
        repo.path().join(".bridle/config.toml"),
        format!("workflow = {:?}\n{config}", workflow.display().to_string()),
    )
    .unwrap();
    fs::write(repo.path().join("marker.txt"), "in-workspace\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .arg("prime")
        .args(args)
        .current_dir(repo.path())
        .env("BRIDLE_HOME", home.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

const CFG: &str = "[startup.aide]\ncommand = \"seq 1 500; cat marker.txt\"\n";

#[test]
fn full_output_lands_in_the_configured_role_only() {
    let aide = prime_args(&["aide", "--startup-only"], CFG);
    assert!(aide.contains("## Startup command"));
    assert!(
        aide.contains("\n1\n") && aide.contains("\n500\n"),
        "all 500 lines"
    );
    assert!(aide.contains("in-workspace"), "runs in the project dir");
    assert!(
        prime("aide", CFG).find("Startup command").is_none(),
        "plain prime has no startup text"
    );
    assert!(
        prime_args(&["aide", "--startup-only"], "").is_empty(),
        "off by default"
    );
}

#[test]
fn failure_and_empty_output_are_visible() {
    let failed = prime_args(
        &["aide", "--startup-only"],
        "[startup.aide]\ncommand = \"echo boom; exit 3\"\n",
    );
    assert!(failed.contains("startup command failed"), "{failed}");
    assert!(failed.contains("boom"));
    let empty = prime_args(
        &["aide", "--startup-only"],
        "[startup.aide]\ncommand = \"true\"\n",
    );
    assert!(empty.contains("startup command produced no output"));
    let slow = prime_args(
        &["aide", "--startup-only"],
        "[startup.aide]\ncommand = \"sleep 30\"\ntimeout_secs = 1\n",
    );
    assert!(slow.contains("startup command failed: timed out after 1s"));
}

#[test]
fn huge_output_is_cut_with_a_stated_limit() {
    let out = prime_args(
        &["aide", "--startup-only"],
        "[startup.aide]\ncommand = \"head -c 400000 /dev/zero | tr '\\\\0' x\"\n",
    );
    assert!(out.contains("[startup command output cut at 262144 bytes]"));
}
