//! `bridle link` (ticket yfjc): UI links for ticket and task IDs from `[gateway] public_url`.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn link(home: &Path, cwd: &Path, args: &[&str]) -> String {
    let out = Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .current_dir(cwd)
        .env("BRIDLE_HOME", home)
        .env_remove("BRIDLE_PROJECT")
        .arg("link")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn home_with(config: &str) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    fs::write(home.path().join("config.toml"), config).unwrap();
    home
}

#[test]
fn a_task_id_links_to_the_task_page() {
    let home = home_with("[gateway]\npublic_url = \"http://ui.example:7878/\"\n");
    let cwd = tempfile::tempdir().unwrap();
    assert_eq!(
        link(home.path(), cwd.path(), &["br-yfjc"]),
        "http://ui.example:7878/task?id=br-yfjc\n"
    );
}

#[test]
fn a_ticket_id_links_to_the_ticket_page_of_the_project() {
    let home = home_with("[gateway]\npublic_url = \"http://ui.example:7878\"\n");
    let cwd = tempfile::tempdir().unwrap();
    assert_eq!(
        link(home.path(), cwd.path(), &["--project", "bridle", "yfjc"]),
        "http://ui.example:7878/ticket?project=bridle&id=yfjc\n"
    );
}

#[test]
fn no_base_url_prints_nothing_and_succeeds() {
    let home = home_with("");
    let cwd = tempfile::tempdir().unwrap();
    assert_eq!(link(home.path(), cwd.path(), &["br-yfjc"]), "");
    assert_eq!(link(home.path(), cwd.path(), &["yfjc"]), "");
}
