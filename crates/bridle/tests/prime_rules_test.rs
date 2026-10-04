//! `bridle prime advisor|orchestrator`: the role's resolved rules, project rules included,
//! honoring each rule's `roles:` (br-m7mp).

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn prime(role: &str, repo: &Path, home: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["prime", role])
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
fn interactive_roles_get_project_rules_listing_them() {
    let workflow = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflow")
        .canonicalize()
        .unwrap();
    let repo = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let r = repo.path();
    fs::create_dir_all(r.join(".bridle/rules")).unwrap();
    fs::write(
        r.join(".bridle/config.toml"),
        format!("workflow = {:?}\n", workflow.display().to_string()),
    )
    .unwrap();
    fs::write(
        r.join(".bridle/rules/worked-on-log.md"),
        "---\nid: worked-on-log\nroles: [advisor, orchestrator]\n---\nKeep the worked-on log.\n",
    )
    .unwrap();
    fs::write(
        r.join(".bridle/rules/workers-only.md"),
        "---\nid: workers-only\nroles: [worker]\n---\nWorker-only thing.\n",
    )
    .unwrap();

    for role in ["advisor", "orchestrator"] {
        let out = prime(role, r, home.path());
        assert!(out.contains("Keep the worked-on log."), "{role}: {out}");
        assert!(!out.contains("Worker-only thing."), "{role}: {out}");
    }
    let aide = prime("aide", r, home.path());
    assert!(!aide.contains("Keep the worked-on log."), "{aide}");
}
