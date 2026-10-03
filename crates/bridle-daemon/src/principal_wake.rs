//! `GET /v1/wake`: the daemon decides when a principal should wake. Every reason lives in
//! [`wake_reasons`] so a new one (task comment, schedule, ticket comment) is added there, not
//! in a prompt. The orchestrator's own `wait-for-wake` is separate (wake.rs).

use std::collections::HashSet;
use std::time::Duration;

use bridle_api::types::{EventQuery, PrincipalWakeReason, event_kind};
use tokio::time::Instant;

use crate::events::Emitter;
use crate::store::{ListMessages, Store, StoreError};

/// How often to look again when no event arrives: covers a lagged or closed bus.
const RECHECK: Duration = Duration::from_secs(5);

/// Task events that count as someone else touching a task: a comment, an ask or answer, a
/// priority or kind change, a state change.
const TASK_EVENTS: [&str; 6] = [
    event_kind::TASK_NOTE_ADDED,
    event_kind::TASK_QUESTION_ASKED,
    event_kind::TASK_QUESTION_ANSWERED,
    event_kind::TASK_PRIORITY,
    event_kind::TASK_KIND,
    event_kind::TASK_STATE,
];

/// Why `principal` (a resolved message address) should wake now; empty means it shouldn't.
/// `since_seq` is the event cursor for task reasons: the caller's last wake, or call start.
/// With `take` (a caller that isn't the human) the unread messages are returned in full and
/// marked read in the same store call, so the next wake doesn't repeat them and none is lost.
pub async fn wake_reasons(
    store: &Store,
    principal: &str,
    since_seq: i64,
    take: bool,
) -> Result<Vec<PrincipalWakeReason>, StoreError> {
    let mut reasons = Vec::new();
    let unread = if take {
        store
            .take_unread_messages(principal, chrono::Utc::now())
            .await?
    } else {
        store
            .list_messages(ListMessages {
                to: Some(principal.to_string()),
                unread: true,
                ..Default::default()
            })
            .await?
    };
    if !unread.is_empty() {
        reasons.push(PrincipalWakeReason {
            reason: "message".to_string(),
            message_ids: unread.iter().map(|m| m.id.clone()).collect(),
            messages: if take { unread } else { Vec::new() },
            ..Default::default()
        });
    }
    reasons.extend(task_reasons(store, principal, since_seq).await?);
    Ok(reasons)
}

/// A task the principal created or claimed had one of [`TASK_EVENTS`] by someone else after
/// `since_seq`. "Created or claimed" is the whole rule for now.
async fn task_reasons(
    store: &Store,
    principal: &str,
    since_seq: i64,
) -> Result<Vec<PrincipalWakeReason>, StoreError> {
    let events = store
        .list_events(EventQuery {
            since: Some(since_seq),
            kind: Some("task.".to_string()),
            limit: Some(10_000),
            ..Default::default()
        })
        .await?;
    let touched = |e: &bridle_api::types::Event| {
        TASK_EVENTS.contains(&e.kind.as_str()) && e.actor != principal
    };
    if !events.iter().any(touched) {
        return Ok(Vec::new());
    }
    let mut mine: HashSet<String> = store
        .list_claims()
        .await?
        .into_iter()
        .filter(|c| c.claimed_by == principal)
        .map(|c| c.task_id)
        .collect();
    let created = store
        .list_events(EventQuery {
            kind: Some(event_kind::TASK_CREATED.to_string()),
            limit: Some(1_000_000),
            ..Default::default()
        })
        .await?;
    mine.extend(
        created
            .iter()
            .filter(|e| e.actor == principal)
            .filter_map(|e| e.data["task"].as_str().map(str::to_string)),
    );
    let mut seen = HashSet::new();
    let mut reasons = Vec::new();
    for e in events.iter().filter(|e| touched(e)) {
        let Some(task) = e.data["task"].as_str().filter(|t| mine.contains(*t)) else {
            continue;
        };
        if seen.insert((task.to_string(), e.kind.clone())) {
            reasons.push(PrincipalWakeReason {
                reason: "task".to_string(),
                task: Some(task.to_string()),
                event: Some(e.kind.clone()),
                ..Default::default()
            });
        }
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
    let since_seq = store
        .list_events(EventQuery {
            limit: Some(1),
            ..Default::default()
        })
        .await?
        .last()
        .map_or(0, |e| e.seq);
    loop {
        let reasons = wake_reasons(store, principal, since_seq, take).await?;
        if !reasons.is_empty() || Instant::now() >= deadline {
            return Ok(reasons);
        }
        let until = deadline.min(Instant::now() + RECHECK);
        let _ = tokio::time::timeout_at(until, async {
            loop {
                match events.recv().await {
                    Ok(ev)
                        if ev.kind == event_kind::MESSAGE_SENT
                            || TASK_EVENTS.contains(&ev.kind.as_str()) =>
                    {
                        break;
                    }
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
    async fn task_and_message_reasons_come_back_together() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Store::open(tmp.path().join("bridle.db"))
            .await
            .expect("open store");
        let me = "external:advisor";
        store
            .append_event(
                "task.created",
                me.into(),
                None,
                serde_json::json!({"task": "t-1"}),
            )
            .await
            .expect("created");
        let since = store
            .append_event(
                "task.state",
                me.into(),
                None,
                serde_json::json!({"task": "t-1"}),
            )
            .await
            .expect("own state")
            .seq;
        store
            .append_event(
                "task.note_added",
                "human".into(),
                None,
                serde_json::json!({"task": "t-1"}),
            )
            .await
            .expect("note");
        store
            .insert_message(NewMessage {
                from: "human".into(),
                to: me.into(),
                to_kind: RecipientKind::External,
                kind: MessageKind::Note,
                body: "hi".into(),
                reply_to: None,
                when: When::Now,
                state: MessageState::Written,
            })
            .await
            .expect("message");
        let got = wake_reasons(&store, me, since, false)
            .await
            .expect("reasons");
        let kinds: Vec<_> = got.iter().map(|r| r.reason.as_str()).collect();
        assert_eq!(kinds, ["message", "task"], "{got:?}");
        assert_eq!(got[1].task.as_deref(), Some("t-1"));
        assert_eq!(got[1].event.as_deref(), Some("task.note_added"));
        // Nothing newer than the note: only the unread message is left.
        let got = wake_reasons(&store, me, i64::MAX, false)
            .await
            .expect("reasons");
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got[0].messages.is_empty(),
            "the human's wake carries ids only"
        );

        // A taking wake returns the text and marks it read; a second one finds nothing.
        let got = wake_reasons(&store, me, i64::MAX, true)
            .await
            .expect("reasons");
        assert_eq!(got[0].messages.len(), 1, "{got:?}");
        assert_eq!(got[0].messages[0].body, "hi");
        assert_eq!(got[0].messages[0].state, MessageState::Read);
        let got = wake_reasons(&store, me, i64::MAX, true)
            .await
            .expect("reasons");
        assert!(got.is_empty(), "{got:?}");
        assert_eq!(store.unread_count(me).await.expect("count"), 0);
    }
}
