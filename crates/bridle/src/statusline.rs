//! `bridle statusline`: Claude Code's statusLine command
//! (docs/design/usage-and-budget.md, "Where bridle can see usage" and
//! `docs/design/cli.md`). Claude Code feeds one JSON object on stdin each
//! time it renders the status line; this prints a short line back and turns
//! the JSON into a [`bridle_api::types::StatusLineReport`] for the daemon.
//!
//! The design doc paraphrases the schema as carrying `rate_limits.five_hour`
//! / `.seven_day`, each with `used_percentage` and `resets_at`, plus session
//! cost and context-window use, but doesn't pin down the exact field names,
//! and Claude Code's own docs weren't reachable from the environment this
//! was built in. So parsing stays tolerant, matching bridle-claude's event
//! model (`crates/bridle-claude/src/events.rs`): walk the raw `Value` for
//! the fields wanted, skip anything missing or reshaped, never error.

use bridle_api::types::{StatusLineRateLimitReading, StatusLineReport};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// The windows the design doc names; any other key under `rate_limits` in
/// the stdin JSON is ignored rather than guessed at.
const WINDOWS: &[&str] = &[
    "five_hour",
    "seven_day",
    "seven_day_opus",
    "seven_day_sonnet",
];

pub fn parse(input: &Value) -> StatusLineReport {
    let model = input
        .pointer("/model/display_name")
        .and_then(Value::as_str)
        .or_else(|| input.pointer("/model/id").and_then(Value::as_str))
        .map(String::from);
    let session_id = input
        .get("session_id")
        .and_then(Value::as_str)
        .map(String::from);
    let cost_usd = input
        .pointer("/cost/total_cost_usd")
        .and_then(Value::as_f64);
    let (context_used_tokens, context_max_tokens) = context_window(input);

    let rate_limits = WINDOWS
        .iter()
        .filter_map(|window| {
            let w = input.pointer(&format!("/rate_limits/{window}"))?;
            let used_percentage = w.get("used_percentage").and_then(Value::as_f64);
            let resets_at = w.get("resets_at").and_then(parse_resets_at);
            if used_percentage.is_none() && resets_at.is_none() {
                return None;
            }
            Some(StatusLineRateLimitReading {
                window: (*window).to_string(),
                utilization: used_percentage.map(|p| p / 100.0),
                resets_at,
            })
        })
        .collect();

    StatusLineReport {
        session_id,
        model,
        cost_usd,
        context_used_tokens,
        context_max_tokens,
        rate_limits,
    }
}

/// Tried under both a nested `context_window` object and Claude Code's
/// documented `exceeds_200k_tokens` flag, since only one of the two shapes
/// may exist depending on version.
fn context_window(input: &Value) -> (Option<u64>, Option<u64>) {
    let used = input
        .pointer("/context_window/used_tokens")
        .and_then(Value::as_u64);
    let max = input
        .pointer("/context_window/max_tokens")
        .and_then(Value::as_u64);
    if used.is_some() || max.is_some() {
        return (used, max);
    }
    // No token counts, just a boolean: report it as "at" the 200k ceiling.
    match input.get("exceeds_200k_tokens").and_then(Value::as_bool) {
        Some(true) => (Some(200_000), Some(200_000)),
        _ => (None, None),
    }
}

/// Either an RFC3339 string or epoch seconds, since it's undocumented which
/// one the real field uses (bridle-claude's `resetsAt` is epoch seconds on
/// the stream-json wire, but statusline JSON is meant for display).
fn parse_resets_at(v: &Value) -> Option<DateTime<Utc>> {
    if let Some(s) = v.as_str() {
        return DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc));
    }
    v.as_i64()
        .and_then(|epoch| DateTime::from_timestamp(epoch, 0))
}

/// What shows in the human's terminal. Kept short and never empty, so a
/// mostly-unparsed input still prints something rather than a blank line.
pub fn render_line(report: &StatusLineReport) -> String {
    let mut parts = Vec::new();
    if let Some(model) = &report.model {
        parts.push(model.clone());
    }
    if let Some(cost) = report.cost_usd {
        parts.push(format!("${cost:.2}"));
    }
    if let (Some(used), Some(max)) = (report.context_used_tokens, report.context_max_tokens)
        && max > 0
    {
        parts.push(format!("ctx {:.0}%", (used as f64 / max as f64) * 100.0));
    }
    for rl in &report.rate_limits {
        if let Some(u) = rl.utilization {
            parts.push(format!("{} {:.0}%", rl.window, u * 100.0));
        }
    }
    if parts.is_empty() {
        "bridle".to_string()
    } else {
        parts.join(" \u{b7} ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_the_documented_shape() {
        let input = json!({
            "session_id": "abc123",
            "model": {"id": "claude-opus-4-1", "display_name": "Opus"},
            "cost": {"total_cost_usd": 1.2345},
            "context_window": {"used_tokens": 50_000, "max_tokens": 200_000},
            "rate_limits": {
                "five_hour": {"used_percentage": 42.0, "resets_at": "2026-09-27T20:00:00Z"},
                "seven_day": {"used_percentage": 10.5, "resets_at": 1790533200},
            },
        });
        let report = parse(&input);
        assert_eq!(report.session_id.as_deref(), Some("abc123"));
        assert_eq!(report.model.as_deref(), Some("Opus"));
        assert_eq!(report.cost_usd, Some(1.2345));
        assert_eq!(report.context_used_tokens, Some(50_000));
        assert_eq!(report.context_max_tokens, Some(200_000));
        assert_eq!(report.rate_limits.len(), 2);
        let five = report
            .rate_limits
            .iter()
            .find(|w| w.window == "five_hour")
            .expect("five_hour");
        assert_eq!(five.utilization, Some(0.42));
        assert!(five.resets_at.is_some());
        let seven = report
            .rate_limits
            .iter()
            .find(|w| w.window == "seven_day")
            .expect("seven_day");
        assert_eq!(seven.utilization, Some(0.105));
        assert!(seven.resets_at.is_some());

        let line = render_line(&report);
        assert!(line.contains("Opus"), "{line}");
        assert!(line.contains("$1.23"), "{line}");
        assert!(line.contains("ctx 25%"), "{line}");
        assert!(line.contains("five_hour 42%"), "{line}");
    }

    #[test]
    fn empty_input_never_errors_and_still_prints_something() {
        let report = parse(&Value::Null);
        assert!(report.model.is_none());
        assert!(report.rate_limits.is_empty());
        assert_eq!(render_line(&report), "bridle");
    }

    #[test]
    fn unrelated_fields_and_a_reshaped_window_are_ignored() {
        let input = json!({
            "hook_event_name": "Status",
            "workspace": {"current_dir": "/tmp"},
            "rate_limits": {
                "five_hour": {"some_new_field": true},
                "unknown_window": {"used_percentage": 99.0},
            },
        });
        let report = parse(&input);
        assert!(report.rate_limits.is_empty());
    }

    #[test]
    fn falls_back_to_exceeds_200k_flag() {
        let input = json!({"exceeds_200k_tokens": true});
        let report = parse(&input);
        assert_eq!(report.context_used_tokens, Some(200_000));
        assert_eq!(report.context_max_tokens, Some(200_000));
    }
}
