//! Is this process's own non-loopback address reachable (tickets 7bs2, t9j4, 9qws)? The macOS
//! application firewall blocks incoming connections silently, but it does not filter a connect
//! from the same host (the 2026-11-09 upgrade: both probes passed while the firewall dialog was
//! pending). So the connection has to come from a peer on another machine; only when no peer
//! answers does it fall back to the weak same-host connect.

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
    let machines = MachineMap::load(home).ok()?;
    let req = ConnectBackRequest {
        addr: addr.to_string(),
    };
    for project in machines.projects.keys() {
        let Ok(Some(remote)) = machines.remote(project) else {
            continue;
        };
        let Ok(Some(token)) = discovery::peer_token(&home.join("credentials.toml"), project) else {
            continue;
        };
        let client = Client::new(remote.url, Some(token));
        if let Ok(Ok(verdict)) =
            tokio::time::timeout(ATTEMPT_TIMEOUT, client.connect_back(&req)).await
        {
            return Some(verdict);
        }
    }
    None
}

/// After each wait, one attempt: a peer's verdict (`ask`), else a same-host connect. True as
/// soon as one attempt gets through; false when none does.
pub async fn reachable<F: Future<Output = Option<bool>>>(
    addr: SocketAddr,
    waits: &[Duration],
    ask: impl Fn(SocketAddr) -> F,
) -> bool {
    for wait in waits {
        tokio::time::sleep(*wait).await;
        let got = match ask(addr).await {
            Some(v) => v,
            None => self_connect(addr).await,
        };
        if got {
            return true;
        }
    }
    false
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
        assert!(!reachable(addr, &waits, |_| async { Some(false) }).await);
    }

    #[tokio::test]
    async fn a_peer_that_gets_through_passes_even_if_self_connect_would_not() {
        let (l, addr) = open_listener().await;
        drop(l);
        assert!(reachable(addr, &[Duration::ZERO], |_| async { Some(true) }).await);
    }

    #[tokio::test]
    async fn with_no_peer_it_falls_back_to_the_self_connect() {
        let (l, addr) = open_listener().await;
        assert!(reachable(addr, &[Duration::ZERO], |_| async { None }).await);
        drop(l);
        assert!(!reachable(addr, &[Duration::ZERO], |_| async { None }).await);
    }

    #[tokio::test]
    async fn no_machine_config_means_no_peer_verdict() {
        let tmp = tempfile::tempdir().expect("tmp");
        let (_l, addr) = open_listener().await;
        assert_eq!(ask_peers(tmp.path(), addr).await, None);
    }
}
