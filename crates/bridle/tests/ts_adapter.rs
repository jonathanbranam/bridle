//! Runs the typescript pack's vitest adapter tests (plain `node --test`, no
//! npm install) against this build of bridle. Skipped when node is missing.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn vitest_adapter_node_tests() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("skipping: node not available");
        return;
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflow/packs/typescript/adapters/vitest-bridle");
    let out = Command::new("node")
        .args(["--test", "test/adapter.test.mjs"])
        .current_dir(dir)
        .env("BRIDLE_BIN", env!("CARGO_BIN_EXE_bridle"))
        .output()
        .expect("run node");
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
