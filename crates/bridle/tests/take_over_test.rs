//! `bridle project take-over` (53gy A): temp bare origin, temp clone, no network, no daemon.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bridle(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bridle"))
        .current_dir(cwd)
        .env("BRIDLE_HOME", home)
        .env_remove("BRIDLE_PROJECT")
        .args(args)
        .output()
        .unwrap()
}

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

fn ok(o: Output) -> String {
    assert!(
        o.status.success(),
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

struct World {
    _dir: tempfile::TempDir,
    home: PathBuf,
    origin: PathBuf,
    clone: PathBuf,
}

/// An origin whose `bridle/state` says `old-box` owns the project `clone`; this machine is
/// `new-box` and `[projects]` still places the project on `old-box`.
fn world() -> World {
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
    ok(git(&seed, &["checkout", "-q", "--orphan", "bridle/state"]));
    fs::write(
        seed.join("owner.toml"),
        "host = \"old-box\"\nsince = \"2026-10-09T14:00:00Z\"\n",
    )
    .unwrap();
    ok(git(&seed, &["add", "owner.toml"]));
    ok(git(&seed, &["commit", "-q", "-m", "owner"]));
    ok(git(&seed, &["push", "-q", "origin", "bridle/state"]));
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
    fs::write(
        home.join("config.toml"),
        "[machine]\nname = \"new-box\"\n\n[projects]\nclone = { machine = \"old-box\", port = 7500 }\n",
    )
    .unwrap();
    World {
        _dir: dir,
        home,
        origin,
        clone,
    }
}

fn origin_owner(w: &World) -> String {
    ok(git(&w.origin, &["show", "bridle/state:owner.toml"]))
}

#[test]
fn take_over_moves_the_owner_and_the_project_entry_then_exits() {
    let w = world();
    let out = bridle(&w.home, &w.clone, &["project", "take-over"]);
    let stdout = ok(out);
    assert!(
        stdout.contains("now owned by new-box (was old-box)"),
        "{stdout}"
    );
    assert!(origin_owner(&w).contains("new-box"));
    let config = fs::read_to_string(w.home.join("config.toml")).unwrap();
    assert!(config.contains("machine = \"new-box\""), "{config}");
    assert!(config.contains("port = 7500"), "{config}");
    // It served nothing.
    assert!(
        !w.clone
            .parent()
            .unwrap()
            .join(".bridle/daemon.json")
            .exists()
    );
}

#[test]
fn a_second_take_over_changes_nothing() {
    let w = world();
    ok(bridle(&w.home, &w.clone, &["project", "take-over"]));
    let tip = ok(git(&w.origin, &["rev-parse", "bridle/state"]));
    let again = ok(bridle(&w.home, &w.clone, &["project", "take-over"]));
    assert!(again.contains("already owned by this machine"), "{again}");
    assert_eq!(ok(git(&w.origin, &["rev-parse", "bridle/state"])), tip);
}

#[test]
fn a_port_in_use_here_moves_up() {
    let w = world();
    fs::write(
        w.home.join("config.toml"),
        "[machine]\nname = \"new-box\"\n\n[projects]\nclone = { machine = \"old-box\", port = 7500 }\nother = { machine = \"new-box\", port = 7500 }\n",
    )
    .unwrap();
    ok(bridle(&w.home, &w.clone, &["project", "take-over"]));
    let config = fs::read_to_string(w.home.join("config.toml")).unwrap();
    assert!(
        config.contains("clone = { machine = \"new-box\", port = 7501 }"),
        "{config}"
    );
}

#[test]
fn serve_as_non_owner_refuses_and_names_the_command() {
    let w = world();
    let out = bridle(&w.home, &w.clone, &["serve"]);
    assert_eq!(out.status.code(), Some(78));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("bridle project take-over"), "{err}");
}

#[test]
fn serve_take_over_is_gone_and_names_the_command() {
    let w = world();
    let out = bridle(&w.home, &w.clone, &["serve", "--take-over"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("bridle project take-over"), "{err}");
}
