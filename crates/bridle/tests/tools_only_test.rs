//! Tools-only clones (hw6c): serve refuses, hooks refuse, `bridle session` refuses.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bridle_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bridle"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn git(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .output()
        .unwrap()
}

/// A temp BRIDLE_HOME whose config lists `repo` as tools-only.
fn home_listing(repo: &Path) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    fs::write(
        home.path().join("config.toml"),
        format!(
            "[machine]\ntools_only = [{:?}]\n",
            repo.display().to_string()
        ),
    )
    .unwrap();
    home
}

fn bridle(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(bridle_bin())
        .current_dir(cwd)
        .env("BRIDLE_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

fn temp_repo() -> (tempfile::TempDir, PathBuf) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().canonicalize().unwrap();
    assert!(git(&p, &["init", "-q"]).status.success());
    (d, p)
}

#[test]
fn serve_refuses_in_a_tools_only_clone() {
    let (_d, repo) = temp_repo();
    let home = home_listing(&repo);
    let out = bridle(home.path(), &repo, &["serve"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("tools-only clone"), "{err}");
}

#[test]
fn hooks_install_is_idempotent_and_blocks_a_commit() {
    let (_d, repo) = temp_repo();
    let home = home_listing(&repo);
    for _ in 0..2 {
        let out = bridle(home.path(), &repo, &["machine", "tools-only-install"]);
        assert!(out.status.success(), "{:?}", out);
    }
    fs::write(repo.join("f"), "x").unwrap();
    git(&repo, &["add", "f"]);
    let commit = git(&repo, &["commit", "-qm", "x"]);
    assert!(!commit.status.success());
    assert!(String::from_utf8_lossy(&commit.stderr).contains("tools-only clone"));
    assert!(
        git(&repo, &["commit", "-qm", "x", "--no-verify"])
            .status
            .success()
    );
}

#[test]
fn hooks_install_moves_a_users_hook_aside_and_unlisting_restores_it() {
    let (_d, repo) = temp_repo();
    let home = home_listing(&repo);
    let hook = repo.join(".git/hooks/pre-push");
    let aside = repo.join(".git/hooks/pre-push.pre-bridle");
    fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    let out = bridle(home.path(), &repo, &["machine", "tools-only-install"]);
    assert!(out.status.success());
    assert_eq!(fs::read_to_string(&aside).unwrap(), "#!/bin/sh\nexit 0\n");
    assert!(
        fs::read_to_string(&hook)
            .unwrap()
            .contains("bridle-tools-only-hook")
    );

    // Unlisted: re-running undoes the install.
    fs::write(home.path().join("config.toml"), "").unwrap();
    let out = bridle(home.path(), &repo, &["machine", "tools-only-install"]);
    assert!(out.status.success());
    assert_eq!(fs::read_to_string(&hook).unwrap(), "#!/bin/sh\nexit 0\n");
    assert!(!aside.exists());
    assert!(!repo.join(".git/hooks/pre-commit").exists());
}

#[test]
fn hooks_install_refuses_outside_the_list() {
    let (_d, repo) = temp_repo();
    let home = tempfile::tempdir().unwrap();
    let out = bridle(home.path(), &repo, &["machine", "tools-only-install"]);
    assert!(!out.status.success());
    assert!(!repo.join(".git/hooks/pre-commit").exists());
}

/// Runs `bridle session <role>` with a stub `claude` that records having started.
fn run_script(role: &str, cwd: &Path, home: &Path) -> (Output, bool) {
    run_script_env(role, cwd, home, None)
}

/// Same, with BRIDLE_AGENT_ID set as under a bridle agent, and the test flag if `flag`.
fn run_script_as_agent(role: &str, home: &Path, flag: bool) -> (Output, bool) {
    run_script_env(role, &repo_root(), home, Some(flag))
}

fn run_script_env(role: &str, cwd: &Path, home: &Path, agent: Option<bool>) -> (Output, bool) {
    let bin = tempfile::tempdir().unwrap();
    let ran = bin.path().join("ran");
    let stub = bin.path().join("claude");
    fs::write(&stub, format!("#!/bin/sh\ntouch {}\n", ran.display())).unwrap();
    Command::new("chmod").arg("+x").arg(&stub).status().unwrap();
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut cmd = Command::new(bridle_bin());
    // This suite may itself run under a bridle agent.
    cmd.env_remove("BRIDLE_AGENT_ID")
        .env_remove("BRIDLE_LAUNCHER_TEST");
    if let Some(flag) = agent {
        cmd.env("BRIDLE_AGENT_ID", "a-test");
        if flag {
            cmd.env("BRIDLE_LAUNCHER_TEST", "1");
        }
    }
    let out = cmd
        .args(["session", role])
        .current_dir(cwd)
        .env("PATH", path)
        .env("BRIDLE_HOME", home)
        .env_remove("TMUX_PANE")
        .output()
        .unwrap();
    (out, ran.exists())
}

#[test]
fn launchers_refuse_in_a_tools_only_clone() {
    let home = home_listing(&repo_root());
    for role in ["orchestrator", "advisor"] {
        let (out, started) = run_script(role, &repo_root(), home.path());
        assert!(!out.status.success(), "{role}");
        assert!(!started, "{role} started claude");
        assert!(String::from_utf8_lossy(&out.stderr).contains("tools-only clone"));
    }
}

#[test]
fn launchers_start_elsewhere() {
    let home = tempfile::tempdir().unwrap();
    let (_, started) = run_script("orchestrator", &repo_root(), home.path());
    assert!(started);
    // Outside any repository too.
    let nowhere = tempfile::tempdir().unwrap();
    let (_, started) = run_script("advisor", nowhere.path(), home.path());
    assert!(started);
}

#[test]
fn launchers_refuse_under_a_bridle_agent_without_the_test_flag() {
    for role in ["orchestrator", "advisor"] {
        let home = tempfile::tempdir().unwrap();
        let (out, started) = run_script_as_agent(role, home.path(), false);
        assert!(!out.status.success(), "{role}");
        assert!(!started, "{role} started claude");
        assert!(String::from_utf8_lossy(&out.stderr).contains("bridle agent"));
        assert!(!home.path().join("orchestrator.pid").exists());
    }
}

#[test]
fn launchers_run_under_a_bridle_agent_with_the_test_flag() {
    let home = tempfile::tempdir().unwrap();
    let (_, started) = run_script_as_agent("orchestrator", home.path(), true);
    assert!(started);
}
