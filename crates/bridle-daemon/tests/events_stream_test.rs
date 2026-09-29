//! §8: SSE backfill + live events, in seq order.

mod support;

use bridle_api::types::{AgentState, SpawnRequest, Workdir};
use futures::StreamExt;
use support::{TestDaemon, start_daemon, wait_for_agent, wait_for_state};

#[tokio::test]
async fn events_stream_receives_backfill_then_live_events_in_seq_order() {
    let (daemon, _tmp) = start_daemon(None).await;

    // Produce some events before anyone subscribes (the backfill).
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
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

    // since=Some(0) is a cursor, so this still exercises backfill-then-live,
    // unlike since=None (no cursor), which starts at the tail (see
    // `events_stream_with_no_cursor_skips_backfill_and_starts_at_the_tail`).
    let mut stream = Box::pin(daemon.client.events_stream(Some(0)));
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
                        task: None,
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
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: Some("hello".to_string()),
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
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
    let first = tokio::time::timeout(support::HANG_GUARD_TIMEOUT, stream.next())
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

#[tokio::test]
async fn events_stream_with_no_cursor_skips_backfill_and_starts_at_the_tail() {
    let (daemon, _tmp) = start_daemon(None).await;

    // Produce some history before anyone subscribes.
    let agent = daemon
        .client
        .spawn(&SpawnRequest {
            components: Vec::new(),
            role: "worker".to_string(),
            name: Some("w1".to_string()),
            prompt: None,
            workdir: Some(Workdir::Repo),
            model: None,
            extra_allowed_tools: Vec::new(),
            extra_env: Vec::new(),
            ignore_budget: false,
        })
        .await
        .expect("spawn");
    wait_for_state(&daemon.client, &agent.id, AgentState::Idle).await;
    let existing = daemon
        .client
        .events(&bridle_api::types::EventQuery::default())
        .await
        .expect("list events");
    assert!(!existing.is_empty(), "expected some prior history");

    // A fresh `--follow` with no `--since` (since=None) shouldn't replay
    // that history — only a live event triggered after subscribing. The
    // client stream is lazy (nothing is sent until first polled), so send
    // the trigger repeatedly in the background rather than once up front:
    // a single send could land before the subscribe that happens on the
    // server once the SSE request actually arrives.
    let mut stream = Box::pin(daemon.client.events_stream(None));
    let trigger_client = daemon.client.clone();
    tokio::spawn(async move {
        for _ in 0..40 {
            let _ = trigger_client
                .send(&bridle_api::types::SendRequest {
                    to: Some("human".to_string()),
                    body: "live event trigger".to_string(),
                    kind: bridle_api::types::MessageKind::Note,
                    when: bridle_api::types::When::Now,
                    reply_to: None,
                    task: None,
                })
                .await;
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });
    let first = tokio::time::timeout(support::HANG_GUARD_TIMEOUT, stream.next())
        .await
        .expect("timed out")
        .expect("stream ended")
        .expect("event ok");
    assert!(
        first.seq > existing.last().unwrap().seq,
        "expected the tail, not backfilled history: got seq {}",
        first.seq
    );
}

#[tokio::test]
async fn shutdown_ends_open_event_streams_and_finishes_promptly() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let (daemon, _tmp) = start_daemon(None).await;
    // Raw HTTP, because `Client::events_stream` reconnects forever and would
    // hide the server closing the stream.
    let token = daemon
        .client
        .create_token(&bridle_api::types::TokenCreateRequest {
            name: "raw-sse".to_string(),
        })
        .await
        .expect("token")
        .token;
    let addr = daemon.running.url.trim_start_matches("http://").to_string();
    let mut conn = tokio::net::TcpStream::connect(&addr)
        .await
        .expect("connect");
    conn.write_all(
        format!(
            "GET /v1/events/stream HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer {token}\r\n\r\n"
        )
        .as_bytes(),
    )
    .await
    .expect("write");
    let mut buf = [0u8; 1024];
    let n = conn.read(&mut buf).await.expect("read headers");
    assert!(String::from_utf8_lossy(&buf[..n]).starts_with("HTTP/1.1 200"));

    let TestDaemon { running, .. } = daemon;
    running.shutdown();
    tokio::time::timeout(std::time::Duration::from_secs(3), running.join())
        .await
        .expect("daemon shutdown hung on the open event stream")
        .expect("join");

    // The client is still holding the connection; it must see EOF.
    let end = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while conn.read(&mut buf).await.map(|n| n > 0).unwrap_or(false) {}
    })
    .await;
    assert!(end.is_ok(), "stream did not end");
}
