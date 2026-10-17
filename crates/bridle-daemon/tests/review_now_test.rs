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
    assert_eq!(
        (r.agent.as_str(), r.threads),
        (&bridle_daemon::doc_watch::agent_name("doc.md")[..], 1)
    );
    let text = std::fs::read_to_string(daemon.repo.join("doc.md")).expect("read");
    assert_eq!(text.matches("[sent 20").count(), 1);
    assert!(text.contains("[!comment] c1 human"));
    assert!(
        text.lines()
            .any(|l| l.contains("on \"Line\" [sent ") && l.ends_with("T]"))
    );

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

#[tokio::test]
async fn a_mention_in_a_document_comment_tells_that_role_once_and_an_escaped_one_nobody() {
    let (daemon, _dir) = start_daemon(None).await;
    let orch = daemon.external_client("orchestrator").await;
    let aide = daemon.external_client("aide").await;
    let doc = "Line.\n\n\
        > [!comment] c1 human, 2026-10-03 14:05, on \"Line\"\n\
        > @orchestrator please look, again @orchestrator; but \\@aide is plain text. @nobody-here?\n";
    std::fs::create_dir_all(daemon.repo.join(".bridle")).expect("mkdir");
    std::fs::write(daemon.repo.join("doc.md"), doc).expect("doc");
    std::fs::write(daemon.repo.join(".bridle/review-documents.txt"), "doc.md\n").expect("list");
    let cfg = daemon.repo.join(".bridle/config.toml");
    let mut text = std::fs::read_to_string(&cfg).unwrap_or_default();
    text.push_str("\n[gateway]\npublic_url = \"http://ui.example\"\n");
    std::fs::write(&cfg, text).expect("config");
    let req = ReviewNowRequest {
        path: "doc.md".into(),
        resend: false,
    };
    daemon.client.review_now(&req).await.expect("now");
    let told = |c: &bridle_api::Client, to: &str| {
        let (c, to) = (c.clone(), to.to_string());
        async move {
            c.list_messages(&bridle_api::types::MessageQuery {
                to: Some(to),
                ..Default::default()
            })
            .await
            .expect("messages")
            .into_iter()
            .map(|m| m.body)
            .collect::<Vec<_>>()
        }
    };
    let lines = told(&orch, "me").await;
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("document doc.md http://ui.example/p/")
            && lines[0].contains("/docs?path=doc.md: human mentioned you"),
        "{lines:?}"
    );
    assert!(told(&aide, "me").await.is_empty());

    // Marked sent, so a plain second look tells nobody again.
    daemon.client.review_now(&req).await.expect("again");
    assert_eq!(told(&orch, "me").await.len(), 1);
}
