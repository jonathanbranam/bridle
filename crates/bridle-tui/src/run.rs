//! Terminal setup/teardown and the event loop: wires a [`bridle_api::Client`]
//! to [`App`], the only place in this crate that touches a real terminal.

use std::io::{self, Stdout};

use bridle_api::{Client, MessageQuery, SendRequest};
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::{App, Key, Message};
use crate::ui;

type Term = Terminal<CrosstermBackend<Stdout>>;

/// Run the TUI to completion: seed the agents list, follow `events_stream`,
/// and handle key presses until the user quits. Restores the terminal on
/// exit, including on panic.
pub async fn run(client: Client) -> anyhow::Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let result = run_app(&mut terminal, client).await;
    restore_terminal()?;
    result
}

fn init_terminal() -> anyhow::Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// So a panic mid-render doesn't leave the user's terminal in raw mode /
/// the alternate screen (ratatui's usual panic-hook-plus-restore idiom).
fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original(info);
    }));
}

async fn run_app(terminal: &mut Term, client: Client) -> anyhow::Result<()> {
    let mut app = App::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    {
        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Ok(agents) = client.list_agents().await {
                let _ = tx.send(Message::AgentsLoaded(agents));
            }
        });
    }
    {
        // events_stream reconnects on its own (bridle_api::Client::events_stream);
        // this task just forwards whatever it yields until the receiver goes away.
        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut events = std::pin::pin!(client.events_stream(None));
            while let Some(Ok(ev)) = events.next().await {
                if tx.send(Message::Event(ev)).is_err() {
                    break;
                }
            }
        });
    }
    {
        // Same polling model as spawn_transcript_poll: there's no SSE stream
        // for messages, so poll the unread inbox (`to: "me"`, same query as
        // `bridle inbox`) and open questions once a second.
        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let query = MessageQuery {
                to: Some("me".to_string()),
                unread: true,
                ..Default::default()
            };
            loop {
                if let Ok(messages) = client.list_messages(&query).await
                    && tx.send(Message::MessagesLoaded(messages)).is_err()
                {
                    break;
                }
                // Open questions aren't "to me", so `bridle inbox` lists them
                // separately; do the same so an opened question stays listed.
                if let Ok(questions) = client.list_open_questions().await
                    && tx.send(Message::QuestionsLoaded(questions)).is_err()
                {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }
    let mut log_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut log_target: Option<String> = None;

    let mut input = EventStream::new();
    terminal.draw(|f| ui::draw(f, &app))?;
    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    app.on_message(msg);
                }
            }
            ev = input.next() => {
                if let Some(Ok(CrosstermEvent::Key(key))) = ev
                    && let Some(k) = map_key(key)
                {
                    app.on_key(k);
                }
            }
        }

        // `logs_agent` always tracks the selected agent (App::sync_logs_target);
        // restart the poll task whenever it changes so the logs view never
        // shows a stale agent's tail.
        if app.logs_agent.as_deref() != log_target.as_deref() {
            if let Some(handle) = log_task.take() {
                handle.abort();
            }
            log_target = app.logs_agent.clone();
            log_task = log_target
                .clone()
                .map(|id| spawn_transcript_poll(client.clone(), tx.clone(), id));
        }

        if let Some(id) = app.pending_mark_read.take() {
            let client = client.clone();
            tokio::spawn(async move {
                let _ = client.mark_read(&id).await;
            });
        }
        if let Some(id) = app.pending_mark_unread.take() {
            let client = client.clone();
            tokio::spawn(async move {
                let _ = client.mark_unread(&id).await;
            });
        }
        if std::mem::take(&mut app.refresh_agents) {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Ok(agents) = client.list_agents().await {
                    let _ = tx.send(Message::AgentsLoaded(agents));
                }
            });
        }

        if let Some(pending) = app.pending_send.take() {
            spawn_reply(client.clone(), pending);
        }

        terminal.draw(|f| ui::draw(f, &app))?;
        if app.should_quit {
            break;
        }
    }
    if let Some(handle) = log_task {
        handle.abort();
    }
    Ok(())
}

/// Poll `Client::transcript` for `agent`'s tail once a second, same model as
/// `bridle logs --follow` (crates/bridle/src/commands.rs's `logs`): track
/// the last-seen line number locally and pass it as `since`. There's no SSE
/// stream for transcript lines, only for events.
fn spawn_transcript_poll(
    client: Client,
    tx: tokio::sync::mpsc::UnboundedSender<Message>,
    agent: String,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut since = None;
        loop {
            if let Ok(lines) = client.transcript(&agent, since, None).await {
                if let Some(last) = lines.last() {
                    since = Some(last.n);
                }
                if !lines.is_empty()
                    && tx
                        .send(Message::TranscriptLines {
                            agent: agent.clone(),
                            lines,
                        })
                        .is_err()
                {
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    })
}

/// Send a composed reply and, once it lands, mark the original message
/// read -- matching what `bridle inbox --mark-read` does for a message
/// once it's been acted on -- so it drops out of the polled unread inbox
/// on its own without `App` needing to hear back from this task.
fn spawn_reply(client: Client, pending: crate::app::PendingSend) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let req = SendRequest {
            to: Some(pending.to),
            body: pending.body,
            reply_to: Some(pending.reply_to.clone()),
            task: None,
            when: bridle_api::When::Now,
            ..Default::default()
        };
        if client.send(&req).await.is_ok() {
            let _ = client.mark_read(&pending.reply_to).await;
        }
    })
}

fn map_key(key: KeyEvent) -> Option<Key> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Char(c) => Some(Key::Char(c)),
        KeyCode::Up => Some(Key::Up),
        KeyCode::Down => Some(Key::Down),
        KeyCode::Left => Some(Key::Left),
        KeyCode::Right => Some(Key::Right),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Backspace => Some(Key::Backspace),
        KeyCode::Esc => Some(Key::Esc),
        _ => None,
    }
}
