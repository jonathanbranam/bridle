//! The CLI's error type: it exists only to carry the exit code (docs/design/cli.md:
//! 0 ok, 1 error, 2 usage error [clap handles that one itself], 3 daemon
//! unreachable) alongside a human-readable message.

use bridle_api::ClientError;
use bridle_api::discovery::DiscoveryError;

#[derive(Debug)]
pub enum CliError {
    /// Exit 3: no daemon could be found or reached.
    Unreachable(String),
    /// Exit 1: anything else.
    Other(anyhow::Error),
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Unreachable(_) => 3,
            CliError::Other(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Unreachable(msg) => write!(f, "{msg}"),
            CliError::Other(e) => write!(f, "{e:#}"),
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::Other(e)
    }
}

impl From<ClientError> for CliError {
    fn from(e: ClientError) -> Self {
        match e {
            ClientError::Unreachable(msg) => CliError::Unreachable(msg),
            other => CliError::Other(anyhow::anyhow!(other)),
        }
    }
}

/// Discovery failures are config/usage problems (bad token, bad workspace),
/// not "couldn't reach a daemon we found" — callers map the specific
/// "no endpoint found" case from `resolve_endpoint` to `Unreachable`
/// themselves, since that's the one cli.md calls out.
impl From<DiscoveryError> for CliError {
    fn from(e: DiscoveryError) -> Self {
        CliError::Other(anyhow::anyhow!(e))
    }
}
