//! Event- and transcript-line formatting, mirroring `bridle`'s
//! `render_event_line` and `print_transcript_line`/`render_stream_json_line`
//! (crates/bridle/src/render.rs, crates/bridle/src/commands.rs) so the tail
//! reads the same in `bridle events --follow`/`bridle logs --follow` and the
//! TUI's views. bridle-tui can't depend on the `bridle` binary crate, so
//! this is a close copy rather than shared code.

use bridle_api::{Event, TranscriptLine};
use serde_json::Value;

/// One line for the event tail: `seq time kind actor agent data-summary`.
pub fn render_event_line(ev: &Event) -> String {
    let agent = ev.agent.as_deref().unwrap_or("-");
    let data = summarize_input(&ev.data);
    format!(
        "{:>6}  {}  {:<22} {:<18} {:<10} {}",
        ev.seq,
        ev.ts.format("%Y-%m-%dT%H:%M:%SZ"),
        ev.kind,
        ev.actor,
        agent,
        data
    )
}

fn summarize_input(input: &Value) -> String {
    let compact = serde_json::to_string(input).unwrap_or_default();
    let inner = compact
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(&compact);
    truncate(inner, 60)
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let head: String = s.chars().take(max_chars).collect();
        format!("{head}\u{2026}")
    }
}

/// Zero or more display lines for one transcript entry, matching `bridle
/// logs`'s non-`--raw` rendering: `in`/`err`/`note` lines and unparseable
/// `out` lines are skipped, and a stream-json `out` line renders via
/// [`render_stream_json_line`].
pub fn render_transcript_line(line: &TranscriptLine) -> Vec<String> {
    if line.dir != "out" {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<Value>(&line.line) else {
        return Vec::new();
    };
    render_stream_json_line(&value)
}

/// One already-JSON-decoded stream-json line into zero or more display
/// lines: assistant `text`/`tool_use` blocks, a replayed user message,
/// `result`, and `system`/`init`; everything else (thinking blocks, other
/// system subtypes, tool-result user messages, `rate_limit_event`, …) is
/// skipped.
fn render_stream_json_line(value: &Value) -> Vec<String> {
    match value.get("type").and_then(Value::as_str) {
        Some("assistant") => render_assistant_blocks(value),
        Some("user") => render_replayed_user(value).into_iter().collect(),
        Some("result") => vec![render_result(value)],
        Some("system") => render_system_init(value).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn render_assistant_blocks(value: &Value) -> Vec<String> {
    let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
        return Vec::new();
    };
    blocks
        .iter()
        .filter_map(|block| match block.get("type").and_then(Value::as_str) {
            Some("text") => block
                .get("text")
                .and_then(Value::as_str)
                .map(|t| format!("  {t}")),
            Some("tool_use") => {
                let name = block.get("name").and_then(Value::as_str).unwrap_or("?");
                let input = block
                    .get("input")
                    .map(tool_input_summary)
                    .unwrap_or_default();
                Some(format!("\u{2192} {name}({input})"))
            }
            _ => None,
        })
        .collect()
}

fn render_replayed_user(value: &Value) -> Option<String> {
    let text = value.pointer("/message/content")?.as_str()?;
    Some(format!("\u{ab} {text}"))
}

fn render_result(value: &Value) -> String {
    let subtype = value.get("subtype").and_then(Value::as_str).unwrap_or("?");
    let cost = value
        .get("total_cost_usd")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    format!("\u{2713} turn done ({subtype}, ${cost:.4})")
}

fn render_system_init(value: &Value) -> Option<String> {
    (value.get("subtype").and_then(Value::as_str) == Some("init"))
        .then(|| "\u{2014} turn start".to_string())
}

/// A short one-line summary of a tool call's input for `tool_use` blocks:
/// the `command`, `file_path` or `pattern` key's string value if there is
/// one (truncated), else the compact JSON.
fn tool_input_summary(input: &Value) -> String {
    for key in ["command", "file_path", "pattern"] {
        if let Some(s) = input.get(key).and_then(Value::as_str) {
            return truncate(s, 60);
        }
    }
    summarize_input(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_line_includes_seq_kind_actor_and_agent() {
        let ev: Event = serde_json::from_value(serde_json::json!({
            "seq": 1042,
            "ts": "2026-09-27T14:03:11.120Z",
            "kind": "message.delivered",
            "actor": "human",
            "agent": "w1",
            "data": {"message": "m-0042"}
        }))
        .unwrap();
        let line = render_event_line(&ev);
        assert!(line.contains("1042"));
        assert!(line.contains("message.delivered"));
        assert!(line.contains("human"));
        assert!(line.contains("w1"));
        assert!(line.contains("m-0042"));
    }

    fn transcript_line(n: u64, dir: &str, line: &str) -> TranscriptLine {
        TranscriptLine {
            n,
            t_ms: 0,
            dir: dir.to_string(),
            line: line.to_string(),
        }
    }

    #[test]
    fn transcript_line_renders_assistant_text_and_tool_use() {
        let line = transcript_line(
            1,
            "out",
            r#"{"type":"assistant","message":{"content":[
                {"type":"text","text":"hello"},
                {"type":"tool_use","name":"Bash","input":{"command":"ls"}}
            ]}}"#,
        );
        let rendered = render_transcript_line(&line);
        assert_eq!(rendered, vec!["  hello", "\u{2192} Bash(ls)"]);
    }

    #[test]
    fn transcript_line_renders_result() {
        let line = transcript_line(
            2,
            "out",
            r#"{"type":"result","subtype":"success","total_cost_usd":0.01}"#,
        );
        assert_eq!(
            render_transcript_line(&line),
            vec!["\u{2713} turn done (success, $0.0100)"]
        );
    }

    #[test]
    fn transcript_line_skips_non_out_lines() {
        let line = transcript_line(3, "in", r#"{"type":"user"}"#);
        assert!(render_transcript_line(&line).is_empty());
    }

    #[test]
    fn transcript_line_skips_thinking_only_blocks() {
        let line = transcript_line(
            4,
            "out",
            r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"hmm"}]}}"#,
        );
        assert!(render_transcript_line(&line).is_empty());
    }
}
