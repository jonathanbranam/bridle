//! `bridle gateway`: runs the gateway in the foreground. Its config is read here and only
//! here, so a bad `[gateway]` section can't affect any other command.
//! See docs/design/human-web-ui.md.

use anyhow::Context;
use bridle_gateway::GatewayConfig;

use crate::error::CliError;

pub async fn run() -> Result<(), CliError> {
    let home = bridle_api::discovery::bridle_home();
    let config = GatewayConfig::load(&home).map_err(anyhow::Error::from)?;
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .try_init()
        .ok();
    let listener = bridle_gateway::bind(&config)
        .await
        .with_context(|| format!("binding the gateway to {}", config.bind))?;
    bridle_gateway::serve(listener).await.context("gateway")?;
    Ok(())
}
