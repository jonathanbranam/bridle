//! App state and update logic, kept free of any terminal/ratatui
//! dependency so it can be exercised directly in tests: feed it a
//! [`Message`] (from `events_stream`, or the initial agents list) or a
//! [`Key`] press, and assert on the resulting [`App`].

use bridle_api::{Agent, AgentState, Event, event_kind};

/// Which of the two views has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Agents,
    Events,
}

impl Focus {
    pub fn toggle(self) -> Self {
        match self {
            Focus::Agents => Focus::Events,
            Focus::Events => Focus::Agents,
        }
    }
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
    Tab,
    Esc,
}

/// Something arriving from the daemon: the initial agents list, or one
/// event off `events_stream`.
#[derive(Debug, Clone)]
pub enum Message {
    AgentsLoaded(Vec<Agent>),
    Event(Event),
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
        }
    }

    pub fn on_key(&mut self, key: Key) {
        match key {
            Key::Char('q') | Key::Esc => self.should_quit = true,
            Key::Tab => self.focus = self.focus.toggle(),
            Key::Char('k') | Key::Up => self.scroll_up(),
            Key::Char('j') | Key::Down => self.scroll_down(),
            Key::Char(_) => {}
        }
    }

    fn scroll_up(&mut self) {
        match self.focus {
            Focus::Agents => self.selected_agent = self.selected_agent.saturating_sub(1),
            Focus::Events => {
                let max = self.events.len().saturating_sub(1);
                self.event_scroll = (self.event_scroll + 1).min(max);
            }
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
    fn tab_toggles_focus() {
        let mut app = App::new();
        assert_eq!(app.focus, Focus::Agents);
        app.on_key(Key::Tab);
        assert_eq!(app.focus, Focus::Events);
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
}
