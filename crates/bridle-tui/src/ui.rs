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
        " bridle tui — {status} — Tab: switch view  j/k, ↑/↓: scroll  r: reply  q: quit"
    ));
    frame.render_widget(line, area);
}

fn draw_agents(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Agents;
    let header = Row::new(vec![
        "ID", "NAME", "ROLE", "STATE", "MODEL", "COST", "TURNS",
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows = app.agents.iter().map(|a| {
        Row::new(vec![
            Cell::from(a.id.clone()),
            Cell::from(a.name.clone()),
            Cell::from(a.role.clone()),
            Cell::from(a.state.to_string()),
            Cell::from(a.model.clone()),
            Cell::from(format!("${:.4}", a.cost_usd_total)),
            Cell::from(a.turns.to_string()),
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
    ];
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows: Vec<Row> = rows
        .enumerate()
        .map(|(i, row)| {
            if focused && i == app.selected_agent {
                row.style(selected_style)
            } else {
                row
            }
        })
        .collect();
    let table = Table::new(rows, widths)
        .header(header)
        .block(border_block("Agents", focused));
    frame.render_widget(table, area);
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
            Cell::from(m.body.clone()),
        ])
    });
    let widths = [
        Constraint::Length(10),
        Constraint::Length(16),
        Constraint::Length(10),
        Constraint::Min(20),
    ];
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows: Vec<Row> = rows
        .enumerate()
        .map(|(i, row)| {
            if focused && i == app.selected_message {
                row.style(selected_style)
            } else {
                row
            }
        })
        .collect();
    let table = Table::new(rows, widths)
        .header(header)
        .block(border_block("Inbox", focused));
    frame.render_widget(table, area);
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
