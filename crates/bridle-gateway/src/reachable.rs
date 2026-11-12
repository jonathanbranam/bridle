//! After a start, the gateway connects to its own non-loopback address (ticket t9j4, follow-up
//! to 7bs2). The macOS application firewall blocks incoming connections to a binary it has not
//! been told to allow, silently, and the gateway is the process that is blocked; the daemon's
//! own check (`probe_own_address` there) cannot see it. When the address never answers the
//! gateway tells the orchestrator of each project on this machine.

use std::net::SocketAddr;
use std::time::Duration;

use bridle_api::client::Client;
use bridle_api::probe::Verdict;

use crate::actions::resolve;
use crate::discovery::{PROBE_TIMEOUT, current_targets};

pub const PROBE_WAITS: [Duration; 3] = [
    Duration::from_secs(3),
    Duration::from_secs(10),
    Duration::from_secs(10),
];

/// The incident text when the gateway cannot reach its own non-loopback address.
pub fn unreachable_text(addr: SocketAddr) -> String {
    format!(
        "The gateway cannot connect to its own address {addr} after a start. On macOS the likely \
         cause is the application firewall blocking the installed `bridle` until its \"Allow\" \
         dialog is answered (check the screen of the Mac, or the signature in the gateway log). \
         The web UI is not reachable from other machines until then."
    )
}

/// The text when no peer could say whether the gateway's address is reachable.
pub fn unverified_text(addr: SocketAddr) -> String {
    format!(
        "Reachability unverified: no peer answered when the gateway's address {addr} was \
         probed; a same-host connect proves nothing on macOS, so a blocked application firewall \
         would go unnoticed. Upgrade the other machines' bridle, or check their daemons."
    )
}

/// Probes `addr` (from a peer machine when one answers, ticket 9qws); if it is blocked, logs an error and passes the incident text to `raise`; if no
/// peer could say, passes the (lower-grade) unverified text to `raise` once. Returns whether it raised.
pub async fn check<F: Future<Output = Option<bool>>>(
    addr: SocketAddr,
    waits: &[Duration],
    ask: impl Fn(SocketAddr) -> F,
    raise: impl AsyncFnOnce(String),
) -> bool {
    match bridle_api::probe::reachable(addr, waits, ask).await {
        Verdict::Reachable => false,
        Verdict::Unverified => {
            tracing::warn!(%addr, "the gateway's reachability is unverified: no peer answered");
            raise(unverified_text(addr)).await;
            true
        }
        Verdict::Blocked => {
            tracing::error!(%addr, "the gateway cannot reach its own address; firewall?");
            raise(unreachable_text(addr)).await;
            true
        }
    }
}

/// Raises the incident on every project's daemon on this machine, with the human's token (the
/// gateway's only identity): each records an `orchestrator.incident` event and notes the human
/// and its orchestrator.
pub async fn tell_orchestrators(text: String) {
    for t in current_targets().await {
        if t.workspace.is_none() {
            continue;
        }
        let Ok((url, token)) = resolve(&t.project).await else {
            continue;
        };
        let client = Client::new_with_timeout(url, Some(token), PROBE_TIMEOUT * 5);
        if let Err(e) = client.report_incident(&text).await {
            tracing::warn!(project = %t.project, "could not report the unreachable gateway: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn raises_when_a_peer_is_blocked_though_the_self_connect_passes() {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = l.local_addr().expect("addr");
        let said = Mutex::new(Vec::new());
        let raised = check(
            addr,
            &[Duration::ZERO],
            |_| async { Some(false) },
            async |t| said.lock().expect("lock").push(t),
        )
        .await;
        assert!(raised && said.lock().expect("lock").len() == 1);
    }

    #[tokio::test]
    async fn a_reachable_peer_raises_nothing() {
        let said = Mutex::new(Vec::new());
        let addr: SocketAddr = "127.0.0.1:9".parse().expect("addr");
        let raised = check(
            addr,
            &[Duration::ZERO],
            |_| async { Some(true) },
            async |t| said.lock().expect("lock").push(t),
        )
        .await;
        assert!(!raised && said.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn no_peer_answer_raises_the_unverified_text_once_not_blocked() {
        let waits = [Duration::ZERO; 2];
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = l.local_addr().expect("addr");
        let said = Mutex::new(Vec::new());
        let raised = check(
            addr,
            &waits,
            |_| async { None },
            async |t| said.lock().expect("lock").push(t),
        )
        .await;
        let got = said.lock().expect("lock").clone();
        assert!(raised && got.len() == 1);
        assert!(got[0].contains("unverified") && !got[0].contains("cannot connect"));
    }

    #[tokio::test]
    async fn a_closed_port_with_no_peer_is_blocked() {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = l.local_addr().expect("addr");
        drop(l);
        let said = Mutex::new(Vec::new());
        let raised = check(
            addr,
            &[Duration::ZERO; 2],
            |_| async { None },
            async |t| said.lock().expect("lock").push(t),
        )
        .await;
        let got = said.lock().expect("lock").clone();
        assert!(raised && got.len() == 1);
        assert!(got[0].contains("firewall") && got[0].contains(&addr.to_string()));
    }
}
