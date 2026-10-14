//! `bridle usage --week` and `--by name|session` against a fake daemon that routes by path.

#![allow(clippy::unwrap_used)]

use std::io::{Read, Write};
use std::process::Command;

const PERIODS: &str = r#"{"timezone":"America/New_York","weeks":[{"starts_at":"2026-10-07T12:00:00Z","resets_at":"2026-10-14T12:00:00Z","peak":0.62,"hit_limit":false,"days":[{"date":"2026-10-13","starts_at":"2026-10-13T04:00:00Z","ends_at":"2026-10-14T04:00:00Z","peak_seven_day":0.62,"peak_five_hour":1.0,"hit_five_hour":true}]}],"windows":[{"starts_at":"2026-10-13T10:00:00Z","resets_at":"2026-10-13T15:00:00Z","peak":1.0,"hit_limit":true}],"five_hour":[],"seven_day":[]}"#;

const GROUPS: &str = r#"{"project":"p","machine":"m","total_cost_usd":3.0,"seven_day_points":6.0,"points_per_dollar":2.0,"share_note":"share_points are estimates","roles":[{"key":"worker","project":"p","machine":"m","turns":2,"tokens":{"input":0,"output":0,"cache_read":0,"cache_write":0},"cost_usd":3.0,"context_growth":10,"share_points":6.0,"children":[{"key":"v58q","project":"p","machine":"m","turns":2,"tokens":{"input":0,"output":0,"cache_read":0,"cache_write":0},"cost_usd":3.0,"context_growth":10,"share_points":6.0,"children":[{"key":"sess-1","project":"p","machine":"m","turns":2,"tokens":{"input":0,"output":0,"cache_read":0,"cache_write":0},"cost_usd":3.0,"context_growth":10,"share_points":6.0,"agent_id":"a-1","children":[]}]}]}]}"#;

fn fake_daemon() -> String {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    std::thread::spawn(move || {
        for mut c in l.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let n = c.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let body = if req.contains("/v1/usage/periods") {
                PERIODS
            } else if req.contains("/v1/usage/groups") {
                GROUPS
            } else {
                "{}"
            };
            let _ = write!(
                c,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    url
}

fn usage(args: &[&str]) -> String {
    let url = fake_daemon();
    let out = Command::new(env!("CARGO_BIN_EXE_bridle"))
        .args(["--project", "p", "--url", &url, "--token", "t", "usage"])
        .args(args)
        .current_dir(tempfile::tempdir().unwrap().keep())
        .env("BRIDLE_HOME", tempfile::tempdir().unwrap().keep())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn week_prints_days_windows_peaks_and_hits() {
    let out = usage(&["--week"]);
    assert!(out.contains("peak 62%"), "{out}");
    assert!(
        out.contains("2026-10-13") && out.contains("5h limit"),
        "{out}"
    );
    assert!(
        out.contains("2026-10-13 15:00") && out.contains("limit"),
        "{out}"
    );
}

#[test]
fn by_name_hides_sessions_and_by_session_shows_them() {
    let name = usage(&["--by", "name"]);
    assert!(name.contains("worker") && name.contains("v58q"), "{name}");
    assert!(!name.contains("sess-1"), "{name}");
    assert!(name.contains("estimates"), "{name}");
    let session = usage(&["--by", "session"]);
    assert!(session.contains("sess-1"), "{session}");
}

#[test]
fn week_and_groups_pass_json_through() {
    let v: serde_json::Value = serde_json::from_str(&usage(&["--json", "--week"])).unwrap();
    assert_eq!(v["weeks"][0]["peak"], 0.62);
    let v: serde_json::Value = serde_json::from_str(&usage(&["--json", "--by", "name"])).unwrap();
    assert_eq!(v["roles"][0]["children"][0]["key"], "v58q");
}
