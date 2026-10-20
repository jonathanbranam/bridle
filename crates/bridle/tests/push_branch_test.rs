//! `bridle push-branch` (t7ya) end to end: temp bare origin and clone, no network.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

fn git(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .output()
        .unwrap()
}

fn ok(o: Output) {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

fn push_branch(dir: &Path, agent: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bridle"))
        .current_dir(dir)
        .env("BRIDLE_AGENT_NAME", agent)
        .env("BRIDLE_HOME", dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("push-branch")
        .args(args)
        .output()
        .unwrap()
}

fn remote_has(origin: &Path, branch: &str) -> bool {
    git(
        origin,
        &[
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/heads/{branch}"),
        ],
    )
    .status
    .success()
}

/// An origin with `main`, and a clone on `bridle/w1` with one commit ahead of it.
fn world() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let origin = root.join("origin.git");
    let clone = root.join("clone");
    ok(Command::new("git")
        .args(["init", "-q", "--bare", "-b", "main"])
        .arg(&origin)
        .output()
        .unwrap());
    ok(Command::new("git")
        .arg("clone")
        .arg("-q")
        .arg(&origin)
        .arg(&clone)
        .output()
        .unwrap());
    ok(git(&clone, &["checkout", "-q", "-b", "main"]));
    ok(git(
        &clone,
        &["commit", "-q", "--allow-empty", "-m", "seed"],
    ));
    ok(git(&clone, &["push", "-q", "origin", "main"]));
    ok(git(&clone, &["checkout", "-q", "-b", "bridle/w1"]));
    ok(git(
        &clone,
        &["commit", "-q", "--allow-empty", "-m", "work"],
    ));
    (dir, origin, clone)
}

#[test]
fn pushes_own_branch_with_and_without_the_name() {
    let (_d, origin, clone) = world();
    ok(push_branch(&clone, "w1", &[]));
    assert!(remote_has(&origin, "bridle/w1"));
    ok(git(
        &clone,
        &["commit", "-q", "--allow-empty", "-m", "more"],
    ));
    ok(push_branch(&clone, "w1", &["bridle/w1"]));
}

#[test]
fn refuses_everything_else_and_pushes_nothing() {
    let (_d, origin, clone) = world();
    ok(git(&clone, &["branch", "bridle/w2"]));
    let cases: [(&str, &[&str]); 8] = [
        ("w1", &["--force"]),
        ("w1", &["-f"]),
        ("w1", &["+bridle/w1"]),
        ("w1", &["bridle/w1:main"]),
        ("w1", &["main"]),
        ("w1", &["bridle/w2"]),
        ("w2", &[]),
        ("w2", &["bridle/w1"]),
    ];
    for (agent, args) in cases {
        let o = push_branch(&clone, agent, args);
        assert!(!o.status.success(), "{agent} {args:?} should be refused");
    }
    assert!(!remote_has(&origin, "bridle/w1"));
    assert!(!remote_has(&origin, "bridle/w2"));
    // The integration branch checked out is refused too.
    ok(git(&clone, &["checkout", "-q", "main"]));
    assert!(!push_branch(&clone, "w1", &[]).status.success());
}
