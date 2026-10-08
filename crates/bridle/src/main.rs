//! `bridle`: the CLI. A thin client of the daemon's API
//! (docs/design/agent-host/api.md), plus `serve`, which runs the daemon itself.

mod advisor;
mod arch_guard;
mod cli;
mod commands;
mod doctor;
mod error;
mod focus;
mod gateway;
mod goals;
mod init;
mod kill_guard;
mod launchd;
mod link;
mod mail_install;
mod migrate;
mod orchestrator;
mod pane;
mod prime;
mod project;
mod render;
mod review;
mod serve;
mod session;
mod sign;
mod spec_coverage;
mod spec_export;
mod spec_import;
mod specid;
mod statusline;
mod stop_check;
mod systemd;
mod ticket;
mod tools_only;
mod trace;
mod vendor;

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
