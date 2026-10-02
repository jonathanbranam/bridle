//! `bridle prime prototyper`: the base role file, then the project's own append.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn prime(repo: &Path, home: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["prime", "prototyper"])
        .current_dir(repo)
        .env("BRIDLE_HOME", home)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn prime_prints_the_real_role_and_the_project_append() {
    let workflow = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflow")
        .canonicalize()
        .unwrap();
    let repo = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(repo.path().join(".bridle/roles")).unwrap();
    fs::write(
        repo.path().join(".bridle/config.toml"),
        format!("workflow = {:?}\n", workflow.display().to_string()),
    )
    .unwrap();

    let out = prime(repo.path(), home.path());
    assert!(
        out.contains("The prototype prompt is the whole brief"),
        "{out}"
    );
    assert!(out.contains("genuinely different approach"), "{out}");
    assert!(out.contains("The project decides"), "{out}");

    fs::write(
        repo.path().join(".bridle/roles/prototyper.md"),
        "Prototypes live in proto/<name>/.\n",
    )
    .unwrap();
    let out = prime(repo.path(), home.path());
    assert!(out.contains("The project decides"), "{out}");
    assert!(out.contains("Prototypes live in proto/<name>/."), "{out}");
}
