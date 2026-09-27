#![allow(dead_code)]
//! Tolerant event model. Every stdout line becomes an `Event` that keeps its
//! raw text and parsed `Value`; `kind` is a typed view for the variants the
//! host needs to act on, with `Unknown` / `Unparsed` fallbacks so a new or
//! reshaped event never breaks the reader.

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
    System { subtype: String },
    Assistant(Message),
    User(Message),
    Result(ResultEvent),
    RateLimit(RateLimitEvent),
    ControlResponse(ControlResponse),
    /// claude asking the host something (permission prompts, hook callbacks).
    ControlRequest(Value),
    /// Valid JSON with a `type` we don't model.
    Unknown { r#type: String },
    /// Known `type`, but the typed view failed; the error is kept for the findings.
    Unparsed { r#type: String, error: String },
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
    /// Kept raw: shape (strings vs objects) is one of the things being checked.
    #[serde(default)]
    pub capabilities: Option<Value>,
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
    /// A string for echoed user text, an array of blocks otherwise.
    #[serde(default)]
    pub content: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Thinking {},
    ToolUse { id: String, name: String, input: Value },
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
            Err(_) => return Self { t_ms, raw, value: Value::Null, kind: EventKind::NotJson },
        };
        let ty = value.get("type").and_then(Value::as_str).unwrap_or("").to_string();
        let subtype = value.get("subtype").and_then(Value::as_str).unwrap_or("").to_string();

        fn typed<T: serde::de::DeserializeOwned>(
            v: &Value,
            ty: &str,
            f: impl FnOnce(T) -> EventKind,
        ) -> EventKind {
            match serde_json::from_value::<T>(v.clone()) {
                Ok(x) => f(x),
                Err(e) => EventKind::Unparsed { r#type: ty.to_string(), error: e.to_string() },
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
        Self { t_ms, raw, value, kind }
    }

    /// `type` or `type/subtype`, for catalogue printing.
    pub fn label(&self) -> String {
        let ty = self.value.get("type").and_then(Value::as_str).unwrap_or("?");
        match self.value.get("subtype").and_then(Value::as_str) {
            Some(st) => format!("{ty}/{st}"),
            None => ty.to_string(),
        }
    }
}
