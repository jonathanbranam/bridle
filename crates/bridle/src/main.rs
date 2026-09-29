//! `bridle`: the CLI. A thin client of the daemon's API
//! (docs/design/agent-host/api.md), plus `serve`, which runs the daemon itself.

mod arch_guard;
mod cli;
mod commands;
mod doctor;
mod error;
mod goals;
mod launchd;
mod prime;
mod render;
mod serve;
mod spec_coverage;
mod spec_export;
mod spec_import;
mod specid;
mod statusline;
mod stop_check;
mod trace;

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
