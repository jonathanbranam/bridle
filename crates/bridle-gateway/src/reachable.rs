//! After a start, the gateway connects to its own non-loopback address (ticket t9j4, follow-up
//! to 7bs2). The macOS application firewall blocks incoming connections to a binary it has not
//! been told to allow, silently, and the gateway is the process that is blocked; the daemon's
//! own check (`probe_own_address` there) cannot see it. When the address never answers the
//! gateway tells the orchestrator of each project on this machine.

use std::net::SocketAddr;
use std::time::Duration;

use bridle_api::client::Client;
use bridle_api::types::{MessageKind, SendRequest, When};

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

/// Connects to `addr` after each wait until one succeeds; `false` when none does.
pub async fn reachable(addr: SocketAddr, waits: &[Duration]) -> bool {
    for wait in waits {
        tokio::time::sleep(*wait).await;
        let connect = tokio::net::TcpStream::connect(addr);
        if matches!(
            tokio::time::timeout(Duration::from_secs(5), connect).await,
            Ok(Ok(_))
        ) {
            return true;
        }
    }
    false
}

/// Probes `addr`; if it never answers, logs an error and passes the incident text to `raise`.
/// Returns whether it raised.
pub async fn check(addr: SocketAddr, waits: &[Duration], raise: impl AsyncFnOnce(String)) -> bool {
    if reachable(addr, waits).await {
        return false;
    }
    tracing::error!(%addr, "the gateway cannot reach its own address; firewall?");
    raise(unreachable_text(addr)).await;
    true
}

/// Sends the incident as a note to `external:orchestrator` of every project on this machine,
/// with the human's token (the gateway's only identity).
pub async fn tell_orchestrators(text: String) {
    for t in current_targets().await {
        if t.workspace.is_none() {
            continue;
        }
        let Ok((url, token)) = resolve(&t.project).await else {
            continue;
        };
        let client = Client::new_with_timeout(url, Some(token), PROBE_TIMEOUT * 5);
        let sent = client
            .send(&SendRequest {
                to: Some("external:orchestrator".to_string()),
                body: text.clone(),
                kind: MessageKind::Note,
                when: When::Now,
                reply_to: None,
                task: None,
            })
            .await;
        if let Err(e) = sent {
            tracing::warn!(project = %t.project, "could not report the unreachable gateway: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn raises_only_when_the_listener_is_unreachable() {
        let waits = [Duration::ZERO; 2];
        let l = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = l.local_addr().expect("addr");
        let said = Mutex::new(Vec::new());
        let raised = check(addr, &waits, async |t| said.lock().expect("lock").push(t)).await;
        assert!(!raised && said.lock().expect("lock").is_empty());
        // A fake unreachable listener: the port is closed.
        drop(l);
        let raised = check(addr, &waits, async |t| said.lock().expect("lock").push(t)).await;
        let got = said.lock().expect("lock").clone();
        assert!(raised && got.len() == 1);
        assert!(got[0].contains("firewall") && got[0].contains(&addr.to_string()));
    }
}
