//! `bridle focus` read form (ticket 2ae8): runs with no daemon, from the local config only.

use std::path::PathBuf;
use std::process::Command;

fn run(home: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(PathBuf::from(env!("CARGO_BIN_EXE_bridle")))
        .args(args)
        .env("BRIDLE_HOME", home)
        .env_remove("BRIDLE_TOKEN")
        .env_remove("BRIDLE_URL")
        .output()
        .expect("run bridle")
}

fn home_with(config: &str) -> tempfile::TempDir {
    let home = tempfile::tempdir().expect("tempdir");
    std::fs::write(home.path().join("config.toml"), config).expect("write config");
    home
}

// Covers every hour but the last minute, so the real clock is almost always inside.
const ALWAYS: &str = "[[focus]]\nname = \"all\"\ndays = \"all\"\nstart = \"00:00\"\nend = \"23:59\"\n\
                      [[focus]]\nname = \"evening\"\ndays = [\"mon\"]\nstart = \"20:00\"\nend = \"22:00\"\nmode = \"locked\"\n";

fn stdout(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

// s-fd51
// s-2fe2
#[test]
fn no_periods_says_focus_is_off_and_succeeds_without_a_daemon() {
    let home = tempfile::tempdir().expect("tempdir");
    let o = run(home.path(), &["focus"]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("focus hours are off"));
}

// s-6dc3
#[test]
fn bare_focus_shows_the_period_and_next_change_and_no_override() {
    let home = home_with(ALWAYS);
    let o = run(home.path(), &["focus"]);
    assert!(o.status.success());
    let out = stdout(&o);
    assert!(out.contains("next change"), "{out}");
    assert!(!out.contains("override"), "{out}");
}

#[test]
fn an_override_file_is_shown_as_pending_with_its_reason() {
    let home = home_with(ALWAYS);
    std::fs::write(
        home.path().join("focus-override.toml"),
        "until = 2099-01-01T00:00:00Z\nreason = \"deploy is broken\"\n",
    )
    .expect("write");
    let out = stdout(&run(home.path(), &["focus"]));
    assert!(out.contains("override pending"), "{out}");
    assert!(out.contains("deploy is broken"), "{out}");
}

#[test]
fn schedule_lists_both_periods() {
    let home = home_with(ALWAYS);
    let out = stdout(&run(home.path(), &["focus", "--schedule"]));
    assert!(out.contains("all") && out.contains("evening"), "{out}");
    assert!(
        out.contains("20:00-22:00") && out.contains("locked"),
        "{out}"
    );
}

// s-e08c
#[test]
fn json_is_one_object_with_the_shared_keys() {
    let home = home_with(ALWAYS);
    let o = run(home.path(), &["--json", "focus"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("one JSON object");
    assert_eq!(v["schedule"][0]["span"]["start"], "00:00");
    assert!(v.get("next_change").is_some());
}

#[test]
fn gate_prints_nothing_with_no_periods() {
    let home = tempfile::tempdir().expect("tempdir");
    let o = run(home.path(), &["focus", "gate"]);
    assert!(o.status.success());
    assert!(stdout(&o).is_empty());
}
