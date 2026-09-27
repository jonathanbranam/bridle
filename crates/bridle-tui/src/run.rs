//! Terminal setup/teardown and the event loop: wires a [`bridle_api::Client`]
//! to [`App`], the only place in this crate that touches a real terminal.

use std::io::{self, Stdout};

use bridle_api::Client;
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
    drop(tx);

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
        terminal.draw(|f| ui::draw(f, &app))?;
        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn map_key(key: KeyEvent) -> Option<Key> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Char(c) => Some(Key::Char(c)),
        KeyCode::Up => Some(Key::Up),
        KeyCode::Down => Some(Key::Down),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Esc => Some(Key::Esc),
        _ => None,
    }
}
