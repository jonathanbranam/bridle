//! Tolerant event model for `claude`'s stream-json stdout. Every stdout line
//! becomes an [`Event`] that keeps its raw text and parsed [`Value`]; `kind`
//! is a typed view for the variants a host needs to act on, with `Unknown` /
//! `Unparsed` / `NotJson` fallbacks so a new or reshaped event never breaks
//! the reader. See docs/spikes/01-stream-json-findings.md for the shapes
//! this was built against.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Event {
    pub t_ms: u64,
    pub raw: String,
    pub value: Value,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    SystemInit(SystemInit),
    /// Any other `system` subtype (api_retry, hook events, status, …).
    System {
        subtype: String,
    },
    Assistant(Message),
    User(Message),
    Result(ResultEvent),
    RateLimit(RateLimitEvent),
    ControlResponse(ControlResponse),
    /// claude asking the host something (permission prompts, hook callbacks).
    /// v1 never sees this in practice (`--permission-prompts none`), but the
    /// wire format allows it.
    ControlRequest(Value),
    /// Valid JSON with a `type` we don't model.
    Unknown {
        r#type: String,
    },
    /// Known `type`, but the typed view failed; the error is kept for
    /// diagnostics.
    Unparsed {
        r#type: String,
        error: String,
    },
    /// Not JSON at all.
    NotJson,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemInit {
    pub session_id: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    /// Kept raw: shape (strings vs objects) varies by version.
    #[serde(default)]
    pub capabilities: Option<Value>,
    #[serde(default)]
    pub claude_code_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub parent_tool_use_id: Option<String>,
    pub message: MessageBody,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageBody {
    /// A string for echoed user text (`--replay-user-messages`), an array
    /// of content blocks otherwise.
    #[serde(default)]
    pub content: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Thinking {},
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(default)]
        is_error: Option<bool>,
    },
    #[serde(other)]
    Other,
}

impl Message {
    pub fn blocks(&self) -> Vec<ContentBlock> {
        match &self.message.content {
            Value::Array(items) => items
                .iter()
                .map(|b| serde_json::from_value(b.clone()).unwrap_or(ContentBlock::Other))
                .collect(),
            Value::String(s) => vec![ContentBlock::Text { text: s.clone() }],
            _ => vec![],
        }
    }

    pub fn text(&self) -> String {
        self.blocks()
            .into_iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn tool_uses(&self) -> Vec<(String, String, Value)> {
        self.blocks()
            .into_iter()
            .filter_map(|b| match b {
                ContentBlock::ToolUse { id, name, input } => Some((id, name, input)),
                _ => None,
            })
            .collect()
    }

    /// A `user` event whose `message.content` is a plain JSON string. That's
    /// how `--replay-user-messages` echoes stdin text back on stdout, at the
    /// moment the model is about to see it (docs/agent-host.md §4.3). Tool
    /// results and the synthetic "interrupted" marker are content *arrays*,
    /// never plain strings, so this never false-positives on them.
    pub fn is_replay(&self) -> bool {
        matches!(self.message.content, Value::String(_))
    }

    /// The echoed text, if this is a replay (see [`Message::is_replay`]).
    pub fn replay_text(&self) -> Option<&str> {
        self.message.content.as_str()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResultEvent {
    pub subtype: String,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub num_turns: Option<u64>,
    pub session_id: String,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default, rename = "modelUsage")]
    pub model_usage: Option<Value>,
    #[serde(default)]
    pub terminal_reason: Option<String>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    /// What `--permission-prompts none` denied. Empty when nothing was
    /// denied; v1 never answers these itself (docs/agent-host.md §4.1).
    #[serde(default)]
    pub permission_denials: Vec<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitEvent {
    pub rate_limit_info: Value,
}

/// One window from `rate_limit_info`, normalized from the two shapes seen on
/// the wire: per-window `unifiedWindows` entries, and (as a fallback) the
/// top-level fields when there's no `unifiedWindows` at all.
#[derive(Debug, Clone, PartialEq)]
pub struct RateLimitWindow {
    pub window: String,
    pub status: Option<String>,
    pub utilization: Option<f64>,
    pub resets_at_epoch: Option<i64>,
}

impl RateLimitEvent {
    /// One entry per `unifiedWindows` key (e.g. `five_hour`, `seven_day`).
    /// The top-level `status` is attached to whichever window matches
    /// `rateLimitType`, since that's the only window it's actually reported
    /// for. Real example (docs/spikes/01-stream-json-findings.md):
    /// `{"status":"allowed","resetsAt":…,"rateLimitType":"five_hour",
    /// "unifiedWindows":{"five_hour":{...},"seven_day":{...}}}`.
    pub fn windows(&self) -> Vec<RateLimitWindow> {
        let info = &self.rate_limit_info;
        let top_status = info.get("status").and_then(Value::as_str).map(String::from);
        let top_type = info
            .get("rateLimitType")
            .and_then(Value::as_str)
            .map(String::from);

        match info.get("unifiedWindows").and_then(Value::as_object) {
            Some(map) if !map.is_empty() => map
                .iter()
                .map(|(name, w)| RateLimitWindow {
                    window: name.clone(),
                    status: if Some(name.as_str()) == top_type.as_deref() {
                        top_status.clone()
                    } else {
                        None
                    },
                    utilization: w.get("utilization").and_then(Value::as_f64),
                    resets_at_epoch: w.get("resetsAt").and_then(Value::as_i64),
                })
                .collect(),
            _ => vec![RateLimitWindow {
                window: top_type.unwrap_or_default(),
                status: top_status,
                utilization: info.get("utilization").and_then(Value::as_f64),
                resets_at_epoch: info.get("resetsAt").and_then(Value::as_i64),
            }],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControlResponse {
    pub response: ControlResponseBody,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControlResponseBody {
    pub subtype: String,
    pub request_id: String,
    #[serde(default)]
    pub response: Option<Value>,
    #[serde(default)]
    pub error: Option<Value>,
}

impl Event {
    pub fn parse(t_ms: u64, raw: String) -> Self {
        let value: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => {
                return Self {
                    t_ms,
                    raw,
                    value: Value::Null,
                    kind: EventKind::NotJson,
                };
            }
        };
        let ty = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let subtype = value
            .get("subtype")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        fn typed<T: serde::de::DeserializeOwned>(
            v: &Value,
            ty: &str,
            f: impl FnOnce(T) -> EventKind,
        ) -> EventKind {
            match serde_json::from_value::<T>(v.clone()) {
                Ok(x) => f(x),
                Err(e) => EventKind::Unparsed {
                    r#type: ty.to_string(),
                    error: e.to_string(),
                },
            }
        }

        let kind = match (ty.as_str(), subtype.as_str()) {
            ("system", "init") => typed(&value, &ty, EventKind::SystemInit),
            ("system", _) => EventKind::System { subtype },
            ("assistant", _) => typed(&value, &ty, EventKind::Assistant),
            ("user", _) => typed(&value, &ty, EventKind::User),
            ("result", _) => typed(&value, &ty, EventKind::Result),
            ("rate_limit_event", _) => typed(&value, &ty, EventKind::RateLimit),
            ("control_response", _) => typed(&value, &ty, EventKind::ControlResponse),
            ("control_request", _) => EventKind::ControlRequest(value.clone()),
            _ => EventKind::Unknown { r#type: ty.clone() },
        };
        Self {
            t_ms,
            raw,
            value,
            kind,
        }
    }

    /// `type` or `type/subtype`, for logging and catalogues.
    pub fn label(&self) -> String {
        let ty = self
            .value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("?");
        match self.value.get("subtype").and_then(Value::as_str) {
            Some(st) => format!("{ty}/{st}"),
            None => ty.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_is_a_plain_string_content() {
        let ev = Event::parse(
            0,
            r#"{"type":"user","message":{"role":"user","content":"hi"},"session_id":"s"}"#
                .to_string(),
        );
        let EventKind::User(m) = &ev.kind else {
            panic!("expected User: {:?}", ev.kind)
        };
        assert!(m.is_replay());
        assert_eq!(m.replay_text(), Some("hi"));
    }

    #[test]
    fn tool_result_is_not_a_replay() {
        let ev = Event::parse(
            0,
            r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t","content":"x","is_error":false}]}}"#
                .to_string(),
        );
        let EventKind::User(m) = &ev.kind else {
            panic!("expected User: {:?}", ev.kind)
        };
        assert!(!m.is_replay());
        assert_eq!(m.replay_text(), None);
    }

    #[test]
    fn rate_limit_windows_from_unified_windows() {
        let ev = Event::parse(
            0,
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","resetsAt":1790533200,"rateLimitType":"five_hour","unifiedWindows":{"five_hour":{"utilization":0.05,"resetsAt":1790533200},"seven_day":{"utilization":0.13,"resetsAt":1790751600}}}}"#
                .to_string(),
        );
        let EventKind::RateLimit(r) = &ev.kind else {
            panic!("expected RateLimit: {:?}", ev.kind)
        };
        let windows = r.windows();
        assert_eq!(windows.len(), 2);
        let five = windows
            .iter()
            .find(|w| w.window == "five_hour")
            .expect("five_hour");
        assert_eq!(five.status.as_deref(), Some("allowed"));
        assert_eq!(five.utilization, Some(0.05));
        assert_eq!(five.resets_at_epoch, Some(1790533200));
        let seven = windows
            .iter()
            .find(|w| w.window == "seven_day")
            .expect("seven_day");
        assert_eq!(seven.status, None);
        assert_eq!(seven.utilization, Some(0.13));
    }

    #[test]
    fn rate_limit_windows_without_unified_windows() {
        let ev = Event::parse(
            0,
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","resetsAt":1,"rateLimitType":"five_hour","utilization":0.5}}"#
                .to_string(),
        );
        let EventKind::RateLimit(r) = &ev.kind else {
            panic!("expected RateLimit: {:?}", ev.kind)
        };
        let windows = r.windows();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].window, "five_hour");
        assert_eq!(windows[0].status.as_deref(), Some("allowed"));
        assert_eq!(windows[0].utilization, Some(0.5));
    }

    #[test]
    fn unknown_type_falls_back() {
        let ev = Event::parse(0, r#"{"type":"something_new","foo":1}"#.to_string());
        assert!(matches!(ev.kind, EventKind::Unknown { .. }));
    }

    #[test]
    fn not_json_falls_back() {
        let ev = Event::parse(0, "not json".to_string());
        assert!(matches!(ev.kind, EventKind::NotJson));
    }
}
