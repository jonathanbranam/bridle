//! `bridle link` (ticket yfjc): UI links for tickets, documents and specs from `[gateway] public_url`.

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
fn a_ticket_id_links_to_the_ticket_page_of_the_project() {
    let home = home_with("[gateway]\npublic_url = \"http://ui.example:7878/\"\n");
    let cwd = tempfile::tempdir().unwrap();
    for id in ["br-yfjc", "yfjc"] {
        assert_eq!(
            link(home.path(), cwd.path(), &["--project", "bridle", id]),
            format!("http://ui.example:7878/p/bridle/tickets/{id}\n")
        );
    }
}

#[test]
fn a_document_path_links_to_the_document_page() {
    let home = home_with("[gateway]\npublic_url = \"http://ui.example:7878\"\n");
    let cwd = tempfile::tempdir().unwrap();
    assert_eq!(
        link(
            home.path(),
            cwd.path(),
            &["--project", "bridle", "docs/design/cli.md"]
        ),
        "http://ui.example:7878/p/bridle/docs?path=docs/design/cli.md\n"
    );
}

#[test]
fn a_spec_id_links_to_its_spec_file_and_anchor() {
    let home = home_with("[gateway]\npublic_url = \"http://ui.example:7878\"\n");
    let repo = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(repo.path())
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q"]);
    fs::create_dir_all(repo.path().join("design/specs")).unwrap();
    fs::write(
        repo.path().join("design/specs/demo.md"),
        "# Demo\n\n## Requirements\n\n### Requirement: It works  {#r-1a2b}\n\nIt SHALL work.\n\n\
         #### Scenario: Runs  {#s-3c4d}\n\n*Verification*: **executable**\n\n\
         - **GIVEN** a\n- **WHEN** b\n- **THEN** c\n",
    )
    .unwrap();
    assert_eq!(
        link(home.path(), repo.path(), &["--project", "bridle", "s-3c4d"]),
        "http://ui.example:7878/p/bridle/specs?path=design/specs/demo.md#s-3c4d\n"
    );
}

#[test]
fn no_base_url_prints_nothing_and_succeeds() {
    let home = home_with("");
    let cwd = tempfile::tempdir().unwrap();
    assert_eq!(link(home.path(), cwd.path(), &["br-yfjc"]), "");
    assert_eq!(link(home.path(), cwd.path(), &["yfjc"]), "");
}
