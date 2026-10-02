//! `bridle gateway` reads its own config section; a bad one fails only that command.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::{Command, Output};

fn bridle(home: &std::path::Path, args: &[&str]) -> Output {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .env("BRIDLE_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn bad_gateway_config_fails_only_the_gateway() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[gateway]\nbind = \"0.0.0.0:7878\"\n",
    )
    .unwrap();

    let out = bridle(home.path(), &["gateway"]);
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("[gateway] bind"), "{err}");

    let out = bridle(home.path(), &["daemons"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("[gateway]"), "{err}");
}

#[test]
fn malformed_config_fails_only_the_gateway() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join("config.toml"), "[gateway\n").unwrap();
    let out = bridle(home.path(), &["gateway"]);
    assert_eq!(out.status.code(), Some(1));
    let out = bridle(home.path(), &["daemons"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
