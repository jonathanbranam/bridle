//! Is this process's own non-loopback address reachable (tickets 7bs2, t9j4, 9qws)? The macOS
//! application firewall blocks incoming connections silently, but it does not filter a connect
//! from the same host (the 2026-11-09 upgrade: both probes passed while the firewall dialog was
//! pending). So the connection has to come from a peer on another machine. When no peer answers
//! (none upgraded yet, none reachable) the same-host connect can only prove the listener is
//! down, never that it is reachable: the result is `Unverified` (ticket maj3), not a pass.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use crate::client::Client;
use crate::discovery;
use crate::machines::MachineMap;
use crate::types::ConnectBackRequest;

const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(10);

/// Connects to `addr` from this host.
pub async fn self_connect(addr: SocketAddr) -> bool {
    matches!(
        tokio::time::timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(addr)).await,
        Ok(Ok(_))
    )
}

/// Asks the daemons on other machines (those we hold a peer token for), in turn, to connect to
/// `addr`. The first verdict wins; `None` when no peer answered.
pub async fn ask_peers(home: &Path, addr: SocketAddr) -> Option<bool> {
    let machines = match MachineMap::load(home) {
        Ok(m) => m,
        Err(e) => {
            tracing::info!(%addr, "probe: no machine map, no peer to ask: {e}");
            return None;
        }
    };
    let req = ConnectBackRequest {
        addr: addr.to_string(),
    };
    for project in machines.projects.keys() {
        let remote = match machines.remote(project) {
            Ok(Some(r)) => r,
            Ok(None) => {
                tracing::info!(%addr, %project, "probe: skipped peer: it is on this machine");
                continue;
            }
            Err(e) => {
                tracing::info!(%addr, %project, "probe: skipped peer: {e}");
                continue;
            }
        };
        let token = match discovery::peer_token(&home.join("credentials.toml"), project) {
            Ok(Some(t)) => t,
            Ok(None) => {
                tracing::info!(%addr, %project, "probe: skipped peer: no peer token");
                continue;
            }
            Err(e) => {
                tracing::info!(%addr, %project, "probe: skipped peer: token error: {e}");
                continue;
            }
        };
        let client = Client::new(remote.url, Some(token));
        match tokio::time::timeout(ATTEMPT_TIMEOUT, client.connect_back(&req)).await {
            Ok(Ok(verdict)) => {
                tracing::info!(%addr, %project, reachable = verdict, "probe: peer answered");
                return Some(verdict);
            }
            Ok(Err(e)) => {
                tracing::info!(%addr, %project, "probe: skipped peer: {e}");
            }
            Err(_) => {
                tracing::info!(%addr, %project, "probe: skipped peer: no answer in time");
            }
        }
    }
    tracing::info!(%addr, "probe: no peer gave a verdict");
    None
}

/// What the probe could establish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// A peer connected back.
    Reachable,
    /// A peer could not connect, or (no peer) not even a same-host connect works.
    Blocked,
    /// No peer answered and the same-host connect worked, which proves nothing on macOS.
    Unverified,
}

/// After each wait, one attempt: a peer's verdict (`ask`), else a same-host connect. A peer's
/// success ends it `Reachable`. Without a peer verdict a passing same-host connect only counts
/// as `Unverified`; a failing one, or a peer's refusal, as `Blocked`. Each attempt is logged.
pub async fn reachable<F: Future<Output = Option<bool>>>(
    addr: SocketAddr,
    waits: &[Duration],
    ask: impl Fn(SocketAddr) -> F,
) -> Verdict {
    let mut verdict = Verdict::Blocked;
    for wait in waits {
        tokio::time::sleep(*wait).await;
        let got = match ask(addr).await {
            Some(true) => {
                tracing::info!(%addr, "probe: reachable (a peer connected back)");
                return Verdict::Reachable;
            }
            Some(false) => {
                tracing::info!(%addr, "probe: a peer could not connect back");
                Verdict::Blocked
            }
            None if self_connect(addr).await => {
                tracing::info!(%addr, "probe: no peer verdict; same-host connect worked (proves nothing)");
                Verdict::Unverified
            }
            None => {
                tracing::info!(%addr, "probe: no peer verdict; same-host connect failed");
                Verdict::Blocked
            }
        };
        // The latest attempt is the freshest evidence (a peer may come up between attempts).
        verdict = got;
    }
    tracing::info!(%addr, ?verdict, "probe: final verdict");
    verdict
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn open_listener() -> (tokio::net::TcpListener, SocketAddr) {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let a = l.local_addr().expect("addr");
        (l, a)
    }

    #[tokio::test]
    async fn a_blocked_peer_verdict_beats_a_passing_self_connect() {
        let (_l, addr) = open_listener().await;
        let waits = [Duration::ZERO; 2];
        let v = reachable(addr, &waits, |_| async { Some(false) }).await;
        assert_eq!(v, Verdict::Blocked);
    }

    #[tokio::test]
    async fn a_peer_that_gets_through_passes_even_if_self_connect_would_not() {
        let (l, addr) = open_listener().await;
        drop(l);
        let v = reachable(addr, &[Duration::ZERO], |_| async { Some(true) }).await;
        assert_eq!(v, Verdict::Reachable);
    }

    #[tokio::test]
    async fn with_no_peer_a_same_host_pass_is_only_unverified() {
        let (l, addr) = open_listener().await;
        let v = reachable(addr, &[Duration::ZERO], |_| async { None }).await;
        assert_eq!(v, Verdict::Unverified);
        drop(l);
        let v = reachable(addr, &[Duration::ZERO], |_| async { None }).await;
        assert_eq!(v, Verdict::Blocked);
    }

    #[tokio::test]
    async fn no_machine_config_means_no_peer_verdict() {
        let tmp = tempfile::tempdir().expect("tmp");
        let (_l, addr) = open_listener().await;
        assert_eq!(ask_peers(tmp.path(), addr).await, None);
    }
}
