//! Tests for `bridle spec coverage` command.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn bridle_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bridle"))
}

#[test]
fn spec_coverage_basic() {
    let temp = TempDir::new().expect("temp dir");
    let root = temp.path();

    // Create design/specs with a spec file
    fs::create_dir_all(root.join("design/specs")).expect("mkdir specs");
    fs::write(
        root.join("design/specs/example.md"),
        r#"# Example

## Requirements

### Requirement: Example {#r-1234}

The system SHALL do things.

#### Scenario: Thing one {#s-b310}

*Verification*: **executable**

- **GIVEN** a thing
- **WHEN** it is done
- **THEN** it worked

#### Scenario: Thing two {#s-b311}

*Verification*: **executable**

- **GIVEN** a thing
- **WHEN** it is tested
- **THEN** it passed
"#,
    )
    .expect("write spec");

    // Create tests directory with a test file that references one scenario
    fs::create_dir_all(root.join("tests")).expect("mkdir tests");
    fs::write(
        root.join("tests/example_test.rs"),
        r#"
// Test for scenario s-b310
#[test]
fn test_thing_one() {
    // something
}
"#,
    )
    .expect("write test");

    // Run the command
    let output = std::process::Command::new(bridle_bin())
        .arg("spec")
        .arg("coverage")
        .arg("--root")
        .arg(root.join("design/specs"))
        .arg("--tests")
        .arg(root.join("tests"))
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run bridle");

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("parse json");

    // Should have 2 executable scenarios
    assert_eq!(report["executable"], 2);
    // Should have 1 bound (s-b310 is referenced in the test)
    assert_eq!(report["bound"], 1);
    // Should have 1 unbound (s-b311 is not referenced)
    assert_eq!(report["unbound"], 1);
    // Check that s-b311 is listed
    let unbound = &report["scenarios"];
    assert_eq!(unbound.as_array().unwrap().len(), 1);
    assert_eq!(unbound[0]["id"], "s-b311");
}

#[test]
fn spec_coverage_all_bound() {
    let temp = TempDir::new().expect("temp dir");
    let root = temp.path();

    // Create design/specs with a spec file
    fs::create_dir_all(root.join("design/specs")).expect("mkdir specs");
    fs::write(
        root.join("design/specs/example.md"),
        r#"# Example

## Requirements

### Requirement: Example {#r-1234}

The system SHALL do things.

#### Scenario: Thing one {#s-b310}

*Verification*: **executable**

- **GIVEN** a thing
- **WHEN** it is done
- **THEN** it worked
"#,
    )
    .expect("write spec");

    // Create tests directory with a test file that references the scenario
    fs::create_dir_all(root.join("tests")).expect("mkdir tests");
    fs::write(
        root.join("tests/example_test.rs"),
        r#"
// Test for scenario s-b310
#[test]
fn test_thing_one() {
    // something
}
"#,
    )
    .expect("write test");

    // Run the command
    let output = std::process::Command::new(bridle_bin())
        .arg("spec")
        .arg("coverage")
        .arg("--root")
        .arg(root.join("design/specs"))
        .arg("--tests")
        .arg(root.join("tests"))
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run bridle");

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("parse json");

    // Should have 1 executable scenario
    assert_eq!(report["executable"], 1);
    // Should have 1 bound
    assert_eq!(report["bound"], 1);
    // Should have 0 unbound
    assert_eq!(report["unbound"], 0);
}

#[test]
fn spec_coverage_require_all() {
    let temp = TempDir::new().expect("temp dir");
    let root = temp.path();

    // Create design/specs with a spec file
    fs::create_dir_all(root.join("design/specs")).expect("mkdir specs");
    fs::write(
        root.join("design/specs/example.md"),
        r#"# Example

## Requirements

### Requirement: Example {#r-1234}

The system SHALL do things.

#### Scenario: Thing one {#s-b310}

*Verification*: **executable**

- **GIVEN** a thing
- **WHEN** it is done
- **THEN** it worked
"#,
    )
    .expect("write spec");

    // Create tests directory with no tests
    fs::create_dir_all(root.join("tests")).expect("mkdir tests");

    // Run the command with --require-all
    let output = std::process::Command::new(bridle_bin())
        .arg("spec")
        .arg("coverage")
        .arg("--root")
        .arg(root.join("design/specs"))
        .arg("--tests")
        .arg(root.join("tests"))
        .arg("--require-all")
        .current_dir(root)
        .output()
        .expect("run bridle");

    // Should exit with code 1 because there are unbound scenarios
    assert_eq!(output.status.code(), Some(1));
}
