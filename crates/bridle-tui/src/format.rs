//! Event-line formatting, mirroring `bridle`'s `render_event_line`
//! (crates/bridle/src/render.rs) so the tail reads the same in `bridle
//! events --follow` and the TUI's event view. bridle-tui can't depend on
//! the `bridle` binary crate, so this is a close copy rather than shared
//! code.

use bridle_api::Event;
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
}
