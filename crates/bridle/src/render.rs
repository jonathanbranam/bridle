//! Rendering the daemon's raw output into terse, human-readable lines: the
//! `bridle logs` transcript view and `bridle events` lines. Works directly
//! on `serde_json::Value`, deliberately not depending on bridle-claude
//! (spec: docs/agent-host.md §4.8, §7; the CLI brief for `bridle logs`).

use bridle_api::Event;
use serde::Serialize;
use serde_json::Value;

/// `--json`: the API response as compact JSON, one line.
pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

/// Render one already-JSON-decoded stream-json line (the `line` field of a
/// transcript entry whose `dir` is `"out"`) into zero or more display lines.
///
/// - assistant `text` blocks: `  <text>`
/// - assistant `tool_use` blocks: `→ Name(short input)`
/// - a replayed user message (string content): `« <text>`
/// - `result`: `✓ turn done (subtype, cost)`
/// - `system`/`init`: `— turn start`
/// - everything else (thinking blocks, other system subtypes, tool-result
///   user messages, `rate_limit_event`, …): skipped.
pub fn render_stream_json_line(value: &Value) -> Vec<String> {
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
                let input = block.get("input").map(summarize_input).unwrap_or_default();
                Some(format!("\u{2192} {name}({input})"))
            }
            // "thinking" and any future block type: skipped.
            _ => None,
        })
        .collect()
}

fn render_replayed_user(value: &Value) -> Option<String> {
    // Only a replayed message (string content = the original stdin text,
    // echoed back via --replay-user-messages) is shown; a tool_result user
    // message has array content and is skipped.
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

/// A short one-line summary of a tool call's input, for `tool_use` and for
/// an event's `data` object: compact JSON with the outer braces stripped,
/// truncated.
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

/// One line for `bridle events [--follow]`: `seq time kind actor agent data-summary`.
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

#[cfg(test)]
mod tests {
    use super::*;

    // Real lines from spikes/stream-json/fixtures/s3.jsonl (the `line`
    // field of `dir: "out"` entries), copied verbatim.

    const SYSTEM_INIT: &str = r#"{"type":"system","subtype":"init","cwd":"$TMPDIR/bridle-spike/s3","session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","tools":["Task","Bash"],"mcp_servers":[],"model":"claude-haiku-4-5-20251001","permissionMode":"dontAsk"}"#;

    const REPLAYED_USER: &str = r#"{"type":"user","message":{"role":"user","content":"Run `sleep 8` with the Bash tool, then reply DONE."},"session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","parent_tool_use_id":null,"uuid":"a7c1aeaa-7571-4f7f-86e0-6aefb713b432","timestamp":"2026-09-27T13:50:17.966Z","isReplay":true}"#;

    const ASSISTANT_THINKING_ONLY: &str = r#"{"type":"assistant","message":{"model":"claude-haiku-4-5-20251001","id":"msg_011CfU3xnUCj9Rh3MUMiDT7P","type":"message","role":"assistant","content":[{"type":"thinking","thinking":"","signature":"abc"}],"container":null,"stop_reason":null,"stop_sequence":null,"stop_details":null},"parent_tool_use_id":null,"session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","uuid":"0b94b2e2-1886-4977-89df-fb0cffba016b","timestamp":"2026-09-27T13:50:19.257Z"}"#;

    const ASSISTANT_TOOL_USE: &str = r#"{"type":"assistant","message":{"model":"claude-haiku-4-5-20251001","id":"msg_011CfU3xnUCj9Rh3MUMiDT7P","type":"message","role":"assistant","content":[{"type":"tool_use","id":"toolu_01HLwKKcrzNkRpbiStqByY6g","name":"Bash","input":{"command":"sleep 8","description":"Sleep for 8 seconds"},"caller":{"type":"direct"}}],"container":null,"stop_reason":null,"stop_sequence":null,"stop_details":null},"parent_tool_use_id":null,"session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","uuid":"e654193b-52ef-46d1-9c0c-4a79783621d0","timestamp":"2026-09-27T13:50:19.546Z"}"#;

    const TOOL_RESULT_USER: &str = r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01HLwKKcrzNkRpbiStqByY6g","type":"tool_result","content":"(Bash completed with no output)","is_error":false}]},"parent_tool_use_id":null,"session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","uuid":"34297844-3f53-4b95-b2b0-460fd15eaf83","timestamp":"2026-09-27T13:50:31.440Z"}"#;

    const ASSISTANT_TEXT: &str = r#"{"type":"assistant","message":{"model":"claude-haiku-4-5-20251001","id":"msg_011CfU3ymukZiKonoDfuQomP","type":"message","role":"assistant","content":[{"type":"text","text":"DONE\nBANANA"}],"container":null,"stop_reason":null,"stop_sequence":null,"stop_details":null},"parent_tool_use_id":null,"session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","uuid":"35f77723-f291-4c46-ab7d-f0153f4713d2","timestamp":"2026-09-27T13:50:33.227Z"}"#;

    const RESULT_LINE: &str = r#"{"duration_api_ms":3334,"stop_reason":"end_turn","session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5","total_cost_usd":0.0202318,"permission_denials":[],"terminal_reason":"completed","is_error":false,"num_turns":2,"subtype":"success","result":"DONE\nBANANA","type":"result","duration_ms":15331,"uuid":"a4a62068-fe6a-4727-b0f1-309ba7a9a7de"}"#;

    const RATE_LIMIT_EVENT: &str = r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed"},"uuid":"28a5e757-b135-428d-9810-a3d3d7aadc8e","session_id":"efdb4e0e-d577-445e-bfbd-3ebbe59f28f5"}"#;

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn system_init_renders_as_turn_start() {
        let out = render_stream_json_line(&parse(SYSTEM_INIT));
        assert_eq!(out, vec!["\u{2014} turn start".to_string()]);
    }

    #[test]
    fn replayed_user_message_is_rendered_with_guillemet() {
        let out = render_stream_json_line(&parse(REPLAYED_USER));
        assert_eq!(
            out,
            vec!["\u{ab} Run `sleep 8` with the Bash tool, then reply DONE.".to_string()]
        );
    }

    #[test]
    fn assistant_thinking_block_is_skipped() {
        let out = render_stream_json_line(&parse(ASSISTANT_THINKING_ONLY));
        assert!(out.is_empty());
    }

    #[test]
    fn assistant_tool_use_renders_with_arrow_and_short_input() {
        let out = render_stream_json_line(&parse(ASSISTANT_TOOL_USE));
        assert_eq!(out.len(), 1);
        assert!(out[0].starts_with("\u{2192} Bash("));
        assert!(out[0].contains("sleep 8"));
    }

    #[test]
    fn tool_result_user_message_is_skipped() {
        let out = render_stream_json_line(&parse(TOOL_RESULT_USER));
        assert!(out.is_empty());
    }

    #[test]
    fn assistant_text_block_is_indented() {
        let out = render_stream_json_line(&parse(ASSISTANT_TEXT));
        assert_eq!(out, vec!["  DONE\nBANANA".to_string()]);
    }

    #[test]
    fn result_line_renders_subtype_and_cost() {
        let out = render_stream_json_line(&parse(RESULT_LINE));
        assert_eq!(
            out,
            vec!["\u{2713} turn done (success, $0.0202)".to_string()]
        );
    }

    #[test]
    fn rate_limit_event_is_skipped() {
        let out = render_stream_json_line(&parse(RATE_LIMIT_EVENT));
        assert!(out.is_empty());
    }

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
}
