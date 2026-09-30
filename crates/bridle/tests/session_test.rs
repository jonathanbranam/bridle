//! `bridle session orchestrator|advisor`: the command lines, env, tags and files the launch
//! scripts produced. `claude` and `tmux` are stubs on PATH; no real claude, no real orchestrator.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Run {
    out: Output,
    home: tempfile::TempDir,
    /// One argument per line, then a line of the env the stub saw.
    claude: String,
    tmux: String,
    /// BRIDLE_HOME's files as claude saw them, while it ran.
    files_during: String,
}

fn stub(dir: &Path, name: &str, body: &str) {
    let p = dir.join(name);
    fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn session(args: &[&str], envs: &[(&str, &str)], claude_exit: i32) -> Run {
    let bin = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let rec = bin.path().join("claude.rec");
    let tmux_rec = bin.path().join("tmux.rec");
    stub(
        bin.path(),
        "claude",
        &format!(
            "for a in \"$@\"; do printf '%s\\n' \"$a\"; done > {rec}\n\
             echo \"AS=$BRIDLE_AS PROJECT=$BRIDLE_PROJECT ADVISOR=$BRIDLE_ADVISOR_NAME\" >> {rec}\n\
             ls {home} > {rec}.files\nexit {claude_exit}",
            rec = rec.display(),
            home = home.path().display()
        ),
    );
    stub(
        bin.path(),
        "tmux",
        &format!("echo \"$@\" >> {}", tmux_rec.display()),
    );
    let cwd: PathBuf = tempfile::tempdir().unwrap().keep();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bridle"));
    cmd.args(["session"])
        .args(args)
        .current_dir(cwd)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.path().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("BRIDLE_HOME", home.path())
        .env("BRIDLE_AGENT_ID", "a-test")
        .env("BRIDLE_LAUNCHER_TEST", "1")
        .env("TMUX_PANE", "%9")
        .env_remove("BRIDLE_PROJECT")
        .env_remove("BRIDLE_SESSION_SUFFIX");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    Run {
        out,
        home,
        claude: fs::read_to_string(&rec).unwrap_or_default(),
        tmux: fs::read_to_string(&tmux_rec).unwrap_or_default(),
        files_during: fs::read_to_string(bin.path().join("claude.rec.files")).unwrap_or_default(),
    }
}

fn lines(r: &Run) -> Vec<&str> {
    r.claude.lines().collect()
}

#[test]
fn orchestrator_command_line_matches_the_script() {
    let r = session(&["orchestrator"], &[], 0);
    let l = lines(&r);
    assert_eq!(l[0], "--settings");
    assert!(l[1].contains("bridle orchestrator note-session") && l[1].contains("disableWorkflows"));
    assert_eq!(
        &l[2..7],
        [
            "--strict-mcp-config",
            "--name",
            "orch-bridle",
            "--remote-control",
            "orch-bridle"
        ]
    );
    assert_eq!(
        l[7],
        "Run `bridle prime orchestrator` and follow what it prints."
    );
    assert_eq!(l[8], "AS=orchestrator PROJECT=bridle ADVISOR=");
    assert!(
        r.tmux.contains("set-option -p -t %9 @bridle orchestrator"),
        "{}",
        r.tmux
    );
    let pid = fs::read_to_string(r.home.path().join("orchestrator.pid")).unwrap();
    assert!(pid.split_whitespace().count() >= 3, "{pid}");
    let exits = fs::read_to_string(r.home.path().join("orchestrator.exits")).unwrap();
    assert!(
        exits.contains("orchestrator claude ended: exit 0"),
        "{exits}"
    );
}

#[test]
fn orchestrator_uses_project_suffix_and_passes_extra_args() {
    let r = session(
        &["--project", "meta-notes", "orchestrator", "--resume", "x"],
        &[("BRIDLE_SESSION_SUFFIX", "nuc")],
        0,
    );
    let l = lines(&r);
    assert_eq!(l[4], "orch-meta-notes-nuc");
    assert_eq!(
        &l[6..10],
        [
            "orch-meta-notes-nuc",
            "--resume",
            "x",
            "Run `bridle prime orchestrator` and follow what it prints."
        ]
    );
    assert!(l[10].contains("PROJECT=meta-notes"));
}

#[test]
fn orchestrator_records_a_nonzero_exit() {
    let r = session(&["orchestrator"], &[], 3);
    assert_eq!(r.out.status.code(), Some(3));
    let exits = fs::read_to_string(r.home.path().join("orchestrator.exits")).unwrap();
    assert!(exits.contains("exit 3"), "{exits}");
}

#[test]
fn advisor_unnamed_has_a_pid_file_only_while_running() {
    let r = session(&["--project", "p", "advisor"], &[], 0);
    let l = lines(&r);
    assert_eq!(
        &l[3..7],
        ["--name", "advisor-p", "--remote-control", "advisor-p"]
    );
    assert!(l[7].starts_with("Run `bridle prime advisor`"));
    assert_eq!(l[8], "AS=advisor PROJECT=p ADVISOR=");
    assert!(r.tmux.contains("@bridle advisor\n"), "{}", r.tmux);
    assert!(
        r.files_during.contains("advisor-p.pid"),
        "{}",
        r.files_during
    );
    assert!(!r.home.path().join("advisor-p.pid").exists());
}

#[test]
fn named_advisor_signs_tags_and_skips_the_pid_file() {
    let r = session(
        &["--project", "p", "advisor", "alice", "--model", "m"],
        &[("BRIDLE_SESSION_SUFFIX", "nuc")],
        0,
    );
    let l = lines(&r);
    assert_eq!(l[4], "advisor-alice-p-nuc");
    assert_eq!(l[7], "--model");
    assert_eq!(l[8], "m");
    assert_eq!(l[10], "AS=advisor PROJECT=p ADVISOR=alice");
    assert!(r.tmux.contains("@bridle advisor-alice"), "{}", r.tmux);
    assert!(!r.home.path().join("advisor-p.pid").exists());
}
