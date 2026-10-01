use std::process::Command;

#[test]
fn test_completions_zsh_generation() {
    let output = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["completions", "zsh"])
        .env_clear()
        .output()
        .expect("failed to generate zsh completions");

    assert!(
        output.status.success(),
        "completions command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let completions = String::from_utf8(output.stdout).expect("invalid utf8");
    assert!(
        !completions.is_empty(),
        "zsh completions should not be empty"
    );
    assert!(
        completions.contains("task"),
        "zsh completions should mention 'task'"
    );
    assert!(
        completions.contains("--kind"),
        "zsh completions should mention '--kind' flag"
    );
}

#[test]
fn test_completions_bash_generation() {
    let output = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["completions", "bash"])
        .env_clear()
        .output()
        .expect("failed to generate bash completions");

    assert!(
        output.status.success(),
        "completions command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let completions = String::from_utf8(output.stdout).expect("invalid utf8");
    assert!(
        !completions.is_empty(),
        "bash completions should not be empty"
    );
    assert!(
        completions.contains("task"),
        "bash completions should mention 'task'"
    );
    assert!(
        completions.contains("--kind"),
        "bash completions should mention '--kind' flag"
    );
}

#[test]
fn test_completions_fish_generation() {
    let output = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["completions", "fish"])
        .env_clear()
        .output()
        .expect("failed to generate fish completions");

    assert!(
        output.status.success(),
        "completions command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let completions = String::from_utf8(output.stdout).expect("invalid utf8");
    assert!(
        !completions.is_empty(),
        "fish completions should not be empty"
    );
    assert!(
        completions.contains("task"),
        "fish completions should mention 'task'"
    );
    assert!(
        completions.contains("--kind"),
        "fish completions should mention '--kind' flag"
    );
}

#[test]
fn test_completions_no_daemon_required() {
    let output = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["completions", "zsh"])
        .env_clear()
        .output()
        .expect("failed to generate completions");

    assert!(
        output.status.success(),
        "completions should work without daemon: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
