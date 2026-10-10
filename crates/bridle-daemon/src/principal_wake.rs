//! `GET /v1/wake`: the daemon decides when a principal should wake. Every reason lives in
//! [`wake_reasons`] so a new one (schedule, ticket comment) is added there, not in a prompt.
//! Task changes are not a reason of their own: each is a `task_update` message to the task's
//! watchers, so the unread message is the doorbell. The orchestrator's own `wait-for-wake` is separate (wake.rs).

use std::time::Duration;

use bridle_api::types::{PrincipalWakeReason, event_kind};
use tokio::time::Instant;

use crate::events::Emitter;
use crate::store::{ListMessages, Store, StoreError};

/// How often to look again when no event arrives: covers a lagged or closed bus.
const RECHECK: Duration = Duration::from_secs(5);

/// Why `principal` (a resolved message address) should wake now; empty means it shouldn't.
/// With `take` (a caller that isn't the human) the unread messages are returned in full but
/// stay unread: the caller's next wake or inbox acknowledges them (9aj2), so a waiter whose
/// output is lost loses nothing.
pub async fn wake_reasons(
    store: &Store,
    principal: &str,
    take: bool,
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
            message_ids: unread.iter().map(|m| m.id.clone()).collect(),
            messages: if take { unread } else { Vec::new() },
            ..Default::default()
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
    take: bool,
) -> Result<Vec<PrincipalWakeReason>, StoreError> {
    let deadline = Instant::now() + timeout;
    // Subscribe before the first look so a message sent in between isn't missed.
    let mut events = emitter.subscribe();
    loop {
        let reasons = wake_reasons(store, principal, take).await?;
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

#[cfg(test)]
mod tests {
    use bridle_api::types::{MessageKind, MessageState, When};

    use super::*;
    use crate::store::{NewMessage, RecipientKind};

    #[tokio::test]
    async fn a_task_update_survives_a_reopen_and_is_offered_until_read() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("bridle.db");
        let me = "external:advisor";
        let store = Store::open(&path).await.expect("open store");
        store
            .insert_message(NewMessage {
                from: "human".into(),
                to: me.into(),
                to_kind: RecipientKind::External,
                kind: MessageKind::TaskUpdate,
                body: "t-1 (x): comment by human: hi".into(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Written,
            })
            .await
            .expect("message");
        drop(store);
        let store = Store::open(&path).await.expect("reopen");
        let got = wake_reasons(&store, me, true).await.expect("reasons");
        assert_eq!(got.len(), 1, "{got:?}");
        assert_eq!(got[0].reason, "message");
        assert_eq!(got[0].messages[0].kind, MessageKind::TaskUpdate);
        // Handing over doesn't read: the same message is offered until it is acknowledged.
        let again = wake_reasons(&store, me, true).await.expect("again");
        assert_eq!(again[0].message_ids, got[0].message_ids);
        store
            .set_message_state(
                &got[0].messages[0].id,
                MessageState::Read,
                chrono::Utc::now(),
            )
            .await
            .expect("read");
        assert!(
            wake_reasons(&store, me, true)
                .await
                .expect("acked")
                .is_empty()
        );
    }
}
