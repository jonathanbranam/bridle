//! The automatic document-review path (ad3t): a still document's pending comments go to its
//! agent from the watcher's own tick, with no `bridle review now`.

mod support;

use std::ops::Not;
use std::time::Duration;

use support::{HANG_GUARD_TIMEOUT, default_overrides, start_daemon_with_config};

// A comment as the gateway leaves it: the newest entry, marked pending.
// The gateway numbers its threads, so the headers carry IDs (the case that broke).
const DOC: &str = "Line.\n\n> [!comment] c1 human, 2026-10-03 14:05, on \"Line\" [pending 2026-10-06 21:19 EDT]\n> Why?\n\nMore.\n\n> [!comment] c2 human, 2026-10-03 14:06, on \"More\" [pending 2026-10-06 21:20 EDT]\n> And?\n";

#[tokio::test]
async fn still_document_is_sent_by_the_tick_alone() {
    let mut o = default_overrides();
    o.doc_watch_interval = Duration::from_millis(100);
    let (daemon, _dir) =
        start_daemon_with_config(Some(o), Some("[review]\nquiet_minutes = 0\n")).await;
    std::fs::create_dir_all(daemon.repo.join(".bridle")).expect("mkdir");
    std::fs::write(daemon.repo.join("doc.md"), DOC).expect("doc");
    std::fs::write(daemon.repo.join(".bridle/review-documents.txt"), "doc.md\n").expect("list");

    let name = bridle_daemon::doc_watch::agent_name("doc.md");
    let deadline = std::time::Instant::now() + HANG_GUARD_TIMEOUT;
    loop {
        let sent = std::fs::read_to_string(daemon.repo.join("doc.md"))
            .expect("read")
            .contains("[pending ")
            .not();
        if sent {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the tick never sent the pending thread"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let agents = daemon.client.list_agents().await.expect("agents");
    let text = std::fs::read_to_string(daemon.repo.join("doc.md")).expect("read");
    // Sent, or already read by the fake agent.
    let marked = text.matches("[sent ").count() + text.matches("[read ").count();
    assert_eq!(marked, 2, "{text}");
    assert!(agents.iter().any(|a| a.name == name), "{agents:?}");
}
