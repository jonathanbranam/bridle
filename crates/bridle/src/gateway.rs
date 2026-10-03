//! `bridle gateway`: runs the gateway in the foreground. Its config is read here and only
//! here, so a bad `[gateway]` section can't affect any other command.
//! See docs/design/human-web-ui.md.

use anyhow::Context;
use bridle_gateway::GatewayConfig;

use crate::cli::{GatewayArgs, GatewayCommand};
use crate::error::CliError;

pub async fn run(args: &GatewayArgs) -> Result<(), CliError> {
    if let Some(GatewayCommand::HashPassword) = args.command {
        return hash_password();
    }
    let home = bridle_api::discovery::bridle_home();
    let config = GatewayConfig::load(&home).map_err(anyhow::Error::from)?;
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .try_init()
        .ok();
    let listener = bridle_gateway::bind(&config)
        .await
        .with_context(|| format!("binding the gateway to {}", config.bind))?;
    bridle_gateway::serve(listener, config.login)
        .await
        .context("gateway")?;
    Ok(())
}

/// Reads from stdin, not an argument, so the password stays out of shell history and `ps`.
fn hash_password() -> Result<(), CliError> {
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("reading the password from stdin")?;
    let password = line.trim_end_matches(['\r', '\n']);
    if password.is_empty() {
        return Err(anyhow::anyhow!("no password on stdin").into());
    }
    let hash = bridle_gateway::auth::hash_password(password)
        .map_err(|e| anyhow::anyhow!("hashing: {e}"))?;
    println!("{hash}");
    Ok(())
}
