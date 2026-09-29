//! `bridle`: the CLI. A thin client of the daemon's API
//! (docs/design/agent-host/api.md), plus `serve`, which runs the daemon itself.

mod cli;
mod commands;
mod error;
mod launchd;
mod prime;
mod render;
mod serve;
mod spec_export;
mod statusline;
mod stop_check;

use std::process::ExitCode;

use clap::Parser;

use cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match commands::run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}
