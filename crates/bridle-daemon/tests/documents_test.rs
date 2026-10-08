//! `/v1/documents`, `/v1/links/resolve` and `/v1/specs` (br-5e4k): the human token reads and
//! edits this daemon's repo's documents; nobody else may.

mod support;

use bridle_api::client::{Client, ClientError, DocumentWrite};
use support::start_daemon;

fn status_of(e: ClientError) -> u16 {
    match e {
        ClientError::Api { status, .. } => status,
        other => panic!("expected an API error, got {other:?}"),
    }
}

#[tokio::test]
async fn the_human_reads_searches_resolves_and_edits_a_document() {
    let (daemon, _dir) = start_daemon(None).await;
    std::fs::create_dir_all(daemon.repo.join("docs/design")).expect("mkdir");
    std::fs::write(daemon.repo.join("docs/design/thing.md"), "One.\n").expect("doc");

    let doc = daemon
        .client
        .read_document("docs/design/thing.md")
        .await
        .expect("read");
    assert_eq!(doc.content, "One.\n");

    let found = daemon
        .client
        .search_documents("thing")
        .await
        .expect("search");
    assert_eq!(found.paths, vec!["docs/design/thing.md".to_string()]);

    let links = daemon
        .client
        .resolve_links(&["docs/design/thing".to_string(), "nope".to_string()])
        .await
        .expect("resolve");
    assert_eq!(links.links[0].path.as_deref(), Some("docs/design/thing.md"));
    assert_eq!(links.links[1].path, None);

    assert!(daemon.client.specs().await.expect("specs").specs.is_empty());

    let saved = daemon
        .client
        .write_document(
            "docs/design/thing.md",
            &DocumentWrite {
                content: "Two.\n".into(),
                hash: doc.hash.clone(),
            },
        )
        .await
        .expect("write");
    assert_eq!(
        std::fs::read_to_string(daemon.repo.join("docs/design/thing.md")).expect("read"),
        "Two.\n"
    );

    // The old hash is stale now; the new one works.
    let stale = daemon
        .client
        .write_document(
            "docs/design/thing.md",
            &DocumentWrite {
                content: "Three.\n".into(),
                hash: doc.hash,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(status_of(stale), 409);
    daemon
        .client
        .write_document(
            "docs/design/thing.md",
            &DocumentWrite {
                content: "Three.\n".into(),
                hash: saved.hash,
            },
        )
        .await
        .expect("write with the new hash");
}

#[tokio::test]
async fn bad_and_missing_paths_are_refused() {
    let (daemon, _dir) = start_daemon(None).await;
    assert_eq!(
        status_of(
            daemon
                .client
                .read_document(".git/config")
                .await
                .unwrap_err()
        ),
        400
    );
    assert_eq!(
        status_of(
            daemon
                .client
                .read_document("docs/nope.md")
                .await
                .unwrap_err()
        ),
        404
    );
}

#[tokio::test]
async fn only_the_human_token_may_use_them() {
    let (daemon, _dir) = start_daemon(None).await;
    std::fs::write(daemon.repo.join("a.md"), "A.\n").expect("doc");

    let anon = Client::new(daemon.running.url.clone(), None);
    let write = DocumentWrite {
        content: "B.\n".into(),
        hash: String::new(),
    };
    // A write needs a token (401); a read without one is the synthetic `local` principal,
    // which isn't the human (403).
    assert_eq!(
        status_of(anon.write_document("a.md", &write).await.unwrap_err()),
        401
    );
    assert_eq!(
        status_of(anon.read_document("a.md").await.unwrap_err()),
        403
    );

    let other = daemon.external_client("not-the-human").await;
    assert_eq!(
        status_of(other.read_document("a.md").await.unwrap_err()),
        403
    );
    assert_eq!(
        status_of(other.write_document("a.md", &write).await.unwrap_err()),
        403
    );
    assert_eq!(
        status_of(other.search_documents("a").await.unwrap_err()),
        403
    );
    assert_eq!(status_of(other.specs().await.unwrap_err()), 403);
    assert_eq!(
        status_of(other.resolve_links(&["a".into()]).await.unwrap_err()),
        403
    );
    assert_eq!(
        std::fs::read_to_string(daemon.repo.join("a.md")).expect("read"),
        "A.\n"
    );
}
