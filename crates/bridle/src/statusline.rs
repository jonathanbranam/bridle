//! `bridle statusline`: Claude Code's statusLine command
//! (docs/design/usage-and-budget.md, "Where bridle can see usage" and
//! `docs/design/cli.md`). Claude Code feeds one JSON object on stdin each
//! time it renders the status line; this prints a short line back and turns
//! the JSON into a [`bridle_api::types::StatusLineReport`] for the daemon.
//!
//! Field names below are confirmed against Claude Code's own docs
//! (docs/questions/open/statusline-real-context-and-a-tighter-layout-s8kn.md);
//! anything not confirmed (`rate_limits.*`'s exact shape beyond the windows
//! bridle cares about) still stays tolerant, matching bridle-claude's event
//! model (`crates/bridle-claude/src/events.rs`): walk the raw `Value` for the
//! fields wanted, skip anything missing or reshaped, never error.

use std::path::{Path, PathBuf};

use bridle_api::types::{Status, StatusLineRateLimitReading, StatusLineReport};
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
    let (context_used_percentage, context_used_tokens, context_max_tokens) = context_window(input);

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
        context_used_percentage,
        context_used_tokens,
        context_max_tokens,
        rate_limits,
    }
}

/// `context_window` "describes the live context window from the most recent
/// API response" (Claude Code's docs, quoted in s8kn): `used_percentage` is
/// precomputed and already what's wanted for display, so it's used directly
/// rather than recomputed from token counts — a recomputation that used to
/// wrongly cap out at 200k on extended-context (1M) models. `used_tokens` is
/// the three `current_usage` input fields (input, cache creation, cache
/// read) added together, recorded for `bridle usage`'s history; both
/// `current_usage` and `used_percentage` are `null`/absent before the first
/// API call and right after `/compact`.
fn context_window(input: &Value) -> (Option<f64>, Option<u64>, Option<u64>) {
    let cw = input.get("context_window");
    let used_percentage = cw
        .and_then(|c| c.get("used_percentage"))
        .and_then(Value::as_f64)
        .map(|p| p / 100.0);
    let max_tokens = cw
        .and_then(|c| c.get("context_window_size"))
        .and_then(Value::as_u64);
    let current_usage = cw
        .and_then(|c| c.get("current_usage"))
        .filter(|u| !u.is_null());
    let used_tokens = current_usage.map(|u| {
        [
            "input_tokens",
            "cache_creation_input_tokens",
            "cache_read_input_tokens",
        ]
        .iter()
        .filter_map(|field| u.get(field).and_then(Value::as_u64))
        .sum()
    });
    (used_percentage, used_tokens, max_tokens)
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

/// The session's working directory, for the folder shown in the line:
/// `workspace.current_dir` if present, else the top-level `cwd` Claude Code
/// also sends.
pub fn workspace_dir(input: &Value) -> Option<PathBuf> {
    input
        .pointer("/workspace/current_dir")
        .or_else(|| input.get("cwd"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
}

/// The current branch of the git repo containing `dir`, or `None` if `dir`
/// isn't in one (or in a detached-HEAD state). Claude Code's statusline JSON
/// doesn't carry the branch itself, so this shells out the same way a
/// hand-written statusline script would.
pub fn git_branch(dir: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8(output.stdout).ok()?;
    let branch = branch.trim();
    if branch.is_empty() {
        None
    } else {
        Some(branch.to_string())
    }
}

/// What shows in the human's terminal: model, context %, rate-limit windows
/// (`5h`/`7d` labels), folder and git branch, then the estimated cost last
/// and parenthesized — de-emphasized since it's a list-price estimate that
/// can differ from the real bill and resets on `/clear` (Claude Code's
/// docs, quoted in s8kn). Kept short and never empty, so a mostly-unparsed
/// input still prints something rather than a blank line.
pub fn render_line(
    report: &StatusLineReport,
    folder: Option<&str>,
    branch: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(model) = &report.model {
        parts.push(model.clone());
    }
    if let Some(p) = report.context_used_percentage {
        parts.push(format!("ctx {:.0}%", p * 100.0));
    }
    for rl in &report.rate_limits {
        if let Some(u) = rl.utilization {
            parts.push(format!("{} {:.0}%", window_label(&rl.window), u * 100.0));
        }
    }
    if let Some(folder) = folder {
        parts.push(format!("\u{1F4C1} {folder}"));
    }
    if let Some(branch) = branch {
        parts.push(format!("\u{1F33F} {branch}"));
    }
    if let Some(cost) = report.cost_usd {
        parts.push(format!("(${cost:.2})"));
    }
    if parts.is_empty() {
        "bridle".to_string()
    } else {
        parts.join(" \u{b7} ")
    }
}

/// Bridle's own counts for the human: agents currently working (`working`
/// and `starting`, both "in progress" from the outside) and messages
/// waiting for the human
/// (docs/questions/resolved/statusline-bridle-counts-with-a-read-only-token-r7cs.md).
pub fn render_counts(status: &Status) -> String {
    let working: u32 = ["working", "starting"]
        .iter()
        .filter_map(|s| status.agents_by_state.get(*s))
        .sum();
    format!(
        "{working} working \u{b7} {} for you",
        status.unread_human_messages
    )
}

fn window_label(window: &str) -> &str {
    match window {
        "five_hour" => "5h",
        "seven_day" => "7d",
        "seven_day_opus" => "7d-opus",
        "seven_day_sonnet" => "7d-sonnet",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The documented example shape (s8kn's findings table): `used_percentage`
    /// precomputed, `current_usage` carrying the last call's input fields.
    #[test]
    fn parses_the_documented_shape() {
        let input = json!({
            "session_id": "abc123",
            "model": {"id": "claude-opus-4-1", "display_name": "Opus"},
            "cost": {"total_cost_usd": 1.2345},
            "context_window": {
                "used_percentage": 42.0,
                "context_window_size": 200_000,
                "current_usage": {
                    "input_tokens": 30_000,
                    "cache_creation_input_tokens": 5_000,
                    "cache_read_input_tokens": 5_000,
                    "output_tokens": 1_200
                },
                "total_input_tokens": 40_000,
                "total_output_tokens": 1_200
            },
            "rate_limits": {
                "five_hour": {"used_percentage": 42.0, "resets_at": "2026-09-27T20:00:00Z"},
                "seven_day": {"used_percentage": 10.5, "resets_at": 1790533200},
            },
        });
        let report = parse(&input);
        assert_eq!(report.session_id.as_deref(), Some("abc123"));
        assert_eq!(report.model.as_deref(), Some("Opus"));
        assert_eq!(report.cost_usd, Some(1.2345));
        assert_eq!(report.context_used_percentage, Some(0.42));
        assert_eq!(report.context_used_tokens, Some(40_000));
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

        let line = render_line(&report, Some("bridle"), Some("bridle/s8kn-statusline"));
        assert!(line.contains("Opus"), "{line}");
        assert!(line.contains("(\u{24}1.23)"), "{line}");
        assert!(line.contains("ctx 42%"), "{line}");
        assert!(line.contains("5h 42%"), "{line}");
        assert!(line.contains("7d 11%") || line.contains("7d 10%"), "{line}");
        assert!(line.contains("\u{1F4C1} bridle"), "{line}");
        assert!(line.contains("\u{1F33F} bridle/s8kn-statusline"), "{line}");
        // The cost is de-emphasized: parenthesized and last.
        assert!(line.trim_end().ends_with(')'), "{line}");
    }

    #[test]
    fn null_current_usage_after_compact_reports_no_used_tokens() {
        let input = json!({
            "context_window": {
                "used_percentage": 0.0,
                "context_window_size": 200_000,
                "current_usage": null
            }
        });
        let report = parse(&input);
        assert_eq!(report.context_used_percentage, Some(0.0));
        assert_eq!(report.context_used_tokens, None);
        assert_eq!(report.context_max_tokens, Some(200_000));
    }

    #[test]
    fn extended_context_model_is_not_capped_at_200k() {
        // A 1M-context model at 200k used tokens is 20%, not the old
        // exceeds_200k_tokens fallback's 100%.
        let input = json!({
            "context_window": {
                "used_percentage": 20.0,
                "context_window_size": 1_000_000,
                "current_usage": {
                    "input_tokens": 200_000,
                    "cache_creation_input_tokens": 0,
                    "cache_read_input_tokens": 0
                }
            },
            "exceeds_200k_tokens": true
        });
        let report = parse(&input);
        assert_eq!(report.context_used_percentage, Some(0.20));
        assert_eq!(report.context_used_tokens, Some(200_000));
        assert_eq!(report.context_max_tokens, Some(1_000_000));
    }

    #[test]
    fn empty_input_never_errors_and_still_prints_something() {
        let report = parse(&Value::Null);
        assert!(report.model.is_none());
        assert!(report.rate_limits.is_empty());
        assert_eq!(render_line(&report, None, None), "bridle");
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

    fn sample_status(agents_by_state: &[(&str, u32)], unread: u32) -> Status {
        Status {
            daemon: bridle_api::types::DaemonInfo {
                project: "demo".to_string(),
                workspace: "/ws".to_string(),
                repo: "/ws/repo".to_string(),
                url: "http://127.0.0.1:0".to_string(),
                pid: 1,
                started_at: Utc::now(),
                version: "0.1.0".to_string(),
            },
            principal: "external:statusline".to_string(),
            agents_by_state: agents_by_state
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
            unread_human_messages: unread,
            rate_limits: Vec::new(),
            claude_version: None,
            budget_state: Default::default(),
        }
    }

    #[test]
    fn render_counts_sums_working_and_starting() {
        let status = sample_status(&[("working", 2), ("starting", 1), ("idle", 5)], 3);
        assert_eq!(render_counts(&status), "3 working \u{b7} 3 for you");
    }

    #[test]
    fn render_counts_handles_no_agents_or_messages() {
        let status = sample_status(&[], 0);
        assert_eq!(render_counts(&status), "0 working \u{b7} 0 for you");
    }

    #[test]
    fn workspace_dir_prefers_workspace_current_dir_over_top_level_cwd() {
        let input = json!({"cwd": "/top", "workspace": {"current_dir": "/nested"}});
        assert_eq!(workspace_dir(&input), Some(PathBuf::from("/nested")));

        let input = json!({"cwd": "/top"});
        assert_eq!(workspace_dir(&input), Some(PathBuf::from("/top")));

        assert_eq!(workspace_dir(&Value::Null), None);
    }
}
