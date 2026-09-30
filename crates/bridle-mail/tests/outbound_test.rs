use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bridle_api::types::{Agent, Message, MessageKind, OpenQuestion, SendRequest, Task};
use bridle_mail::{
    Bridge, FakeMailer, FakeStore, Feed, MailConfig, Outbound, Outcome, Rejection, Sent, Sink,
    Tokens, digest_due,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::json;

fn cfg() -> MailConfig {
    MailConfig {
        bucket: "b".into(),
        allow: vec!["me@example.com".into()],
        ..MailConfig::default()
    }
}

fn message(id: &str, kind: &str, from: &str, body: &str) -> Message {
    serde_json::from_value(json!({
        "id": id, "from": from, "to": "human", "kind": kind, "body": body,
        "reply_to": null, "when": "now", "state": "delivered",
        "created_at": "2026-09-30T10:00:00Z", "written_at": null,
        "delivered_at": null, "read_at": null,
    }))
    .expect("message")
}

fn task(id: &str, state: &str, claimed_by: Option<&str>) -> Task {
    serde_json::from_value(json!({
        "id": id, "title": format!("title of {id}"), "kind": "feature", "state": state,
        "body": "", "thread": [], "created_at": "2026-09-30T10:00:00Z",
        "updated_at": "2026-09-30T10:00:00Z", "claimed_by": claimed_by,
    }))
    .expect("task")
}

#[derive(Default)]
struct FakeFeed {
    messages: Mutex<Vec<Message>>,
    tasks: Mutex<Vec<Task>>,
}

#[async_trait]
impl Feed for FakeFeed {
    async fn human_messages(&self) -> anyhow::Result<Vec<Message>> {
        Ok(self.messages.lock().expect("lock").clone())
    }
    async fn agents(&self) -> anyhow::Result<Vec<Agent>> {
        Ok(Vec::new())
    }
    async fn tasks(&self) -> anyhow::Result<Vec<Task>> {
        Ok(self.tasks.lock().expect("lock").clone())
    }
    async fn open_questions(&self) -> anyhow::Result<Vec<OpenQuestion>> {
        Ok(Vec::new())
    }
}

#[derive(Default)]
struct Recorder(Mutex<Vec<SendRequest>>);

#[async_trait]
impl Sink for Recorder {
    async fn send(&self, req: SendRequest) -> anyhow::Result<()> {
        self.0.lock().expect("lock").push(req);
        Ok(())
    }
}

struct Rig {
    bridge: Bridge,
    store: Arc<FakeStore>,
    sink: Arc<Recorder>,
    mailer: Arc<FakeMailer>,
    feed: Arc<FakeFeed>,
    tokens: Tokens,
    _dir: tempfile::TempDir,
}

fn rig() -> Rig {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(FakeStore::default());
    let sink = Arc::new(Recorder::default());
    let mailer = Arc::new(FakeMailer::default());
    let feed = Arc::new(FakeFeed::default());
    let tokens = Tokens::new(b"a machine key of some length".to_vec());
    let bridge = Bridge::new(
        store.clone(),
        sink.clone(),
        cfg(),
        "proj".into(),
        dir.path().join("attachments"),
    )
    .with_outbound(Outbound {
        mailer: mailer.clone(),
        feed: feed.clone(),
        tokens: tokens.clone(),
        sent: Sent::open(&dir.path().join("mail")).expect("sent"),
    });
    Rig {
        bridge,
        store,
        sink,
        mailer,
        feed,
        tokens,
        _dir: dir,
    }
}

fn at(h: u32, m: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 9, 30)
        .expect("date")
        .and_hms_opt(h, m, 0)
        .expect("time")
}

fn reply_raw(to_local: &str) -> Vec<u8> {
    format!(
        "Authentication-Results: amazonses.com; dmarc=pass header.from=example.com\r\n\
         X-SES-Spam-Verdict: PASS\r\nX-SES-Virus-Verdict: PASS\r\n\
         From: me@example.com\r\nTo: {to_local}@dev.branam.us\r\nSubject: Re: q\r\n\
         Content-Type: text/plain\r\n\r\nYes, go ahead.\r\n"
    )
    .into_bytes()
}

#[tokio::test]
async fn a_question_is_mailed_once_with_a_reply_token() {
    let r = rig();
    r.feed.messages.lock().expect("lock").extend([
        message("m-0001", "question", "a-1", "Ship it?\nMore detail."),
        message("m-0002", "note", "a-1", "FYI, a note"),
        message("m-0003", "answer", "a-1", "an answer"),
    ]);
    r.bridge.poll_outbound(at(6, 0)).await.expect("poll");
    r.bridge.poll_outbound(at(6, 1)).await.expect("poll");
    let sent = r.mailer.sent();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].subject, "[bridle/proj] a-1: Ship it?");
    assert_eq!(sent[0].to, "dev@branam.us");
    let reply = sent[0].reply_to.clone().expect("reply-to");
    let local = reply.strip_suffix("@dev.branam.us").expect("domain");
    let (head, tag) = local.rsplit_once('.').expect("tag");
    assert_eq!(head, "proj+r-m-0001");
    assert!(r.tokens.verify("proj", "m-0001", tag));
    assert!(!r.tokens.verify("proj", "m-0002", tag));
}

#[tokio::test]
async fn a_valid_reply_becomes_an_answer_to_the_asker() {
    let r = rig();
    r.feed
        .messages
        .lock()
        .expect("lock")
        .push(message("m-0001", "question", "a-1", "Ship it?"));
    let local = r.tokens.reply_local("proj", "m-0001");
    r.store.put("inbound/ses1", reply_raw(&local));
    let done = r.bridge.poll_once().await.expect("poll");
    assert_eq!(
        done[0].1,
        Outcome::Delivered {
            to: "a-1".to_string()
        }
    );
    let sent = r.sink.0.lock().expect("lock");
    assert_eq!(sent[0].kind, MessageKind::Answer);
    assert_eq!(sent[0].reply_to.as_deref(), Some("m-0001"));
    assert!(
        sent[0]
            .body
            .contains("via email from me@example.com (SES message ses1)")
    );
    assert!(sent[0].body.contains("Yes, go ahead."));
    assert!(r.store.keys().is_empty(), "deleted once accepted");
}

#[tokio::test]
async fn tampered_and_replayed_tokens_are_rejected() {
    let r = rig();
    r.feed
        .messages
        .lock()
        .expect("lock")
        .push(message("m-0001", "question", "a-1", "Ship it?"));
    let good = r.tokens.reply_local("proj", "m-0001");
    // The tag of another question, a flipped tag, and no tag at all.
    let other = r.tokens.reply_local("proj", "m-0009");
    let tag = other.rsplit_once('.').expect("tag").1;
    let swapped = format!("proj+r-m-0001.{tag}");
    let flipped = format!("{}0", &good[..good.len() - 1]);
    for (i, local) in [swapped, flipped, "proj+r-m-0001.".to_string()]
        .iter()
        .enumerate()
    {
        r.store.put(&format!("inbound/bad{i}"), reply_raw(local));
    }
    let done = r.bridge.poll_once().await.expect("poll");
    // The empty-tag address isn't a reply route at all.
    assert!(done.iter().any(|(_, o)| *o == Outcome::Skipped));
    assert_eq!(
        done.iter()
            .filter(|(_, o)| *o == Outcome::Rejected(Rejection::BadToken))
            .count(),
        2
    );
    assert!(r.sink.0.lock().expect("lock").is_empty());

    // A valid token for a question that has since been answered is a replay.
    let mut answered = message("m-0001", "question", "a-1", "Ship it?");
    answered.answered_by = Some("external:mail".into());
    r.feed.messages.lock().expect("lock")[0] = answered;
    r.store.put("inbound/replay", reply_raw(&good));
    let done = r.bridge.poll_once().await.expect("poll");
    let replay = done
        .iter()
        .find(|(k, _)| k == "inbound/replay")
        .expect("replay");
    assert_eq!(
        replay.1,
        Outcome::Rejected(Rejection::Replayed("m-0001".into()))
    );
    assert!(r.sink.0.lock().expect("lock").is_empty());
}

#[tokio::test]
async fn the_digest_goes_once_a_day_at_its_time() {
    let r = rig();
    {
        let mut m = r.feed.messages.lock().expect("lock");
        m.push(message("m-0001", "question", "a-1", "Ship it?"));
        m.push(message("m-0002", "note", "a-2", "Blocked on credentials"));
    }
    r.feed.tasks.lock().expect("lock").extend([
        task("t-1", "planned", None),
        task("t-2", "claimed", Some("a-7")),
        task("t-3", "integrated", None),
    ]);
    r.bridge.poll_outbound(at(6, 29)).await.expect("poll");
    assert_eq!(r.mailer.sent().len(), 1, "only the question before 6:30");
    r.bridge.poll_outbound(at(6, 30)).await.expect("poll");
    r.bridge.poll_outbound(at(18, 0)).await.expect("poll");
    let sent = r.mailer.sent();
    assert_eq!(sent.len(), 2, "one digest for the day");
    let digest = &sent[1];
    assert_eq!(digest.subject, "[bridle/proj] digest 2026-09-30");
    assert_eq!(digest.to, "dev@branam.us");
    assert!(digest.reply_to.is_none());
    for want in [
        "m-0001",
        "Blocked on credentials",
        "planned 1",
        "claimed 1",
        "integrated 1",
        "t-2 in progress (a-7)",
    ] {
        assert!(digest.body.contains(want), "{want} in {}", digest.body);
    }
}

#[test]
fn digest_schedule() {
    let six_thirty = NaiveTime::from_hms_opt(6, 30, 0).expect("time");
    let today = at(0, 0).date();
    let yesterday = today.pred_opt();
    assert!(!digest_due(at(6, 29), six_thirty, yesterday));
    assert!(digest_due(at(6, 30), six_thirty, yesterday));
    assert!(digest_due(at(6, 30), six_thirty, None));
    assert!(!digest_due(at(23, 0), six_thirty, Some(today)));
}

#[test]
fn keys_persist_and_differ_per_machine() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("mail.key");
    let a = Tokens::load_or_create(&path).expect("create");
    let again = Tokens::load_or_create(&path).expect("reload");
    assert_eq!(a.tag("p", "m-1"), again.tag("p", "m-1"));
    let other = Tokens::new(b"another machine's key".to_vec());
    assert_ne!(a.tag("p", "m-1"), other.tag("p", "m-1"));
}
