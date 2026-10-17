use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bridle_api::types::{Agent, Message, MessageKind, OpenQuestion, SendRequest, Ticket};
use bridle_mail::{
    Bridge, FakeMailer, FakeStore, Feed, MailConfig, Outbound, Outcome, Rejection, Sent, Sink,
    Tokens, decode_tickets, digest_due, digest_mail, is_bare_sent_notice,
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

fn task(id: &str, state: &str, claimed_by: Option<&str>) -> Ticket {
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
    tasks: Mutex<Vec<Ticket>>,
    tasks_fail: std::sync::atomic::AtomicBool,
}

#[async_trait]
impl Feed for FakeFeed {
    async fn human_messages(&self) -> anyhow::Result<Vec<Message>> {
        Ok(self.messages.lock().expect("lock").clone())
    }
    async fn agents(&self) -> anyhow::Result<Vec<Agent>> {
        Ok(Vec::new())
    }
    async fn tasks(&self) -> anyhow::Result<Vec<Ticket>> {
        if self.tasks_fail.load(std::sync::atomic::Ordering::SeqCst) {
            anyhow::bail!("unknown variant `newkind`");
        }
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
    async fn send(&self, req: SendRequest) -> anyhow::Result<Vec<String>> {
        self.0.lock().expect("lock").push(req);
        Ok(vec!["m-9000".to_string()])
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

#[tokio::test]
async fn a_task_list_that_fails_to_decode_still_gets_the_digest() {
    let r = rig();
    r.feed
        .tasks_fail
        .store(true, std::sync::atomic::Ordering::SeqCst);
    r.bridge.poll_outbound(at(6, 30)).await.expect("poll");
    let sent = r.mailer.sent();
    assert_eq!(sent.len(), 1, "the digest went anyway");
    assert!(sent[0].subject.contains("digest"));
}

#[test]
fn one_unknown_kind_or_state_skips_only_that_ticket() {
    let ok = |id: &str| {
        json!({
            "id": id, "title": "t", "kind": "feature", "state": "planned",
            "body": "", "thread": [], "created_at": "2026-09-30T10:00:00Z",
            "updated_at": "2026-09-30T10:00:00Z",
        })
    };
    let mut new_kind = ok("t-2");
    new_kind["kind"] = json!("brand-new-kind");
    let mut new_state = ok("t-3");
    new_state["state"] = json!("brand_new_state");
    let got = decode_tickets(vec![ok("t-1"), new_kind, new_state, ok("t-4")]);
    let ids: Vec<_> = got.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["t-1", "t-4"]);
}

struct DownMailer;

#[async_trait]
impl bridle_mail::Mailer for DownMailer {
    async fn send(&self, _: bridle_mail::OutMail) -> anyhow::Result<()> {
        anyhow::bail!("ses refused")
    }
}

#[tokio::test]
async fn a_digest_that_cannot_be_sent_is_told_to_the_human_once_a_day() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sink = Arc::new(Recorder::default());
    let bridge = Bridge::new(
        Arc::new(FakeStore::default()),
        sink.clone(),
        cfg(),
        "proj".into(),
        dir.path().join("attachments"),
    )
    .with_outbound(Outbound {
        mailer: Arc::new(DownMailer),
        feed: Arc::new(FakeFeed::default()),
        tokens: Tokens::new(b"a machine key of some length".to_vec()),
        sent: Sent::open(&dir.path().join("mail")).expect("sent"),
    });
    assert!(bridge.poll_outbound(at(6, 30)).await.is_err());
    assert!(bridge.poll_outbound(at(6, 31)).await.is_err());
    let told = sink.0.lock().expect("lock").clone();
    assert_eq!(told.len(), 1, "once for the day");
    assert_eq!(told[0].to.as_deref(), Some("human"));
    assert!(told[0].body.contains("ses refused"), "{}", told[0].body);
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

fn message_at(id: &str, kind: &str, body: &str, created: &str, read: Option<&str>) -> Message {
    let mut m = message(id, kind, "a-1", body);
    m.created_at = created.parse().expect("created");
    m.read_at = read.map(|r| r.parse().expect("read"));
    m
}

#[test]
fn the_digest_stamps_local_time_and_groups_by_age() {
    // Eastern (UTC-4) on Friday 2026-10-16, 6:30 AM.
    let tz = chrono::FixedOffset::west_opt(4 * 3600).expect("tz");
    let now = NaiveDate::from_ymd_opt(2026, 10, 16)
        .expect("date")
        .and_hms_opt(6, 30, 0)
        .expect("time");
    let messages = [
        // 11:00 UTC is 7:00 AM Eastern; created yesterday, updated later the same day.
        message_at(
            "m-1",
            "note",
            "Yesterday's blocker",
            "2026-10-15T11:00:00Z",
            Some("2026-10-15T13:15:00Z"),
        ),
        message_at("m-2", "note", "Last week's", "2026-10-10T20:00:00Z", None),
        message_at("m-3", "note", "Ancient", "2026-09-01T20:00:00Z", None),
        // 02:00 UTC on the 16th is still the 15th at 10:00 PM Eastern: yesterday.
        message_at("m-4", "note", "Late night", "2026-10-16T02:00:00Z", None),
    ];
    let mail = digest_mail("proj", &cfg(), &messages, &[], &[], now, &tz);
    let body = &mail.body;
    assert!(
        body.contains("m-1 from a-1: Yesterday's blocker [created Thu Oct 15 7:00 AM, updated Thu Oct 15 9:15 AM]"),
        "{body}"
    );
    assert!(body.contains("[created Thu Oct 15 10:00 PM]"), "{body}");
    let pos = |s: &str| body.find(s).unwrap_or_else(|| panic!("{s} in {body}"));
    assert!(pos("  Yesterday\n") < pos("m-1"));
    assert!(pos("m-1") < pos("  Last week\n"));
    assert!(pos("  Last week\n") < pos("m-2"));
    assert!(pos("m-2") < pos("  Older\n"));
    assert!(pos("  Older\n") < pos("m-3"));
    assert!(
        !body.contains("  Today\n"),
        "empty groups are left out: {body}"
    );
}

#[test]
fn bare_message_sent_items_are_dropped_unless_they_link_somewhere() {
    let tz = chrono::FixedOffset::west_opt(4 * 3600).expect("tz");
    let now = at(6, 30);
    let bare = message("m-1", "note", "a-1", "Message sent");
    let mut replying = message("m-2", "note", "a-1", "message sent.");
    replying.reply_to = Some("m-0".into());
    let real = message(
        "m-3",
        "note",
        "a-1",
        "Message sent to the build farm failed",
    );
    assert!(is_bare_sent_notice(&bare));
    assert!(!is_bare_sent_notice(&replying));
    assert!(!is_bare_sent_notice(&real));
    let mail = digest_mail("proj", &cfg(), &[bare, replying, real], &[], &[], now, &tz);
    assert!(!mail.body.contains("m-1 from"), "{}", mail.body);
    assert!(mail.body.contains("m-2 from") && mail.body.contains("m-3 from"));
    assert!(mail.body.contains("Blockers and decisions waiting (2)"));
}

#[test]
fn merged_tickets_from_the_last_week_are_listed() {
    let tz = chrono::FixedOffset::west_opt(4 * 3600).expect("tz");
    let now = NaiveDate::from_ymd_opt(2026, 10, 1)
        .expect("date")
        .and_hms_opt(6, 30, 0)
        .expect("time");
    // task() was last updated 2026-09-30, yesterday.
    let mail = digest_mail(
        "proj",
        &cfg(),
        &[],
        &[],
        &[task("t-3", "integrated", None)],
        now,
        &tz,
    );
    assert!(
        mail.body.contains("Merged in the last week (1)"),
        "{}",
        mail.body
    );
    assert!(mail.body.contains("t-3 merged: title of t-3"));
}

#[test]
fn digest_stamps_real_us_eastern_whatever_the_host_zone() {
    // A UTC host: the digest is handed America/New_York, so 11:00 UTC is 7:00 AM in October
    // (EDT) and 12:00 UTC is 7:00 AM in January (EST).
    let tz = chrono_tz::America::New_York;
    let now = NaiveDate::from_ymd_opt(2026, 10, 16)
        .expect("date")
        .and_hms_opt(6, 30, 0)
        .expect("time");
    let messages = [
        message_at("m-1", "note", "Summer", "2026-10-15T11:00:00Z", None),
        message_at("m-2", "note", "Winter", "2026-01-15T12:00:00Z", None),
    ];
    let mail = digest_mail("proj", &cfg(), &messages, &[], &[], now, &tz);
    assert!(mail.body.contains("Times are US Eastern."), "{}", mail.body);
    assert!(
        mail.body.contains("[created Thu Oct 15 7:00 AM]"),
        "{}",
        mail.body
    );
    assert!(
        mail.body.contains("[created Thu Jan 15 7:00 AM]"),
        "{}",
        mail.body
    );
}
