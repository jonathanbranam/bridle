//! Runs the real `claude` binary. Costs tokens, so it's `#[ignore]`d and
//! gated on `BRIDLE_LIVE_TESTS=1`. Run with:
//!
//!     BRIDLE_LIVE_TESTS=1 cargo test -p bridle-claude --test live_test -- --ignored
//!
//! or `just test-live` once the daemon has that recipe.

use std::time::{Duration, Instant};

use bridle_claude::events::EventKind;
use bridle_claude::{ClaudeCommand, Session, Transcript, spawn};
use tokio::time::timeout;
use uuid::Uuid;

#[tokio::test]
#[ignore = "spends real tokens against the real claude binary"]
async fn one_turn_against_real_claude_haiku() {
    if std::env::var("BRIDLE_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping: set BRIDLE_LIVE_TESTS=1 to run this test");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let mut cmd = ClaudeCommand::new(dir.path(), Session::New(Uuid::new_v4()));
    cmd.model = Some("haiku".to_string());
    let transcript = Transcript::open(&dir.path().join("transcript.jsonl"), Instant::now())
        .expect("open transcript");

    let mut spawned = spawn(&cmd, transcript).await.expect("spawn real claude");
    spawned
        .handle
        .send_user("Reply with exactly: OK")
        .expect("send_user");

    let result = timeout(Duration::from_secs(60), async {
        loop {
            let ev = spawned
                .events
                .recv()
                .await
                .expect("events channel ended before result");
            if let EventKind::Result(r) = ev.kind {
                return r;
            }
        }
    })
    .await
    .expect("timed out waiting for a result from real claude");

    assert_eq!(result.subtype, "success");
    assert!(!result.is_error);

    spawned.handle.close_stdin();
    let outcome = timeout(Duration::from_secs(10), spawned.exit)
        .await
        .expect("exit timeout")
        .expect("exit sender dropped");
    assert_eq!(outcome.code, Some(0));
}
