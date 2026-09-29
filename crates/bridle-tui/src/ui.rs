//! Rendering. Reads [`App`] but never mutates it; all state changes go
//! through `App::on_message`/`App::on_key`.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, List, ListItem, Row, Table};

use ratatui::widgets::{Clear, Paragraph, Wrap};

use crate::app::{App, ConnectionStatus, Focus};
use crate::format::render_event_line;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Min(3),
        ])
        .split(frame.area());

    draw_status_line(frame, chunks[0], app);
    draw_agents(frame, chunks[1], app);
    draw_events(frame, chunks[2], app);
    draw_logs(frame, chunks[3], app);
    draw_inbox(frame, chunks[4], app);

    if let Some(msg) = &app.viewing {
        draw_message(frame, frame.area(), msg);
    }
    if let Some(compose) = &app.compose {
        draw_compose(frame, frame.area(), compose);
    }
}

fn draw_status_line(frame: &mut Frame, area: Rect, app: &App) {
    let status = match app.connection {
        ConnectionStatus::Connecting => "connecting…",
        ConnectionStatus::Connected => "connected",
    };
    let line = Line::from(format!(
        " bridle tui — {status} — Tab: switch view  j/k, ↑/↓: scroll  Enter: open  r: reply  q: quit"
    ));
    frame.render_widget(line, area);
}

/// The selected row is only marked in the focused pane.
fn highlight_style(focused: bool) -> Style {
    if focused {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    }
}

fn draw_agents(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Agents;
    let header = Row::new(vec![
        "ID", "NAME", "ROLE", "STATE", "MODEL", "COST", "TURNS", "CONTEXT",
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows = app.agents.iter().map(|a| {
        Row::new(vec![
            Cell::from(a.id.clone()),
            Cell::from(a.name.clone()),
            Cell::from(a.role.clone()),
            Cell::from(a.state.to_string()),
            Cell::from(a.model.clone()),
            Cell::from(format!(
                "${}",
                crate::format::format_cost_dollars(a.cost_usd_total)
            )),
            Cell::from(a.turns.to_string()),
            // Not a real zero-sized context: no turn has ended yet.
            Cell::from(crate::format::format_context_tokens(a.context_tokens)),
        ])
    });
    let widths = [
        Constraint::Length(10),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Length(9),
    ];
    let rows: Vec<Row> = rows.collect();
    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(highlight_style(focused))
        .block(border_block("Agents", focused));
    let mut state = app.agents_table_state;
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_events(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Events;
    let block = border_block("Events", focused);
    let inner_height = block.inner(area).height as usize;

    // `event_scroll` lines are scrolled up from the tail; the visible
    // window is the `inner_height` events ending `event_scroll` back from
    // the newest one.
    let end = app.events.len().saturating_sub(app.event_scroll);
    let start = end.saturating_sub(inner_height);
    let items: Vec<ListItem> = app.events[start..end]
        .iter()
        .map(|ev| ListItem::new(render_event_line(ev)))
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_logs(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Logs;
    let block = border_block("Logs", focused);
    let inner_height = block.inner(area).height as usize;

    let end = app.log_lines.len().saturating_sub(app.log_scroll);
    let start = end.saturating_sub(inner_height);
    let items: Vec<ListItem> = app.log_lines[start..end]
        .iter()
        .map(|line| ListItem::new(line.as_str()))
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_inbox(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Inbox;
    let header = Row::new(vec!["ID", "FROM", "KIND", "BODY"])
        .style(Style::default().add_modifier(Modifier::BOLD));
    let rows = app.messages.iter().map(|m| {
        Row::new(vec![
            Cell::from(m.id.clone()),
            Cell::from(m.from.clone()),
            Cell::from(format!("{:?}", m.kind).to_lowercase()),
            Cell::from(match &m.answered_by {
                Some(by) => format!(
                    "answered by {by}: {}",
                    m.answered_line.as_deref().unwrap_or_default()
                ),
                None => m.body.clone(),
            }),
        ])
    });
    let widths = [
        Constraint::Length(10),
        Constraint::Length(16),
        Constraint::Length(10),
        Constraint::Min(20),
    ];
    let rows: Vec<Row> = rows.collect();
    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(highlight_style(focused))
        .block(border_block("Inbox", focused));
    let mut state = app.inbox_table_state;
    frame.render_stateful_widget(table, area, &mut state);
}

/// The opened message in full, same layout as `bridle inbox show`.
fn draw_message(frame: &mut Frame, area: Rect, msg: &bridle_api::Message) {
    let popup = centered_rect(area, 80, 80);
    frame.render_widget(Clear, popup);

    let kind = format!("{:?}", msg.kind).to_lowercase();
    let time = msg.created_at.with_timezone(&chrono::Local);
    let mut lines = vec![
        Line::from(format!("From: {}", msg.from)),
        Line::from(format!("Kind: {kind}")),
        Line::from(format!("Time: {}", time.format("%Y-%m-%d %H:%M:%S %Z"))),
    ];
    if let Some(reply_to) = &msg.reply_to {
        lines.push(Line::from(format!("Reply-To: {reply_to}")));
    }
    if let Some(by) = &msg.answered_by {
        let reply = msg.answered_reply.as_deref().unwrap_or("?");
        lines.push(Line::from(format!("Answered by: {by} ({reply})")));
    }
    lines.push(Line::from(""));
    lines.extend(msg.body.lines().map(|l| Line::from(l.to_string())));

    let block = Block::default()
        .title(format!("{} (r: reply, Esc/Enter: close)", msg.id))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

/// A centered popup with the reply-in-progress body and a `|` cursor, drawn
/// over everything else while `app.compose` is `Some`.
fn draw_compose(frame: &mut Frame, area: Rect, compose: &crate::app::Compose) {
    let popup = centered_rect(area, 60, 30);
    frame.render_widget(Clear, popup);

    let byte_index = compose
        .body
        .char_indices()
        .nth(compose.cursor)
        .map(|(i, _)| i)
        .unwrap_or(compose.body.len());
    let mut text = compose.body.clone();
    text.insert(byte_index, '\u{2588}'); // block cursor

    let block = Block::default()
        .title(format!(
            "Reply to {} (Enter: send, Esc: cancel)",
            compose.to
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn border_block(title: &'static str, focused: bool) -> Block<'static> {
    let style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    use super::*;
    use crate::app::Message;

    fn agent(id: &str) -> bridle_api::Agent {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": id, "role": "worker", "state": "idle",
            "model": "sonnet", "session_id": "s", "pid": null, "cwd": "/",
            "worktree": null, "branch": null,
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z",
            "last_event_at": null, "turns": 0, "turn_started_at": null,
            "cost_usd_total": 0.0, "exit": null, "created_by": "human",
            "held_messages": 0, "unacked_messages": 0, "components": [],
            "context_tokens": null,
        }))
        .expect("agent json")
    }

    fn inbox_message(id: &str) -> bridle_api::Message {
        serde_json::from_value(serde_json::json!({
            "id": id, "from": "w", "to": "human", "kind": "note", "body": "hi",
            "reply_to": null, "when": "now", "state": "delivered",
            "created_at": "2026-01-01T00:00:00Z", "written_at": null,
            "delivered_at": null, "read_at": null,
        }))
        .expect("message json")
    }

    fn render(app: &App) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(140, 40)).expect("terminal");
        terminal.draw(|f| draw(f, app)).expect("draw");
        terminal.backend().buffer().clone()
    }

    /// Whether any cell on the row containing `needle` is reversed, and the
    /// same for the row containing `other`.
    fn reversed(buf: &Buffer, needle: &str) -> bool {
        let w = buf.area.width;
        (0..buf.area.height).any(|y| {
            let line: String = (0..w).map(|x| buf[(x, y)].symbol()).collect();
            line.contains(needle)
                && (0..w).any(|x| buf[(x, y)].modifier.contains(Modifier::REVERSED))
        })
    }

    #[test]
    fn selected_agent_row_is_highlighted() {
        let mut app = App::new();
        app.on_message(Message::AgentsLoaded(vec![agent("sel-a"), agent("oth-b")]));
        let buf = render(&app);
        assert!(reversed(&buf, "sel-a"));
        assert!(!reversed(&buf, "oth-b"));
    }

    #[test]
    fn selected_inbox_row_is_highlighted_when_focused() {
        let mut app = App::new();
        app.on_message(Message::MessagesLoaded(vec![
            inbox_message("m-sel"),
            inbox_message("m-oth"),
        ]));
        app.focus = Focus::Inbox;
        let buf = render(&app);
        assert!(reversed(&buf, "m-sel"));
        assert!(!reversed(&buf, "m-oth"));
    }
}
