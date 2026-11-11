//! `bridle case-check` (ticket k3ux): fails on tracked paths that collide case-insensitively.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_with(files: &[&str]) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    for f in files {
        let p = d.path().join(f);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "").unwrap();
    }
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .current_dir(d.path())
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    d
}

fn check(dir: &Path) -> Output {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .current_dir(dir)
        .arg("case-check")
        .output()
        .unwrap()
}

#[test]
fn fails_on_timelog_collision_naming_both() {
    // Both are indexed even on a case-insensitive disk only if written under distinct names, so
    // add them to the index directly.
    let d = repo_with(&["x.txt"]);
    for f in ["src/timelog.ts", "src/TimeLog.tsx"] {
        let blob = Command::new("git")
            .current_dir(d.path())
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let sha = String::from_utf8(blob.stdout).unwrap();
        assert!(
            Command::new("git")
                .current_dir(d.path())
                .args([
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    &format!("100644,{},{f}", sha.trim())
                ])
                .status()
                .unwrap()
                .success()
        );
    }
    let out = check(d.path());
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("timelog.ts") && err.contains("TimeLog.tsx"),
        "{err}"
    );
}

#[test]
fn passes_without_collision() {
    let d = repo_with(&[
        "src/time-log.ts",
        "src/TimeLogView.tsx",
        "Cargo.toml",
        "Cargo.lock",
    ]);
    let out = check(d.path());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
