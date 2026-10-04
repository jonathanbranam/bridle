//! `POST /v1/review/add` (jrm2): a UI comment puts its document under review; adding twice
//! changes nothing; a document with no pending thread stays out when asked.

mod support;

use bridle_api::types::ReviewAddRequest;
use support::start_daemon;

const DOC: &str = "Line.\n\n> [!comment] human, 2026-10-03 14:05, on \"Line\"\n> Why?\n";

#[tokio::test]
async fn review_add_registers_once_and_only_if_pending() {
    let (daemon, _dir) = start_daemon(None).await;
    std::fs::write(daemon.repo.join("doc.md"), DOC).expect("doc");
    std::fs::write(daemon.repo.join("plain.md"), "Line.\n").expect("plain");
    let req = |path: &str, only_if_pending| ReviewAddRequest {
        path: path.into(),
        only_if_pending,
    };
    let list = || std::fs::read_to_string(daemon.repo.join(".bridle/review-documents.txt"));

    let r = daemon
        .client
        .review_add(&req("plain.md", true))
        .await
        .expect("plain");
    assert!(!r.under_review);
    assert!(list().is_err());

    for _ in 0..2 {
        let r = daemon
            .client
            .review_add(&req("doc.md", true))
            .await
            .expect("add");
        assert!(r.under_review);
    }
    assert_eq!(list().expect("list"), "doc.md\n");

    assert!(
        daemon
            .client
            .review_add(&req("../x.md", false))
            .await
            .is_err()
    );
    assert!(
        daemon
            .client
            .review_add(&req("nope.md", false))
            .await
            .is_err()
    );
}
