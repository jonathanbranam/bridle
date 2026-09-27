//! bridle's terminal UI. An Elm-style split: `app` holds state and pure
//! update logic (no terminal/ratatui dependency, so it's unit-testable on
//! its own), `ui` renders an [`app::App`] with ratatui, and `run` wires a
//! `bridle_api::Client` to both over a real terminal. See docs/design/cli.md.

pub mod app;
pub mod format;
mod run;
mod ui;

pub use app::App;
pub use run::run;
