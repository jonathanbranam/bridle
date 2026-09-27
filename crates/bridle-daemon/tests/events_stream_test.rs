//! §8: SSE backfill + live events, in seq order.

mod support;

use bridle_api::types::{AgentState, SpawnRequest, Workdir};
use futures::StreamExt;
use support::{start_daemon, wait_for_agent, wait_for_state};

#[tokio::test]
async fn events_stream_receives_backfill_then_live_events_in_seq_order() {
    let (daemon, _tmp) = start_daemon(None).await;

    // Produce some events before anyone subscribes (the backfill).
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;

    let backfilled = daemon
        .client
        .events(&bridle_api::types::EventQuery::default())
        .await
        .expect("list events");
    assert!(
        backfilled.len() >= 2,
        "expected some events already recorded"
    );

    let mut stream = Box::pin(daemon.client.events_stream(None));
    let mut seen = Vec::new();
    // Collect the backfill plus at least one live event.
    let target = backfilled.len() + 1;
    let collected = tokio::time::timeout(std::time::Duration::from_secs(15), async {
        while seen.len() < target {
            let ev = stream
                .next()
                .await
                .expect("stream ended early")
                .expect("event ok");
            seen.push(ev);
            if seen.len() == backfilled.len() {
                // Trigger a live event now that the backfill should be drained.
                let _ = daemon
                    .client
                    .send(&bridle_api::types::SendRequest {
                        to: Some("human".to_string()),
                        body: "live event trigger".to_string(),
                        kind: bridle_api::types::MessageKind::Note,
                        when: bridle_api::types::When::Now,
                        reply_to: None,
                    })
                    .await;
            }
        }
        seen.clone()
    })
    .await
    .expect("timed out waiting for backfill + live events");

    // Monotonic, strictly increasing seq.
    for w in collected.windows(2) {
        assert!(w[0].seq < w[1].seq, "events out of order: {:?}", collected);
    }
    assert_eq!(collected.len(), target);
}

#[tokio::test]
async fn events_stream_since_skips_already_seen_events() {
    let (daemon, _tmp) = start_daemon(None).await;
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("hello".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
        })
        .await
        .expect("spawn");
    wait_for_agent(&daemon.client, &agent.id, |a| {
        a.turns >= 1 && a.state == AgentState::Idle
    })
    .await;

    let all = daemon
        .client
        .events(&bridle_api::types::EventQuery::default())
        .await
        .expect("list events");
    assert!(all.len() >= 4, "expected several events, got {}", all.len());
    let cutoff = all[all.len() / 2].seq;

    let mut stream = Box::pin(daemon.client.events_stream(Some(cutoff)));
    let first = tokio::time::timeout(std::time::Duration::from_secs(10), stream.next())
        .await
        .expect("timed out")
        .expect("stream ended")
        .expect("event ok");
    assert!(
        first.seq > cutoff,
        "expected seq > {cutoff}, got {}",
        first.seq
    );
}
