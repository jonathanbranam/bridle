//! `GET /v1/wake`: the daemon decides when a principal should wake. Every reason lives in
//! [`wake_reasons`] so a new one (task comment, schedule, ticket comment) is added there, not
//! in a prompt. The orchestrator's own `wait-for-wake` is separate (wake.rs).

use std::time::Duration;

use bridle_api::types::{PrincipalWakeReason, event_kind};
use tokio::time::Instant;

use crate::events::Emitter;
use crate::store::{ListMessages, Store, StoreError};

/// How often to look again when no event arrives: covers a lagged or closed bus.
const RECHECK: Duration = Duration::from_secs(5);

/// Why `principal` (a resolved message address) should wake now; empty means it shouldn't.
pub async fn wake_reasons(
    store: &Store,
    principal: &str,
) -> Result<Vec<PrincipalWakeReason>, StoreError> {
    let mut reasons = Vec::new();
    let unread = store
        .list_messages(ListMessages {
            to: Some(principal.to_string()),
            unread: true,
            ..Default::default()
        })
        .await?;
    if !unread.is_empty() {
        reasons.push(PrincipalWakeReason {
            reason: "message".to_string(),
            message_ids: unread.into_iter().map(|m| m.id).collect(),
        });
    }
    Ok(reasons)
}

/// Holds until [`wake_reasons`] is non-empty or `timeout` passes (then empty).
pub async fn wait(
    store: &Store,
    emitter: &Emitter,
    principal: &str,
    timeout: Duration,
) -> Result<Vec<PrincipalWakeReason>, StoreError> {
    let deadline = Instant::now() + timeout;
    // Subscribe before the first look so a message sent in between isn't missed.
    let mut events = emitter.subscribe();
    loop {
        let reasons = wake_reasons(store, principal).await?;
        if !reasons.is_empty() || Instant::now() >= deadline {
            return Ok(reasons);
        }
        let until = deadline.min(Instant::now() + RECHECK);
        let _ = tokio::time::timeout_at(until, async {
            loop {
                match events.recv().await {
                    Ok(ev) if ev.kind == event_kind::MESSAGE_SENT => break,
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    // No sender left: fall back to the timed recheck.
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        std::future::pending::<()>().await
                    }
                }
            }
        })
        .await;
    }
}
