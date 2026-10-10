//! The owner-only pre-push hook (8z7j): temp bare origin, temp clones, no network.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bridle(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bridle"))
        .current_dir(cwd)
        .env("BRIDLE_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

fn git(dir: &Path, args: &[&str]) -> Output {
    // Ignore the host's git config: its template dir installs hooks into `git init`.
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

fn ok(o: Output) -> String {
    assert!(
        o.status.success(),
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn this_host() -> String {
    let out = Command::new("hostname").output().unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

struct World {
    _dir: tempfile::TempDir,
    home: PathBuf,
    origin: PathBuf,
    clone: PathBuf,
}

/// An origin with `main`, and, when `owner` is given, a `bridle/state` branch whose
/// `owner.toml` names that host; plus one clone with the owner hook installed by `bridle sync`.
fn world(owner: Option<&str>) -> World {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let home = root.join("home");
    fs::create_dir(&home).unwrap();
    let origin = root.join("origin.git");
    ok(git(
        &root,
        &[
            "init",
            "-q",
            "--bare",
            "-b",
            "main",
            origin.to_str().unwrap(),
        ],
    ));
    let seed = root.join("seed");
    ok(git(
        &root,
        &["init", "-q", "-b", "main", seed.to_str().unwrap()],
    ));
    ok(git(&seed, &["commit", "-q", "--allow-empty", "-m", "seed"]));
    ok(git(
        &seed,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    ));
    ok(git(&seed, &["push", "-q", "origin", "main"]));
    if let Some(host) = owner {
        ok(git(&seed, &["checkout", "-q", "--orphan", "bridle/state"]));
        fs::write(
            seed.join("owner.toml"),
            format!("host = {host:?}\nsince = \"2026-10-09T14:00:00Z\"\n"),
        )
        .unwrap();
        ok(git(&seed, &["add", "owner.toml"]));
        ok(git(&seed, &["commit", "-q", "-m", "owner"]));
        ok(git(&seed, &["push", "-q", "origin", "bridle/state"]));
    }
    let clone = root.join("clone");
    ok(git(
        &root,
        &[
            "clone",
            "-q",
            origin.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    ));
    ok(bridle_out(&home, &clone, &["sync"]));
    World {
        _dir: dir,
        home,
        origin,
        clone,
    }
}

fn bridle_out(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    bridle(home, cwd, args)
}

fn commit(w: &World) {
    ok(git(
        &w.clone,
        &["commit", "-q", "--no-verify", "--allow-empty", "-m", "work"],
    ));
}

fn push(w: &World, refspec: &str) -> Output {
    git(&w.clone, &["push", "origin", refspec])
}

#[test]
fn owner_pushes_the_integration_branch() {
    let w = world(Some(&this_host()));
    commit(&w);
    ok(push(&w, "main"));
}

#[test]
fn non_owner_is_refused_but_feature_branches_and_tags_pass() {
    let w = world(Some("some-other-box"));
    commit(&w);
    let out = push(&w, "main");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("some-other-box"), "{err}");
    assert!(err.contains("2026-10-09T14:00:00Z"), "{err}");
    assert!(err.contains("bridle serve --take-over"), "{err}");
    ok(push(&w, "HEAD:refs/heads/feature"));
    ok(git(&w.clone, &["tag", "v1"]));
    ok(push(&w, "v1"));
    // The refused push moved nothing on origin.
    let tip = ok(git(&w.origin, &["rev-parse", "main"]));
    let seed = ok(git(&w.clone, &["rev-parse", "origin/main"]));
    assert_eq!(tip, seed);
}

#[test]
fn a_missing_owner_file_refuses() {
    let w = world(None);
    commit(&w);
    let out = push(&w, "main");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("failing closed"));
}

#[test]
fn a_fresh_fetch_beats_a_stale_local_owner() {
    // The clone's local state branch says this host; origin says another one.
    let w = world(Some("some-other-box"));
    let me = format!(
        "host = {:?}\nsince = \"2026-10-01T00:00:00Z\"\n",
        this_host()
    );
    let blob = {
        let mut c = Command::new("git");
        c.env("GIT_CONFIG_GLOBAL", "/dev/null")
            .arg("-C")
            .arg(&w.clone)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());
        let mut child = c.spawn().unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(me.as_bytes())
            .unwrap();
        String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
    };
    let tree = {
        let mut c = Command::new("git");
        c.env("GIT_CONFIG_GLOBAL", "/dev/null")
            .arg("-C")
            .arg(&w.clone)
            .arg("mktree")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());
        let mut child = c.spawn().unwrap();
        use std::io::Write;
        writeln!(
            child.stdin.take().unwrap(),
            "100644 blob {}\towner.toml",
            blob.trim()
        )
        .unwrap();
        String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
    };
    let commit_id = ok(git(&w.clone, &["commit-tree", tree.trim(), "-m", "stale"]));
    ok(git(
        &w.clone,
        &["update-ref", "refs/heads/bridle/state", commit_id.trim()],
    ));
    commit(&w);
    assert!(!push(&w, "main").status.success());
}

#[test]
fn a_symlinked_hook_is_not_written_through() {
    let w = world(Some(&this_host()));
    let shared = w.clone.parent().unwrap().join("shared-pre-push");
    fs::write(&shared, "#!/bin/sh\n# bridle-owner-hook\nold\n").unwrap();
    let hook = w.clone.join(".git/hooks/pre-push");
    fs::remove_file(&hook).unwrap();
    std::os::unix::fs::symlink(&shared, &hook).unwrap();
    let out = bridle(&w.home, &w.clone, &["sync"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("symlink"));
    assert!(fs::read_to_string(&shared).unwrap().contains("old"));
    assert!(
        fs::symlink_metadata(&hook)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn a_foreign_hook_is_left_alone() {
    let w = world(Some(&this_host()));
    let hook = w.clone.join(".git/hooks/pre-push");
    fs::write(&hook, "#!/bin/sh\necho mine\n").unwrap();
    let out = bridle(&w.home, &w.clone, &["sync"]);
    assert!(String::from_utf8_lossy(&out.stderr).contains("not installed"));
    assert_eq!(fs::read_to_string(&hook).unwrap(), "#!/bin/sh\necho mine\n");
}

#[test]
fn install_is_idempotent_and_chains_with_tools_only() {
    let w = world(Some(&this_host()));
    let hook = w.clone.join(".git/hooks/pre-push");
    let first = fs::read_to_string(&hook).unwrap();
    ok(bridle(&w.home, &w.clone, &["sync"]));
    assert_eq!(fs::read_to_string(&hook).unwrap(), first);

    // Listed as tools-only: one script carries both, and refuses on the tools-only reason.
    fs::write(
        w.home.join("config.toml"),
        format!(
            "[machine]\ntools_only = [{:?}]\n",
            w.clone.display().to_string()
        ),
    )
    .unwrap();
    ok(bridle(
        &w.home,
        &w.clone,
        &["machine", "tools-only-install"],
    ));
    let both = fs::read_to_string(&hook).unwrap();
    assert!(both.contains("bridle-tools-only-hook") && both.contains("bridle-owner-hook"));
    commit(&w);
    let out = push(&w, "main");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("tools-only"));
    // A later sync keeps both parts.
    ok(bridle(&w.home, &w.clone, &["sync"]));
    assert_eq!(fs::read_to_string(&hook).unwrap(), both);

    // No longer listed: the tools-only part goes, the owner check stays and the owner can push.
    fs::write(w.home.join("config.toml"), "").unwrap();
    ok(bridle(
        &w.home,
        &w.clone,
        &["machine", "tools-only-install"],
    ));
    let owner_only = fs::read_to_string(&hook).unwrap();
    assert!(owner_only.contains("bridle-owner-hook") && !owner_only.contains("tools-only"));
    ok(push(&w, "main"));
}
