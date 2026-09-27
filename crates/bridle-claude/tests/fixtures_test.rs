//! Parses every captured stdout line from spike 01's fixtures
//! (docs/spikes/01-stream-json-findings.md) through the real event parser,
//! to catch regressions against real claude output. These are pre-recorded
//! transcripts, not live runs.

use std::fs;
use std::path::{Path, PathBuf};

use bridle_claude::events::{Event, EventKind};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Replays a transcript's `"dir":"out"` lines (what claude wrote to stdout)
/// through `Event::parse`, in order.
fn out_events(name: &str) -> Vec<Event> {
    let path = fixtures_dir().join(name);
    let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path:?}: {e}"));
    content
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v.get("dir").and_then(|d| d.as_str()) == Some("out"))
        .map(|v| {
            let line = v
                .get("line")
                .and_then(|l| l.as_str())
                .expect("line field")
                .to_string();
            let t_ms = v.get("t_ms").and_then(|t| t.as_u64()).unwrap_or(0);
            Event::parse(t_ms, line)
        })
        .collect()
}

const ALL_FIXTURES: &[&str] = &[
    "s1.jsonl",
    "s2.jsonl",
    "s3.jsonl",
    "s4.jsonl",
    "s5.jsonl",
    "s6.jsonl",
    "s7.jsonl",
    "s8probe.jsonl",
    "s10.jsonl",
    "s11.jsonl",
];

#[test]
fn all_fixture_lines_parse_without_falling_back() {
    for name in ALL_FIXTURES {
        for ev in out_events(name) {
            assert!(
                !matches!(ev.kind, EventKind::Unparsed { .. } | EventKind::NotJson),
                "{name}: {} failed to parse: {:?}\nraw: {}",
                ev.label(),
                ev.kind,
                ev.raw,
            );
        }
    }
}

#[test]
fn s3_has_two_replays() {
    let events = out_events("s3.jsonl");
    let replays: Vec<&str> = events
        .iter()
        .filter_map(|e| match &e.kind {
            EventKind::User(m) if m.is_replay() => m.replay_text(),
            _ => None,
        })
        .collect();
    assert_eq!(
        replays.len(),
        2,
        "expected two replays in s3.jsonl, got {replays:?}"
    );
    assert_eq!(
        replays[0],
        "Run `sleep 8` with the Bash tool, then reply DONE."
    );
    assert_eq!(replays[1], "Also reply BANANA");
}

#[test]
fn rate_limit_windows_parse_five_hour_and_seven_day() {
    let mut found = false;
    for name in ALL_FIXTURES {
        for ev in out_events(name) {
            if let EventKind::RateLimit(r) = &ev.kind {
                let windows = r.windows();
                if windows.iter().any(|w| w.window == "five_hour")
                    && windows.iter().any(|w| w.window == "seven_day")
                {
                    found = true;
                    let five = windows
                        .iter()
                        .find(|w| w.window == "five_hour")
                        .expect("five_hour");
                    assert!(five.utilization.is_some());
                    assert!(five.resets_at_epoch.is_some());
                }
            }
        }
    }
    assert!(
        found,
        "expected at least one rate_limit_event with five_hour + seven_day windows"
    );
}

#[test]
fn result_event_fields_parse_including_error_during_execution() {
    let events = out_events("s4.jsonl");
    let error_results: Vec<_> = events
        .iter()
        .filter_map(|e| match &e.kind {
            EventKind::Result(r) if r.subtype == "error_during_execution" => Some(r.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !error_results.is_empty(),
        "expected an error_during_execution result in s4.jsonl"
    );
    for r in &error_results {
        assert!(r.is_error);
        assert_eq!(r.terminal_reason.as_deref(), Some("aborted_tools"));
        assert!(r.stop_reason.is_some());
        assert!(r.num_turns.is_some());
        assert!(r.usage.is_some());
    }

    let success_results: Vec<_> = events
        .iter()
        .filter_map(|e| match &e.kind {
            EventKind::Result(r) if r.subtype == "success" => Some(r.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !success_results.is_empty(),
        "expected a success result in s4.jsonl too"
    );
    for r in &success_results {
        assert!(!r.is_error);
        assert_eq!(r.stop_reason.as_deref(), Some("end_turn"));
    }
}
