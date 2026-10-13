//! `bridle dev site`: two sites at once, login, the fixtures through the gateway, `down` of one
//! leaving the other, and nothing written to the real home. Real daemons and gateways on free
//! ports, no `claude`, no network beyond loopback.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Command, Output};

fn bridle(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_bridle"));
    c.current_dir(cwd).env("HOME", home).args(args);
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("BRIDLE_") {
            c.env_remove(k);
        }
    }
    c.output().unwrap()
}

fn up(home: &Path, cwd: &Path, name: &str) -> serde_json::Value {
    let o = bridle(
        home,
        cwd,
        &["dev", "site", "up", "--name", name, "--no-ui", "--json"],
    );
    assert!(
        o.status.success(),
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    serde_json::from_slice(&o.stdout).unwrap()
}

fn curl(args: &[&str]) -> String {
    let o = Command::new("curl")
        .args(["-sf", "-m", "5"])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "curl {args:?}");
    String::from_utf8(o.stdout).unwrap()
}

#[test]
fn two_sites_log_in_serve_the_fixtures_and_go_down_one_at_a_time() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let a = up(home.path(), cwd.path(), "one");
    let b = up(home.path(), cwd.path(), "two");
    assert_ne!(a["gateway_port"], b["gateway_port"]);

    // Log in and read the projects: the two fixtures, nothing else.
    let port = a["gateway_port"].as_u64().unwrap();
    let jar = cwd.path().join("jar");
    let jar = jar.to_str().unwrap();
    let base = format!("http://127.0.0.1:{port}/api/v1");
    let login = format!(
        r#"{{"username":"agent","password":"{}"}}"#,
        a["password"].as_str().unwrap()
    );
    curl(&[
        "-c",
        jar,
        "-H",
        "content-type: application/json",
        "-d",
        &login,
        &format!("{base}/login"),
    ]);
    let projects: serde_json::Value =
        serde_json::from_str(&curl(&["-b", jar, &format!("{base}/projects")])).unwrap();
    let mut names: Vec<&str> = projects["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["project"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["acme-docs", "acme-shop"]);

    // Down of one leaves the other answering.
    let o = bridle(home.path(), cwd.path(), &["dev", "site", "down", "one"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!cwd.path().join(".agent-site/one").exists());
    let other = b["gateway_port"].as_u64().unwrap();
    curl(&[&format!("http://127.0.0.1:{other}/api/v1/health")]);

    let o = bridle(home.path(), cwd.path(), &["dev", "site", "down", "--all"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let list = bridle(home.path(), cwd.path(), &["dev", "site", "list"]);
    assert!(String::from_utf8_lossy(&list.stdout).contains("no sites"));

    // Nothing reached the real home.
    assert!(!home.path().join(".bridle").exists());
}
