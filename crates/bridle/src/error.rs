//! The CLI's error type: it exists only to carry the exit code (docs/design/cli.md:
//! 0 ok, 1 error, 2 usage error [clap handles that one itself], 3 daemon
//! unreachable, 4 `wait` timed out, 5 a wake was superseded or stopped, 6 the daemon ended a wait because it is restarting or stopping) alongside a human-readable message.

use bridle_api::ClientError;
use bridle_api::discovery::DiscoveryError;

/// `sysexits.h` EX_CONFIG.
pub const OWNER_REFUSED_EXIT: u8 = 78;

#[derive(Debug)]
pub enum CliError {
    /// Exit 3: no daemon could be found or reached.
    Unreachable(String),
    /// Exit 4: `bridle wait` gave up at `--timeout`.
    Timeout(String),
    /// Exit 6: the daemon ended the wait because it is restarting or shutting down.
    Stopping(String),
    /// Exit 5: the wait was ended by a newer wait from the same session, or by `--stop`.
    Superseded(String),
    /// Exit 78 (EX_CONFIG): `serve` refused because another machine owns the project. Final for a
    /// supervisor: the systemd unit lists 78 in `RestartPreventExitStatus`, the launchd wrapper maps it to 0.
    OwnerRefused(String),
    /// Exit 1: anything else.
    Other(anyhow::Error),
}

impl CliError {
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Unreachable(_) => 3,
            CliError::Timeout(_) => 4,
            CliError::Superseded(_) => 5,
            CliError::Stopping(_) => 6,
            CliError::OwnerRefused(_) => OWNER_REFUSED_EXIT,
            CliError::Other(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Unreachable(msg)
            | CliError::Timeout(msg)
            | CliError::Superseded(msg)
            | CliError::Stopping(msg)
            | CliError::OwnerRefused(msg) => {
                write!(f, "{msg}")
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_refusal_exits_78() {
        assert_eq!(CliError::OwnerRefused("x".into()).exit_code(), 78);
    }
}
