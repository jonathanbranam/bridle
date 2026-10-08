//! Mail 3: mail follows the project's owner, waits and tells the sender, unknown names, the
//! advisor, and "got it" replies. Fakes only: no AWS, no daemon.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bridle_api::types::{Agent, Message, OpenQuestion, SendRequest, Task};
use bridle_mail::{
    Bridge, FakeMailer, FakeStore, Feed, FixedLocal, MailConfig, Outbound, Outcome, Sent, Sink,
    Tokens,
};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

fn cfg() -> MailConfig {
    MailConfig {
        bucket: "b".into(),
        allow: vec!["me@example.com".into()],
        projects: vec!["proj".into(), "other".into()],
        digest_at: "23:59".into(),
        ..MailConfig::default()
    }
}

fn raw(to: &str) -> Vec<u8> {
    format!(
        "Authentication-Results: amazonses.com; dmarc=pass header.from=example.com\r\n\
         X-SES-Spam-Verdict: PASS\r\nX-SES-Virus-Verdict: PASS\r\n\
         From: me@example.com\r\nTo: {to}@dev.branam.us\r\nSubject: Idea\r\n\
         Content-Type: text/plain\r\n\r\nDo the thing.\r\n"
    )
    .into_bytes()
}

#[derive(Default)]
struct Hub {
    sent: Mutex<Vec<SendRequest>>,
    inbox: Mutex<Vec<Message>>,
    read: Mutex<Vec<String>>,
    down: Mutex<bool>,
}

#[async_trait]
impl Sink for Hub {
    async fn send(&self, req: SendRequest) -> anyhow::Result<Vec<String>> {
        if *self.down.lock().expect("lock") {
            anyhow::bail!("daemon down");
        }
        self.sent.lock().expect("lock").push(req);
        Ok(vec!["m-0100".into()])
    }
    async fn inbox(&self) -> anyhow::Result<Vec<Message>> {
        Ok(self.inbox.lock().expect("lock").clone())
    }
    async fn mark_read(&self, id: &str) -> anyhow::Result<()> {
        self.read.lock().expect("lock").push(id.to_string());
        Ok(())
    }
}

struct NoFeed;

#[async_trait]
impl Feed for NoFeed {
    async fn human_messages(&self) -> anyhow::Result<Vec<Message>> {
        Ok(Vec::new())
    }
    async fn agents(&self) -> anyhow::Result<Vec<Agent>> {
        Ok(Vec::new())
    }
    async fn tasks(&self) -> anyhow::Result<Vec<Task>> {
        Ok(Vec::new())
    }
    async fn open_questions(&self) -> anyhow::Result<Vec<OpenQuestion>> {
        Ok(Vec::new())
    }
}

struct Rig {
    bridge: Bridge,
    store: Arc<FakeStore>,
    hub: Arc<Hub>,
    mailer: Arc<FakeMailer>,
    _dir: tempfile::TempDir,
}

fn rig(project: &str, owns: bool, aide: bool) -> Rig {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(FakeStore::default());
    let hub = Arc::new(Hub::default());
    let mailer = Arc::new(FakeMailer::default());
    let bridge = Bridge::new(
        store.clone(),
        hub.clone(),
        cfg(),
        project.into(),
        dir.path().join("attachments"),
    )
    .with_local(Arc::new(FixedLocal { owns, aide }))
    .with_outbound(Outbound {
        mailer: mailer.clone(),
        feed: Arc::new(NoFeed),
        tokens: Tokens::new(b"a machine key of some length".to_vec()),
        sent: Sent::open(&dir.path().join("mail")).expect("sent"),
    });
    Rig {
        bridge,
        store,
        hub,
        mailer,
        _dir: dir,
    }
}

fn t0() -> DateTime<Utc> {
    "2026-09-30T10:00:00Z".parse().expect("time")
}

#[tokio::test]
async fn a_bridge_that_does_not_own_the_project_leaves_the_mail() {
    let r = rig("proj", false, false);
    r.store.put("inbound/a", raw("proj"));
    let out = r.bridge.poll_once_at(t0()).await.expect("poll");
    assert_eq!(out[0].1, Outcome::Waiting);
    assert_eq!(r.store.keys(), ["inbound/a"]);
    assert!(r.hub.sent.lock().expect("lock").is_empty());
    assert!(
        r.mailer.sent().is_empty(),
        "nothing yet: it may just be a move"
    );
}

#[tokio::test]
async fn waiting_mail_tells_the_sender_once_after_the_limit() {
    let r = rig("proj", false, false);
    r.store.put("inbound/a", raw("proj"));
    r.bridge.poll_once_at(t0()).await.expect("poll");
    r.bridge
        .poll_once_at(t0() + Duration::minutes(59))
        .await
        .expect("poll");
    assert!(r.mailer.sent().is_empty());
    for extra in [61, 90] {
        r.bridge
            .poll_once_at(t0() + Duration::minutes(extra))
            .await
            .expect("poll");
    }
    let sent = r.mailer.sent();
    assert_eq!(sent.len(), 1, "once");
    assert_eq!(sent[0].to, "me@example.com");
    assert_eq!(sent[0].from, "proj@dev.branam.us");
    assert!(
        sent[0]
            .body
            .starts_with("not delivered yet: no machine is running proj")
    );
    assert_eq!(r.store.keys(), ["inbound/a"], "still there for the owner");
}

#[tokio::test]
async fn a_down_daemon_on_the_owner_counts_as_waiting_too() {
    let r = rig("proj", true, false);
    *r.hub.down.lock().expect("lock") = true;
    r.store.put("inbound/a", raw("proj"));
    assert!(r.bridge.poll_once_at(t0()).await.expect("poll").is_empty());
    r.bridge
        .poll_once_at(t0() + Duration::hours(2))
        .await
        .expect("poll");
    assert_eq!(r.mailer.sent().len(), 1);
    assert_eq!(r.store.keys(), ["inbound/a"]);
}

#[tokio::test]
async fn an_unknown_project_gets_the_valid_names_from_the_first_projects_bridge() {
    let r = rig("proj", true, false);
    r.store.put("inbound/a", raw("nosuch"));
    let out = r.bridge.poll_once_at(t0()).await.expect("poll");
    assert_eq!(out[0].1, Outcome::UnknownProject("nosuch".into()));
    let sent = r.mailer.sent();
    assert_eq!(sent[0].to, "me@example.com");
    assert!(sent[0].body.contains("no project named nosuch"));
    assert!(sent[0].body.contains("proj, other"));
    assert!(r.store.keys().is_empty());

    // Another project's bridge leaves it for that one.
    let o = rig("other", true, false);
    o.store.put("inbound/a", raw("nosuch"));
    let out = o.bridge.poll_once_at(t0()).await.expect("poll");
    assert_eq!(out[0].1, Outcome::Skipped);
    assert!(o.mailer.sent().is_empty());
}

#[tokio::test]
async fn a_stranger_writing_to_an_unknown_project_gets_no_reply() {
    let r = rig("proj", true, false);
    let stranger = String::from_utf8(raw("nosuch"))
        .expect("utf8")
        .replace("me@example.com", "evil@example.com");
    r.store.put("inbound/a", stranger);
    r.bridge.poll_once_at(t0()).await.expect("poll");
    assert!(r.mailer.sent().is_empty());
}

#[tokio::test]
async fn mail_goes_to_the_aide_only_while_it_runs() {
    for (aide, to) in [(true, "external:aide"), (false, "external:orchestrator")] {
        let r = rig("proj", true, aide);
        r.store.put("inbound/a", raw("proj"));
        r.bridge.poll_once_at(t0()).await.expect("poll");
        assert_eq!(r.hub.sent.lock().expect("lock")[0].to.as_deref(), Some(to));
    }
}

#[tokio::test]
async fn got_it_goes_back_to_the_sender() {
    let r = rig("proj", true, true);
    r.store.put("inbound/a", raw("proj"));
    r.bridge.poll_once_at(t0()).await.expect("poll");
    let reply: Message = serde_json::from_value(json!({
        "id": "m-0101", "from": "external:aide", "to": "external:mail", "kind": "note",
        "body": "got it: filed ticket ab12", "reply_to": "m-0100", "when": "idle",
        "state": "delivered", "created_at": "2026-09-30T10:05:00Z", "written_at": null,
        "delivered_at": null, "read_at": null,
    }))
    .expect("message");
    let unrelated = Message {
        id: "m-0102".into(),
        reply_to: Some("m-0001".into()),
        ..reply.clone()
    };
    *r.hub.inbox.lock().expect("lock") = vec![reply, unrelated];
    r.bridge
        .poll_outbound(t0().naive_local())
        .await
        .expect("outbound");
    let sent = r.mailer.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "me@example.com");
    assert_eq!(sent[0].subject, "Re: Idea");
    assert_eq!(sent[0].body, "got it: filed ticket ab12");
    assert_eq!(*r.hub.read.lock().expect("lock"), ["m-0101"]);
}
