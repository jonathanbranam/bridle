//! Tests for `bridle pane tag` and `bridle pane untag` commands.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bridle_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bridle"))
}

fn run_bridle(cwd: &Path, args: &[&str], env_pane: Option<&str>) -> (bool, String, String) {
    let mut cmd = Command::new(bridle_bin());
    cmd.current_dir(cwd);

    if let Some(pane) = env_pane {
        cmd.env("TMUX_PANE", pane);
    } else {
        cmd.env_remove("TMUX_PANE");
    }

    cmd.args(args);

    let output = cmd.output().expect("failed to run bridle");
    let ok = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (ok, stdout, stderr)
}

fn fake_tmux(dir: &Path) {
    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin dir");

    let tmux_script = bin_dir.join("tmux");
    fs::write(
        &tmux_script,
        r#"#!/usr/bin/env bash
# Stub tmux that records arguments to .tmux-calls and succeeds.
echo "$@" >> .tmux-calls
"#,
    )
    .expect("write tmux stub");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmux_script, fs::Permissions::from_mode(0o755))
            .expect("chmod tmux stub");
    }
}

#[test]
fn pane_tag_with_stub_tmux() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    fake_tmux(dir);

    // Set PATH to include our stub tmux
    let bin_dir = dir.join("bin");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), old_path);

    let mut cmd = Command::new(bridle_bin());
    cmd.current_dir(dir);
    cmd.env("PATH", &new_path);
    cmd.env("TMUX_PANE", "%0");
    cmd.args(["pane", "tag", "test-tag"]);

    let output = cmd.output().expect("failed to run bridle");
    assert!(
        output.status.success(),
        "bridle pane tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("tagged"),
        "expected 'tagged' in output, got: {stdout}"
    );

    let calls_path = dir.join(".tmux-calls");
    assert!(calls_path.exists(), ".tmux-calls file not created");
    let calls = fs::read_to_string(&calls_path).expect("read .tmux-calls");
    assert!(
        calls.contains("set-option") && calls.contains("@bridle") && calls.contains("test-tag"),
        "unexpected tmux calls: {calls}"
    );
}

#[test]
fn pane_untag_with_stub_tmux() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    fake_tmux(dir);

    let bin_dir = dir.join("bin");
    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), old_path);

    let mut cmd = Command::new(bridle_bin());
    cmd.current_dir(dir);
    cmd.env("PATH", &new_path);
    cmd.env("TMUX_PANE", "%0");
    cmd.args(["pane", "untag"]);

    let output = cmd.output().expect("failed to run bridle");
    assert!(
        output.status.success(),
        "bridle pane untag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("untagged"),
        "expected 'untagged' in output, got: {stdout}"
    );

    let calls_path = dir.join(".tmux-calls");
    assert!(calls_path.exists(), ".tmux-calls file not created");
    let calls = fs::read_to_string(&calls_path).expect("read .tmux-calls");
    assert!(
        calls.contains("set-option") && calls.contains("@bridle") && calls.contains("-u"),
        "unexpected tmux calls: {calls}"
    );
}

#[test]
fn pane_tag_errors_outside_tmux() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    let (ok, _stdout, stderr) = run_bridle(dir, &["pane", "tag", "test-tag"], None);
    assert!(!ok, "bridle pane tag should fail outside tmux");
    assert!(
        stderr.contains("TMUX_PANE") || stderr.contains("tmux pane"),
        "expected error about TMUX_PANE, got: {stderr}"
    );
}

#[test]
fn pane_untag_errors_outside_tmux() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    let (ok, _stdout, stderr) = run_bridle(dir, &["pane", "untag"], None);
    assert!(!ok, "bridle pane untag should fail outside tmux");
    assert!(
        stderr.contains("TMUX_PANE") || stderr.contains("tmux pane"),
        "expected error about TMUX_PANE, got: {stderr}"
    );
}

#[test]
fn pane_tag_clears_same_tag_from_other_panes() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin dir");

    // Create a tmux stub that simulates two panes with the same tag
    let tmux_script = bin_dir.join("tmux");
    fs::write(
        &tmux_script,
        r#"#!/usr/bin/env bash
# Stub tmux that simulates list-panes behavior
if [[ "$1" == "list-panes" ]]; then
  # Simulate two panes: %0 with tag "orchestrator", %1 with tag "orchestrator"
  echo "%0:orchestrator"
  echo "%1:orchestrator"
elif [[ "$1" == "set-option" ]]; then
  echo "$@" >> .tmux-calls
fi
"#,
    )
    .expect("write tmux stub");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmux_script, fs::Permissions::from_mode(0o755))
            .expect("chmod tmux stub");
    }

    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), old_path);

    let mut cmd = Command::new(bridle_bin());
    cmd.current_dir(dir);
    cmd.env("PATH", &new_path);
    cmd.env("TMUX_PANE", "%0");
    cmd.args(["pane", "tag", "orchestrator"]);

    let output = cmd.output().expect("failed to run bridle");
    assert!(
        output.status.success(),
        "bridle pane tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should print that tag was moved from %1
    assert!(
        stdout.contains("moved orchestrator from %1"),
        "expected 'moved orchestrator from %1' in output, got: {stdout}"
    );

    let calls_path = dir.join(".tmux-calls");
    let calls = fs::read_to_string(&calls_path).expect("read .tmux-calls");
    // Should have two set-option calls: one to clear %1, one to set %0
    assert!(
        calls.matches("set-option").count() >= 1,
        "expected at least one set-option call to clear tag from other pane: {calls}"
    );
    assert!(
        calls.contains("-u") && calls.contains("%1"),
        "expected set-option -u call for %1 to clear tag: {calls}"
    );
}

#[test]
fn pane_tag_no_message_when_no_other_pane_has_tag() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin dir");

    // Create a tmux stub where only current pane has the tag
    let tmux_script = bin_dir.join("tmux");
    fs::write(
        &tmux_script,
        r#"#!/usr/bin/env bash
# Stub tmux where only %0 has the tag
if [[ "$1" == "list-panes" ]]; then
  echo "%0:orchestrator"
  echo "%1:"
elif [[ "$1" == "set-option" ]]; then
  echo "$@" >> .tmux-calls
fi
"#,
    )
    .expect("write tmux stub");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmux_script, fs::Permissions::from_mode(0o755))
            .expect("chmod tmux stub");
    }

    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), old_path);

    let mut cmd = Command::new(bridle_bin());
    cmd.current_dir(dir);
    cmd.env("PATH", &new_path);
    cmd.env("TMUX_PANE", "%0");
    cmd.args(["pane", "tag", "orchestrator"]);

    let output = cmd.output().expect("failed to run bridle");
    assert!(
        output.status.success(),
        "bridle pane tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should not print "moved" message
    assert!(
        !stdout.contains("moved"),
        "expected no 'moved' message when no other pane has tag, got: {stdout}"
    );
    // Should still print tagged message
    assert!(
        stdout.contains("tagged"),
        "expected 'tagged' in output, got: {stdout}"
    );
}

#[test]
fn pane_tag_still_succeeds_when_list_panes_fails() {
    let tmpdir = tempfile::TempDir::new().expect("create temp dir");
    let dir = tmpdir.path();

    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin dir");

    // Create a tmux stub that fails on list-panes but succeeds on set-option
    let tmux_script = bin_dir.join("tmux");
    fs::write(
        &tmux_script,
        r#"#!/usr/bin/env bash
# Stub tmux that fails on list-panes
if [[ "$1" == "list-panes" ]]; then
  exit 1
elif [[ "$1" == "set-option" ]]; then
  echo "$@" >> .tmux-calls
fi
"#,
    )
    .expect("write tmux stub");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmux_script, fs::Permissions::from_mode(0o755))
            .expect("chmod tmux stub");
    }

    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), old_path);

    let mut cmd = Command::new(bridle_bin());
    cmd.current_dir(dir);
    cmd.env("PATH", &new_path);
    cmd.env("TMUX_PANE", "%0");
    cmd.args(["pane", "tag", "test-tag"]);

    let output = cmd.output().expect("failed to run bridle");
    // Should still succeed even though list-panes failed
    assert!(
        output.status.success(),
        "bridle pane tag should succeed even when list-panes fails: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("tagged"),
        "expected 'tagged' in output, got: {stdout}"
    );

    // Should have called set-option to tag the pane
    let calls_path = dir.join(".tmux-calls");
    let calls = fs::read_to_string(&calls_path).expect("read .tmux-calls");
    assert!(
        calls.contains("set-option") && calls.contains("@bridle") && calls.contains("test-tag"),
        "unexpected tmux calls: {calls}"
    );
}
