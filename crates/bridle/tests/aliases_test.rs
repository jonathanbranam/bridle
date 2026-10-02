//! The CLI grouping (ticket a67t) keeps every old top-level command line working: hooks,
//! launchers, launchd/systemd units and scripts spell the old names, and self-upgrade
//! installs a new binary under running agents.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn bridle() -> Command {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
}

/// The 54 top-level names before the grouping.
const OLD_NAMES: &[&str] = &[
    "serve",
    "stop-daemon",
    "restart",
    "doctor",
    "init",
    "launchd",
    "systemd",
    "rebuild",
    "daemons",
    "status",
    "spawn",
    "agents",
    "show",
    "send",
    "inbox",
    "interrupt",
    "stop",
    "resume",
    "renew",
    "rm",
    "logs",
    "events",
    "wait",
    "usage",
    "cost",
    "tui",
    "budget",
    "token",
    "task",
    "impact",
    "probe",
    "land",
    "conflict",
    "port",
    "dep",
    "ask",
    "answer",
    "claim",
    "release",
    "ready",
    "queue",
    "statusline",
    "stop-check",
    "arch-guard",
    "orchestrator",
    "focus",
    "wait-for-wake",
    "mail",
    "handover",
    "prime",
    "rules",
    "sync",
    "workflow",
    "spec",
    "goals",
    "ticket",
    "arch",
    "explore",
    "pane",
    "machine",
    "session",
    "advisor",
    "trace",
];

fn help_ok(args: &[&str]) {
    let out = bridle()
        .args(args)
        .arg("--help")
        .output()
        .expect("run bridle");
    assert!(
        out.status.success(),
        "bridle {args:?} --help failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn every_old_top_level_name_still_parses() {
    for name in OLD_NAMES {
        help_ok(&[name]);
    }
}

#[test]
fn new_groups_parse() {
    for args in [
        &["daemon", "serve"][..],
        &["daemon", "stop"],
        &["daemon", "list"],
        &["agent", "spawn"],
        &["agent", "list"],
        &["agent", "logs"],
        &["agent", "wake"],
        &["task", "claim"],
        &["task", "queue"],
        &["task", "land"],
        &["usage", "cost"],
        &["usage", "budget"],
        &["orchestrator", "prime"],
        &["orchestrator", "wait-for-wake"],
        &["workflow", "rules"],
        &["workflow", "trace"],
        &["hook", "statusline"],
    ] {
        help_ok(args);
    }
}

fn run_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    let home = tempfile::tempdir().expect("tempdir");
    let mut child = bridle()
        .args(args)
        .env("BRIDLE_HOME", home.path())
        .env_remove("BRIDLE_TOKEN")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn bridle");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait")
}

#[test]
fn hook_commands_work_through_old_and_new_paths() {
    for (old, new) in [
        (&["statusline"][..], &["hook", "statusline"][..]),
        (&["stop-check"], &["hook", "stop-check"]),
        (&["arch-guard"], &["hook", "arch-guard"]),
        (
            &["orchestrator", "note-session"],
            &["orchestrator", "note-session"],
        ),
    ] {
        let a = run_with_stdin(old, "{}");
        let b = run_with_stdin(new, "{}");
        assert!(a.status.success(), "bridle {old:?}: {:?}", a);
        assert!(b.status.success(), "bridle {new:?}: {:?}", b);
        assert_eq!(a.stdout, b.stdout, "{old:?} vs {new:?}");
    }
}

#[test]
fn old_flag_shapes_still_parse() {
    // `usage --by` has flags on the same command that now also has subcommands.
    let out = bridle().args(["usage", "--help"]).output().expect("run");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("--by") && text.contains("cost") && text.contains("budget"));
    help_ok(&["wait-for-wake", "--mail"]);
}
