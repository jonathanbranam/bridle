//! `bridle review now` (x8jt): sends a document's pending threads at once, marks them sent in
//! the file, skips marked threads unless resent.

mod support;

use bridle_api::types::ReviewNowRequest;
use support::start_daemon;

const DOC: &str = "Line.\n\n> [!comment] human, 2026-10-03 14:05, on \"Line\"\n> Why?\n";

#[tokio::test]
async fn review_now_sends_marks_and_skips_marked() {
    let (daemon, _dir) = start_daemon(None).await;
    std::fs::create_dir_all(daemon.repo.join(".bridle")).expect("mkdir");
    std::fs::write(daemon.repo.join("doc.md"), DOC).expect("doc");
    std::fs::write(daemon.repo.join(".bridle/review-documents.txt"), "doc.md\n").expect("list");
    let req = |resend| ReviewNowRequest {
        path: "doc.md".into(),
        resend,
    };

    // Not waiting out any quiet period.
    let r = daemon.client.review_now(&req(false)).await.expect("now");
    assert_eq!((r.agent.as_str(), r.threads), ("doc-doc", 1));
    let text = std::fs::read_to_string(daemon.repo.join("doc.md")).expect("read");
    assert_eq!(text.matches(" · sent 20").count(), 1);
    assert!(text.contains("on \"Line\" · sent "));

    // Marked: not sent again, and the file is untouched.
    let r = daemon.client.review_now(&req(false)).await.expect("again");
    assert_eq!(r.threads, 0);
    assert_eq!(
        std::fs::read_to_string(daemon.repo.join("doc.md")).expect("read"),
        text
    );

    // Resend sends it again.
    let r = daemon.client.review_now(&req(true)).await.expect("resend");
    assert_eq!(r.threads, 1);

    // A document that isn't under review is refused.
    let e = daemon
        .client
        .review_now(&ReviewNowRequest {
            path: "other.md".into(),
            resend: false,
        })
        .await;
    assert!(e.is_err());
}
