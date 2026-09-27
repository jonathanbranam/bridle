//! Incremental Server-Sent-Events parsing for `GET /v1/events/stream`. Pure
//! and independent of reqwest so it can be unit tested with hand-fed chunks.
//!
//! Framing (docs/design/agent-host/api.md): `id: <seq>` and `event: <kind>` lines
//! are optional and ignored — the `data:` line already carries the full
//! JSON [`Event`], seq included. A blank line ends an event. Lines starting
//! with `:` are comments/keepalives and are ignored. `data:` may repeat
//! (SSE joins repeats with `\n`); v1 only ever needs one.

use std::collections::VecDeque;

use bytes::Bytes;
use futures::{Stream, StreamExt};

use crate::client::ClientError;
use crate::types::Event;

/// Feed raw bytes in, get parsed events out, tolerant of chunk boundaries
/// landing anywhere (mid-line, mid-field, mid-event).
#[derive(Debug, Default)]
pub(crate) struct SseParser {
    buf: Vec<u8>,
    data_lines: Vec<String>,
}

impl SseParser {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn feed(&mut self, chunk: &[u8]) -> Vec<Result<Event, ClientError>> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        while let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
            let raw: Vec<u8> = self.buf.drain(..=pos).collect();
            let mut line = &raw[..raw.len() - 1]; // drop '\n'
            if line.last() == Some(&b'\r') {
                line = &line[..line.len() - 1];
            }
            self.handle_line(&String::from_utf8_lossy(line), &mut out);
        }
        out
    }

    fn handle_line(&mut self, line: &str, out: &mut Vec<Result<Event, ClientError>>) {
        if line.is_empty() {
            self.dispatch(out);
        } else if line.starts_with(':') {
            // comment / keepalive
        } else if let Some(rest) = line.strip_prefix("data:") {
            self.data_lines
                .push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
        }
        // `id:`, `event:` and any other field are parsed by the SSE framing
        // but not needed to build an `Event` (it's already in `data`).
    }

    fn dispatch(&mut self, out: &mut Vec<Result<Event, ClientError>>) {
        if self.data_lines.is_empty() {
            return;
        }
        let data = self.data_lines.join("\n");
        self.data_lines.clear();
        match serde_json::from_str::<Event>(&data) {
            Ok(ev) => out.push(Ok(ev)),
            Err(e) => out.push(Err(ClientError::Decode(format!("bad SSE event data: {e}")))),
        }
    }
}

/// Turn a byte stream (as returned by `reqwest::Response::bytes_stream`)
/// into a stream of parsed [`Event`]s.
pub(crate) fn parse_event_stream<S>(
    byte_stream: S,
) -> impl Stream<Item = Result<Event, ClientError>>
where
    S: Stream<Item = Result<Bytes, ClientError>> + Unpin,
{
    struct State<S> {
        stream: S,
        parser: SseParser,
        pending: VecDeque<Result<Event, ClientError>>,
        done: bool,
    }

    futures::stream::unfold(
        State {
            stream: byte_stream,
            parser: SseParser::new(),
            pending: VecDeque::new(),
            done: false,
        },
        |mut state| async move {
            loop {
                if let Some(item) = state.pending.pop_front() {
                    return Some((item, state));
                }
                if state.done {
                    return None;
                }
                match state.stream.next().await {
                    Some(Ok(bytes)) => {
                        let events = state.parser.feed(&bytes);
                        state.pending.extend(events);
                    }
                    Some(Err(e)) => {
                        state.done = true;
                        return Some((Err(e), state));
                    }
                    None => return None,
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev_json(seq: i64) -> String {
        format!(
            r#"{{"seq":{seq},"ts":"2026-09-27T00:00:00Z","kind":"agent.text","actor":"human","agent":null,"data":{{}}}}"#
        )
    }

    #[test]
    fn single_event_in_one_chunk() {
        let mut p = SseParser::new();
        let chunk = format!("id: 1\nevent: agent.text\ndata: {}\n\n", ev_json(1));
        let out = p.feed(chunk.as_bytes());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].as_ref().unwrap().seq, 1);
    }

    #[test]
    fn comment_and_keepalive_lines_are_ignored() {
        let mut p = SseParser::new();
        let chunk = format!(": keepalive\n\ndata: {}\n\n", ev_json(2));
        let out = p.feed(chunk.as_bytes());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].as_ref().unwrap().seq, 2);
    }

    #[test]
    fn multiple_data_lines_are_joined_with_newline() {
        // SSE joins repeated `data:` lines with `\n` before the payload is
        // used. JSON treats `\n` as insignificant whitespace between tokens,
        // so splitting right after a comma (a token boundary) keeps the
        // rejoined payload valid, exercising the join rule without
        // corrupting a string literal.
        let json = ev_json(3);
        let split_at = json.find(',').unwrap() + 1;
        let (a, b) = json.split_at(split_at);
        let mut p = SseParser::new();
        let chunk = format!("data: {a}\ndata: {b}\n\n");
        let out = p.feed(chunk.as_bytes());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].as_ref().unwrap().seq, 3);
    }

    #[test]
    fn split_across_feed_calls_mid_line() {
        let json = ev_json(4);
        let mut p = SseParser::new();
        assert!(p.feed(b"data: ").is_empty());
        let mid = json.len() / 2;
        assert!(p.feed(&json.as_bytes()[..mid]).is_empty());
        assert!(p.feed(&json.as_bytes()[mid..]).is_empty());
        let out = p.feed(b"\n\n");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].as_ref().unwrap().seq, 4);
    }

    #[test]
    fn two_events_back_to_back() {
        let mut p = SseParser::new();
        let chunk = format!("data: {}\n\ndata: {}\n\n", ev_json(5), ev_json(6));
        let out = p.feed(chunk.as_bytes());
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].as_ref().unwrap().seq, 5);
        assert_eq!(out[1].as_ref().unwrap().seq, 6);
    }

    #[test]
    fn bad_json_yields_a_decode_error_without_killing_the_parser() {
        let mut p = SseParser::new();
        let out = p.feed(b"data: not json\n\n");
        assert_eq!(out.len(), 1);
        assert!(out[0].is_err());
        // parser keeps working afterwards
        let out2 = p.feed(format!("data: {}\n\n", ev_json(7)).as_bytes());
        assert_eq!(out2.len(), 1);
        assert_eq!(out2[0].as_ref().unwrap().seq, 7);
    }

    #[tokio::test]
    async fn parse_event_stream_flattens_chunks_into_events() {
        let chunk1 = format!("data: {}\n\ndata: ", ev_json(1));
        let json2 = ev_json(2);
        let (a, b) = json2.split_at(json2.len() / 2);
        let chunk2 = a.to_string();
        let chunk3 = format!("{b}\n\n");
        let raw: Vec<Result<Bytes, ClientError>> = vec![
            Ok(Bytes::from(chunk1)),
            Ok(Bytes::from(chunk2)),
            Ok(Bytes::from(chunk3)),
        ];
        let byte_stream = futures::stream::iter(raw);
        let events: Vec<_> = parse_event_stream(byte_stream).collect().await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].as_ref().unwrap().seq, 1);
        assert_eq!(events[1].as_ref().unwrap().seq, 2);
    }
}
