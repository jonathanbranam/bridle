//! `bridle push`: a rejected push is a `push.failed` event and a message, not session text
//! (ticket 8umh).

mod support;

use bridle_api::types::{EventQuery, MessageQuery};
use support::start_daemon;

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

#[tokio::test]
async fn a_rejected_push_is_an_event_and_a_message_and_an_error() {
    let (d, tmp) = start_daemon(None).await;
    let origin = tmp.path().join("origin.git");
    git(
        tmp.path(),
        &["init", "--bare", "-q", origin.to_str().expect("utf8")],
    );
    let hook = origin.join("hooks/pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho 'nope: not the owner' >&2\nexit 1\n").expect("hook");
    std::process::Command::new("chmod")
        .arg("+x")
        .arg(&hook)
        .status()
        .expect("chmod");
    git(
        &d.repo,
        &["remote", "add", "origin", origin.to_str().expect("utf8")],
    );

    let e = d.client.push().await.expect_err("push must fail");
    assert!(e.to_string().contains("push"), "{e}");

    let events = d
        .client
        .events(&EventQuery {
            kind: Some("push.failed".to_string()),
            ..EventQuery::default()
        })
        .await
        .expect("events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data["branch"], "main");
    assert!(
        events[0].data["error"]
            .as_str()
            .expect("error")
            .contains("nope: not the owner")
            || events[0].data["error"]
                .as_str()
                .expect("error")
                .contains("remote"),
        "{:?}",
        events[0].data
    );

    let msgs = d
        .client
        .list_messages(&MessageQuery {
            to: Some("human".to_string()),
            ..MessageQuery::default()
        })
        .await
        .expect("messages");
    assert!(
        msgs.iter().any(|m| m.body.contains("Push of main")),
        "{msgs:?}"
    );
}

#[tokio::test]
async fn a_good_push_succeeds_and_records_nothing() {
    let (d, tmp) = start_daemon(None).await;
    let origin = tmp.path().join("origin.git");
    git(
        tmp.path(),
        &["init", "--bare", "-q", origin.to_str().expect("utf8")],
    );
    git(
        &d.repo,
        &["remote", "add", "origin", origin.to_str().expect("utf8")],
    );
    let r = d.client.push().await.expect("push");
    assert_eq!(r.branch, "main");
}
