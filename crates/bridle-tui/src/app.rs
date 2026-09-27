//! App state and update logic, kept free of any terminal/ratatui
//! dependency so it can be exercised directly in tests: feed it a
//! [`Message`] (from `events_stream`, or the initial agents list) or a
//! [`Key`] press, and assert on the resulting [`App`].

use bridle_api::{Agent, AgentState, Event, Message as ApiMessage, TranscriptLine, event_kind};

use crate::format::render_transcript_line;

/// Which of the four views has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Agents,
    Events,
    Logs,
    Inbox,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Agents => Focus::Events,
            Focus::Events => Focus::Logs,
            Focus::Logs => Focus::Inbox,
            Focus::Inbox => Focus::Agents,
        }
    }
}

/// Line-editing state for composing a reply: `body` plus a cursor position
/// (a char index, not a byte offset, so it moves one visible character at a
/// time regardless of UTF-8 width) and the message being replied to.
#[derive(Debug, Clone, Default)]
pub struct Compose {
    pub to: String,
    pub reply_to: String,
    pub body: String,
    pub cursor: usize,
}

impl Compose {
    fn byte_index(&self) -> usize {
        self.body
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.body.len())
    }

    fn char_len(&self) -> usize {
        self.body.chars().count()
    }

    fn insert(&mut self, c: char) {
        let idx = self.byte_index();
        self.body.insert(idx, c);
        self.cursor += 1;
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let idx = self.byte_index();
        let len = self.body[idx..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0);
        self.body.drain(idx..idx + len);
    }

    fn cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn cursor_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.char_len());
    }
}

/// A reply App has finished composing, for `run.rs` to actually send: `App`
/// has no client of its own (see the module doc), so submitting a compose
/// just hands this off the same way `logs_agent` hands off which agent to
/// poll transcript for.
#[derive(Debug, Clone)]
pub struct PendingSend {
    pub to: String,
    pub reply_to: String,
    pub body: String,
}

/// Whether we've heard from the daemon yet. `events_stream` reconnects
/// transparently on its own (bridle-api::Client::events_stream) and never
/// surfaces a transient disconnect to its caller, so this only tracks
/// whether the initial contact has happened, not moment-to-moment
/// liveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
}

/// A key press, translated from whatever terminal backend is in use
/// (crossterm, in `run.rs`) so this module has no terminal dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Tab,
    Enter,
    Backspace,
    Esc,
}

/// Something arriving from the daemon: the initial agents list, or one
/// event off `events_stream`.
#[derive(Debug, Clone)]
pub enum Message {
    AgentsLoaded(Vec<Agent>),
    Event(Event),
    /// One poll's worth of transcript lines for `agent` (possibly empty).
    /// Dropped by `App` if `agent` isn't the currently selected agent, so a
    /// response to a since-superseded poll can't clobber the new tail.
    TranscriptLines {
        agent: String,
        lines: Vec<TranscriptLine>,
    },
    /// One poll's worth of the unread inbox (`to: "me"`, unread only --
    /// same query as `bridle inbox`).
    MessagesLoaded(Vec<ApiMessage>),
}

/// Everything on screen. Rendering (`ui.rs`) only ever reads this; only
/// [`App::on_message`] and [`App::on_key`] write to it.
#[derive(Debug, Default)]
pub struct App {
    pub agents: Vec<Agent>,
    pub events: Vec<Event>,
    pub focus: Focus,
    /// Index into `agents` of the selected row.
    pub selected_agent: usize,
    /// Lines scrolled up from the tail (0 = pinned to the newest event).
    pub event_scroll: usize,
    /// Id of the agent `log_lines` holds the transcript for, kept in step
    /// with `selected_agent` by `sync_logs_target` regardless of `focus` --
    /// so the log poll (driven off this field by `run.rs`) is always
    /// following the selected agent, and the logs view never has to
    /// backfill on focus change.
    pub logs_agent: Option<String>,
    pub log_lines: Vec<String>,
    /// Lines scrolled up from the tail (0 = pinned to the newest line).
    pub log_scroll: usize,
    /// Unread messages addressed to `me`, same query as `bridle inbox`.
    pub messages: Vec<ApiMessage>,
    /// Index into `messages` of the selected row.
    pub selected_message: usize,
    /// Set while composing a reply to `messages[selected_message]`; `None`
    /// means the inbox view is just a list.
    pub compose: Option<Compose>,
    /// A finished compose waiting for `run.rs` to send and mark the
    /// original read; taken (cleared) once `run.rs` has picked it up.
    pub pending_send: Option<PendingSend>,
    pub connection: ConnectionStatus,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_message(&mut self, msg: Message) {
        self.connection = ConnectionStatus::Connected;
        match msg {
            Message::AgentsLoaded(agents) => {
                self.agents = agents;
                self.clamp_selected_agent();
            }
            Message::Event(ev) => {
                self.apply_event_to_agents(&ev);
                self.events.push(ev);
            }
            Message::TranscriptLines { agent, lines } => {
                if self.logs_agent.as_deref() == Some(agent.as_str()) {
                    for line in &lines {
                        self.log_lines.extend(render_transcript_line(line));
                    }
                }
            }
            Message::MessagesLoaded(messages) => {
                self.messages = messages;
                self.clamp_selected_message();
            }
        }
        self.sync_logs_target();
    }

    pub fn on_key(&mut self, key: Key) {
        if self.compose.is_some() {
            self.on_compose_key(key);
            return;
        }
        match key {
            Key::Char('q') | Key::Esc => self.should_quit = true,
            Key::Tab => self.focus = self.focus.next(),
            Key::Char('r') if self.focus == Focus::Inbox => self.start_reply(),
            Key::Char('k') | Key::Up => self.scroll_up(),
            Key::Char('j') | Key::Down => self.scroll_down(),
            Key::Char(_) | Key::Left | Key::Right | Key::Enter | Key::Backspace => {}
        }
        self.sync_logs_target();
    }

    fn on_compose_key(&mut self, key: Key) {
        match key {
            Key::Esc => self.compose = None,
            Key::Enter => self.submit_compose(),
            Key::Char(c) => {
                if let Some(compose) = &mut self.compose {
                    compose.insert(c);
                }
            }
            Key::Backspace => {
                if let Some(compose) = &mut self.compose {
                    compose.backspace();
                }
            }
            Key::Left => {
                if let Some(compose) = &mut self.compose {
                    compose.cursor_left();
                }
            }
            Key::Right => {
                if let Some(compose) = &mut self.compose {
                    compose.cursor_right();
                }
            }
            Key::Up | Key::Down | Key::Tab => {}
        }
    }

    /// Start composing a reply to the selected inbox message, addressed
    /// back to its sender.
    fn start_reply(&mut self) {
        let Some(msg) = self.messages.get(self.selected_message) else {
            return;
        };
        self.compose = Some(Compose {
            to: msg.from.clone(),
            reply_to: msg.id.clone(),
            body: String::new(),
            cursor: 0,
        });
    }

    /// Hand the finished compose off to `run.rs` as a [`PendingSend`] and
    /// close the compose view. An empty body is dropped rather than sent.
    fn submit_compose(&mut self) {
        let Some(compose) = &self.compose else {
            return;
        };
        if compose.body.is_empty() {
            return;
        }
        let compose = self.compose.take().expect("checked above");
        self.pending_send = Some(PendingSend {
            to: compose.to,
            reply_to: compose.reply_to,
            body: compose.body,
        });
    }

    fn clamp_selected_message(&mut self) {
        if self.messages.is_empty() {
            self.selected_message = 0;
        } else {
            self.selected_message = self.selected_message.min(self.messages.len() - 1);
        }
    }

    /// The id of the currently selected agent, or `None` if the list is
    /// empty.
    pub fn selected_agent_id(&self) -> Option<&str> {
        self.agents.get(self.selected_agent).map(|a| a.id.as_str())
    }

    /// Reset the logs view whenever the selected agent changes (a
    /// navigation key, a reload of the agents list, or the selected agent
    /// being removed), so `logs_agent` -- and therefore what `run.rs`
    /// polls -- always tracks the current selection.
    fn sync_logs_target(&mut self) {
        let current = self.selected_agent_id().map(str::to_string);
        if current != self.logs_agent {
            self.logs_agent = current;
            self.log_lines.clear();
            self.log_scroll = 0;
        }
    }

    fn scroll_up(&mut self) {
        match self.focus {
            Focus::Agents => self.selected_agent = self.selected_agent.saturating_sub(1),
            Focus::Events => {
                let max = self.events.len().saturating_sub(1);
                self.event_scroll = (self.event_scroll + 1).min(max);
            }
            Focus::Logs => {
                let max = self.log_lines.len().saturating_sub(1);
                self.log_scroll = (self.log_scroll + 1).min(max);
            }
            Focus::Inbox => self.selected_message = self.selected_message.saturating_sub(1),
        }
    }

    fn scroll_down(&mut self) {
        match self.focus {
            Focus::Agents => {
                if !self.agents.is_empty() {
                    self.selected_agent = (self.selected_agent + 1).min(self.agents.len() - 1);
                }
            }
            Focus::Events => self.event_scroll = self.event_scroll.saturating_sub(1),
            Focus::Logs => self.log_scroll = self.log_scroll.saturating_sub(1),
            Focus::Inbox => {
                if !self.messages.is_empty() {
                    self.selected_message =
                        (self.selected_message + 1).min(self.messages.len() - 1);
                }
            }
        }
    }

    fn clamp_selected_agent(&mut self) {
        if self.agents.is_empty() {
            self.selected_agent = 0;
        } else {
            self.selected_agent = self.selected_agent.min(self.agents.len() - 1);
        }
    }

    /// Keep the visible agent list in step with the live event feed for
    /// the state changes an event actually carries: `agent.state`'s
    /// `to` field, and `agent.removed` dropping the row. Anything else
    /// (a brand-new agent from `agent.spawned`, for instance) only shows
    /// up on the next full `agents` list.
    fn apply_event_to_agents(&mut self, ev: &Event) {
        match ev.kind.as_str() {
            event_kind::AGENT_STATE => {
                let Some(id) = &ev.agent else { return };
                let Some(to) = ev.data.get("to").and_then(|v| v.as_str()) else {
                    return;
                };
                let Ok(state) =
                    serde_json::from_value::<AgentState>(serde_json::Value::String(to.to_string()))
                else {
                    return;
                };
                if let Some(agent) = self.agents.iter_mut().find(|a| &a.id == id) {
                    agent.state = state;
                    agent.updated_at = ev.ts;
                }
            }
            event_kind::AGENT_REMOVED => {
                let Some(id) = &ev.agent else { return };
                self.agents.retain(|a| &a.id != id);
                self.clamp_selected_agent();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bridle_api::AgentState;
    use chrono::Utc;

    use super::*;

    fn agent(id: &str, state: AgentState) -> Agent {
        Agent {
            id: id.to_string(),
            name: id.to_string(),
            role: "worker".to_string(),
            state,
            model: "sonnet".to_string(),
            session_id: "s-1".to_string(),
            pid: None,
            cwd: "/ws/wt/w1".to_string(),
            worktree: None,
            branch: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_event_at: None,
            turns: 0,
            turn_started_at: None,
            cost_usd_total: 0.0,
            exit: None,
            created_by: "human".to_string(),
            held_messages: 0,
            unacked_messages: 0,
        }
    }

    fn event(seq: i64, kind: &str, agent: Option<&str>, data: serde_json::Value) -> Event {
        Event {
            seq,
            ts: Utc::now(),
            kind: kind.to_string(),
            actor: "human".to_string(),
            agent: agent.map(str::to_string),
            data,
        }
    }

    #[test]
    fn starts_connecting_with_nothing_loaded() {
        let app = App::new();
        assert_eq!(app.connection, ConnectionStatus::Connecting);
        assert!(app.agents.is_empty());
        assert!(app.events.is_empty());
        assert_eq!(app.focus, Focus::Agents);
    }

    #[test]
    fn agents_loaded_seeds_the_list_and_marks_connected() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![
            agent("a-1", AgentState::Idle),
            agent("a-2", AgentState::Working),
        ]));
        assert_eq!(app.connection, ConnectionStatus::Connected);
        assert_eq!(app.agents.len(), 2);
    }

    #[test]
    fn event_is_appended_to_the_tail() {
        let mut app = App::new();
        app.on_message(Message::Event(event(
            1,
            "agent.text",
            None,
            serde_json::json!({}),
        )));
        assert_eq!(app.events.len(), 1);
        assert_eq!(app.events[0].seq, 1);
    }

    #[test]
    fn agent_state_event_updates_the_matching_agent() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![agent("a-1", AgentState::Idle)]));
        app.on_message(Message::Event(event(
            2,
            event_kind::AGENT_STATE,
            Some("a-1"),
            serde_json::json!({"from": "idle", "to": "working"}),
        )));
        assert_eq!(app.agents[0].state, AgentState::Working);
    }

    #[test]
    fn agent_state_event_for_unknown_agent_is_ignored() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![agent("a-1", AgentState::Idle)]));
        app.on_message(Message::Event(event(
            2,
            event_kind::AGENT_STATE,
            Some("a-nope"),
            serde_json::json!({"from": "idle", "to": "working"}),
        )));
        assert_eq!(app.agents[0].state, AgentState::Idle);
    }

    #[test]
    fn agent_removed_event_drops_the_row() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![
            agent("a-1", AgentState::Idle),
            agent("a-2", AgentState::Idle),
        ]));
        app.on_message(Message::Event(event(
            2,
            event_kind::AGENT_REMOVED,
            Some("a-1"),
            serde_json::json!({}),
        )));
        assert_eq!(app.agents.len(), 1);
        assert_eq!(app.agents[0].id, "a-2");
    }

    #[test]
    fn tab_cycles_through_the_four_views() {
        let mut app = App::new();
        assert_eq!(app.focus, Focus::Agents);
        app.on_key(Key::Tab);
        assert_eq!(app.focus, Focus::Events);
        app.on_key(Key::Tab);
        assert_eq!(app.focus, Focus::Logs);
        app.on_key(Key::Tab);
        assert_eq!(app.focus, Focus::Inbox);
        app.on_key(Key::Tab);
        assert_eq!(app.focus, Focus::Agents);
    }

    #[test]
    fn q_and_esc_request_quit() {
        let mut app = App::new();
        app.on_key(Key::Char('q'));
        assert!(app.should_quit);

        let mut app = App::new();
        app.on_key(Key::Esc);
        assert!(app.should_quit);
    }

    #[test]
    fn agents_navigation_is_clamped_to_the_list() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![
            agent("a-1", AgentState::Idle),
            agent("a-2", AgentState::Idle),
        ]));
        assert_eq!(app.selected_agent, 0);
        app.on_key(Key::Up); // already at top
        assert_eq!(app.selected_agent, 0);
        app.on_key(Key::Down);
        assert_eq!(app.selected_agent, 1);
        app.on_key(Key::Down); // already at bottom
        assert_eq!(app.selected_agent, 1);
        app.on_key(Key::Up);
        assert_eq!(app.selected_agent, 0);
    }

    #[test]
    fn removing_the_selected_agent_reclamps_the_selection() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![
            agent("a-1", AgentState::Idle),
            agent("a-2", AgentState::Idle),
        ]));
        app.on_key(Key::Down);
        assert_eq!(app.selected_agent, 1);
        app.on_message(Message::Event(event(
            2,
            event_kind::AGENT_REMOVED,
            Some("a-2"),
            serde_json::json!({}),
        )));
        assert_eq!(app.selected_agent, 0);
    }

    #[test]
    fn events_scroll_is_bounded_by_the_focused_view() {
        let mut app = App::new();
        app.on_message(Message::Event(event(
            1,
            "agent.text",
            None,
            serde_json::json!({}),
        )));
        app.on_message(Message::Event(event(
            2,
            "agent.text",
            None,
            serde_json::json!({}),
        )));
        // Focus starts on Agents: j/k move the agent selection, not the scroll.
        app.on_key(Key::Up);
        assert_eq!(app.event_scroll, 0);

        app.on_key(Key::Tab); // now focused on Events
        app.on_key(Key::Up);
        assert_eq!(app.event_scroll, 1);
        app.on_key(Key::Up); // already at the oldest of the two events
        assert_eq!(app.event_scroll, 1);
        app.on_key(Key::Down);
        assert_eq!(app.event_scroll, 0);
        app.on_key(Key::Down); // already pinned to the tail
        assert_eq!(app.event_scroll, 0);
    }

    fn transcript_line(n: u64, line: &str) -> TranscriptLine {
        TranscriptLine {
            n,
            t_ms: 0,
            dir: "out".to_string(),
            line: line.to_string(),
        }
    }

    #[test]
    fn selecting_an_agent_sets_it_as_the_logs_target() {
        let mut app = App::new();
        assert_eq!(app.logs_agent, None);
        app.on_message(Message::AgentsLoaded(vec![
            agent("a-1", AgentState::Idle),
            agent("a-2", AgentState::Idle),
        ]));
        assert_eq!(app.logs_agent, Some("a-1".to_string()));
    }

    #[test]
    fn transcript_lines_for_the_selected_agent_are_rendered_into_log_lines() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![agent("a-1", AgentState::Idle)]));
        app.on_message(Message::TranscriptLines {
            agent: "a-1".to_string(),
            lines: vec![transcript_line(
                1,
                r#"{"type":"result","subtype":"success","total_cost_usd":0.0}"#,
            )],
        });
        assert_eq!(app.log_lines, vec!["\u{2713} turn done (success, $0.0000)"]);
    }

    #[test]
    fn transcript_lines_for_a_stale_agent_are_dropped() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![agent("a-1", AgentState::Idle)]));
        app.on_message(Message::TranscriptLines {
            agent: "a-nope".to_string(),
            lines: vec![transcript_line(
                1,
                r#"{"type":"result","subtype":"success","total_cost_usd":0.0}"#,
            )],
        });
        assert!(app.log_lines.is_empty());
    }

    #[test]
    fn changing_the_selected_agent_reloads_the_logs_view() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![
            agent("a-1", AgentState::Idle),
            agent("a-2", AgentState::Idle),
        ]));
        app.on_message(Message::TranscriptLines {
            agent: "a-1".to_string(),
            lines: vec![transcript_line(
                1,
                r#"{"type":"result","subtype":"success","total_cost_usd":0.0}"#,
            )],
        });
        assert_eq!(app.log_lines.len(), 1);

        app.on_key(Key::Down); // select a-2
        assert_eq!(app.logs_agent, Some("a-2".to_string()));
        assert!(app.log_lines.is_empty());
    }

    #[test]
    fn logs_scroll_is_bounded_by_the_focused_view() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![agent("a-1", AgentState::Idle)]));
        app.on_message(Message::TranscriptLines {
            agent: "a-1".to_string(),
            lines: vec![
                transcript_line(
                    1,
                    r#"{"type":"assistant","message":{"content":[{"type":"text","text":"one"}]}}"#,
                ),
                transcript_line(
                    2,
                    r#"{"type":"assistant","message":{"content":[{"type":"text","text":"two"}]}}"#,
                ),
            ],
        });
        assert_eq!(app.log_lines.len(), 2);

        app.on_key(Key::Tab); // Events
        app.on_key(Key::Tab); // Logs
        assert_eq!(app.focus, Focus::Logs);
        app.on_key(Key::Up);
        assert_eq!(app.log_scroll, 1);
        app.on_key(Key::Up); // already at the oldest line
        assert_eq!(app.log_scroll, 1);
        app.on_key(Key::Down);
        assert_eq!(app.log_scroll, 0);
        app.on_key(Key::Down); // already pinned to the tail
        assert_eq!(app.log_scroll, 0);
    }

    fn inbox_message(id: &str, from: &str, body: &str) -> ApiMessage {
        ApiMessage {
            id: id.to_string(),
            from: from.to_string(),
            to: "human".to_string(),
            kind: bridle_api::MessageKind::Note,
            body: body.to_string(),
            reply_to: None,
            when: bridle_api::When::Now,
            state: bridle_api::MessageState::Delivered,
            created_at: Utc::now(),
            written_at: None,
            delivered_at: None,
            read_at: None,
        }
    }

    #[test]
    fn messages_loaded_seeds_the_inbox() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![
            inbox_message("m-1", "w1", "hi"),
            inbox_message("m-2", "w2", "hey"),
        ]));
        assert_eq!(app.messages.len(), 2);
        assert_eq!(app.selected_message, 0);
    }

    #[test]
    fn inbox_navigation_is_clamped_to_the_list() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![
            inbox_message("m-1", "w1", "hi"),
            inbox_message("m-2", "w2", "hey"),
        ]));
        app.on_key(Key::Tab); // Events
        app.on_key(Key::Tab); // Logs
        app.on_key(Key::Tab); // Inbox
        assert_eq!(app.focus, Focus::Inbox);
        app.on_key(Key::Up); // already at top
        assert_eq!(app.selected_message, 0);
        app.on_key(Key::Down);
        assert_eq!(app.selected_message, 1);
        app.on_key(Key::Down); // already at bottom
        assert_eq!(app.selected_message, 1);
    }

    #[test]
    fn reloading_the_inbox_reclamps_the_selection() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![
            inbox_message("m-1", "w1", "hi"),
            inbox_message("m-2", "w2", "hey"),
        ]));
        app.on_key(Key::Tab); // Events
        app.on_key(Key::Tab); // Logs
        app.on_key(Key::Tab); // Inbox
        app.on_key(Key::Down);
        assert_eq!(app.selected_message, 1);
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-2", "w2", "hey",
        )]));
        assert_eq!(app.selected_message, 0);
    }

    fn focus_inbox(app: &mut App) {
        app.on_key(Key::Tab); // Events
        app.on_key(Key::Tab); // Logs
        app.on_key(Key::Tab); // Inbox
    }

    #[test]
    fn r_on_the_inbox_starts_composing_a_reply_to_the_sender() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        let compose = app.compose.as_ref().expect("compose started");
        assert_eq!(compose.to, "w1");
        assert_eq!(compose.reply_to, "m-1");
        assert_eq!(compose.body, "");
        assert_eq!(compose.cursor, 0);
    }

    #[test]
    fn r_elsewhere_is_not_treated_as_reply() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        app.on_key(Key::Char('r')); // still focused on Agents
        assert!(app.compose.is_none());
    }

    #[test]
    fn compose_insert_and_backspace_edit_the_body_at_the_cursor() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        app.on_key(Key::Char('h'));
        app.on_key(Key::Char('i'));
        assert_eq!(app.compose.as_ref().unwrap().body, "hi");
        assert_eq!(app.compose.as_ref().unwrap().cursor, 2);

        app.on_key(Key::Backspace);
        assert_eq!(app.compose.as_ref().unwrap().body, "h");
        assert_eq!(app.compose.as_ref().unwrap().cursor, 1);

        app.on_key(Key::Backspace);
        app.on_key(Key::Backspace); // already empty
        assert_eq!(app.compose.as_ref().unwrap().body, "");
        assert_eq!(app.compose.as_ref().unwrap().cursor, 0);
    }

    #[test]
    fn compose_cursor_movement_inserts_at_the_right_spot() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        app.on_key(Key::Char('a'));
        app.on_key(Key::Char('c'));
        app.on_key(Key::Left);
        app.on_key(Key::Char('b'));
        assert_eq!(app.compose.as_ref().unwrap().body, "abc");
        assert_eq!(app.compose.as_ref().unwrap().cursor, 2);

        app.on_key(Key::Right);
        assert_eq!(app.compose.as_ref().unwrap().cursor, 3);
        app.on_key(Key::Right); // already at the end
        assert_eq!(app.compose.as_ref().unwrap().cursor, 3);
    }

    #[test]
    fn esc_cancels_the_compose_without_sending() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        app.on_key(Key::Char('x'));
        app.on_key(Key::Esc);
        assert!(app.compose.is_none());
        assert!(app.pending_send.is_none());
        assert!(!app.should_quit);
    }

    #[test]
    fn enter_submits_the_compose_as_a_pending_send() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        app.on_key(Key::Char('o'));
        app.on_key(Key::Char('k'));
        app.on_key(Key::Enter);

        assert!(app.compose.is_none());
        let pending = app.pending_send.as_ref().expect("pending send");
        assert_eq!(pending.to, "w1");
        assert_eq!(pending.reply_to, "m-1");
        assert_eq!(pending.body, "ok");
    }

    #[test]
    fn enter_on_an_empty_body_does_not_submit() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        app.on_key(Key::Enter);
        assert!(app.compose.is_some());
        assert!(app.pending_send.is_none());
    }

    #[test]
    fn q_while_composing_is_typed_not_treated_as_quit() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![inbox_message(
            "m-1", "w1", "hi",
        )]));
        focus_inbox(&mut app);
        app.on_key(Key::Char('r'));
        app.on_key(Key::Char('q'));
        assert_eq!(app.compose.as_ref().unwrap().body, "q");
        assert!(!app.should_quit);
    }
}
