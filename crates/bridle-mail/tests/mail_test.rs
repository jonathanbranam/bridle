use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bridle_api::types::SendRequest;
use bridle_mail::{Bridge, FakeStore, FixedLocal, MailConfig, Outcome, Rejection, Sink, evaluate};

fn cfg() -> MailConfig {
    MailConfig {
        bucket: "b".into(),
        allow: vec!["me@example.com".into(), "@corp.example".into()],
        ..MailConfig::default()
    }
}

struct Spec<'a> {
    from: &'a str,
    to: &'a str,
    auth: &'a str,
    extra: &'a str,
    body: &'a str,
}

impl Default for Spec<'_> {
    fn default() -> Self {
        Spec {
            from: "me@example.com",
            to: "proj@dev.branam.us",
            auth: "amazonses.com; spf=pass; dkim=pass; dmarc=pass header.from=example.com",
            extra: "",
            body: "Please do the thing.\r\n",
        }
    }
}

impl Spec<'_> {
    fn raw(&self) -> Vec<u8> {
        format!(
            "Authentication-Results: {}\r\nX-SES-Spam-Verdict: PASS\r\nX-SES-Virus-Verdict: PASS\r\n\
             {}From: {}\r\nTo: {}\r\nSubject: Idea\r\nMessage-ID: <x@y>\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n{}",
            self.auth, self.extra, self.from, self.to, self.body
        )
        .into_bytes()
    }
}

fn eval(s: &Spec) -> Result<bridle_mail::Accepted, Rejection> {
    evaluate(&s.raw(), &cfg(), "proj")
}

#[test]
fn accepts_an_allowed_dmarc_passing_mail() {
    let m = eval(&Spec::default()).expect("accepted");
    assert_eq!(m.subject, "Idea");
    assert_eq!(m.text, "Please do the thing.");
    assert_eq!(m.route.task, None);
}

#[test]
fn rejects_without_dmarc_pass_or_for_another_domain() {
    for auth in [
        "amazonses.com; dmarc=fail header.from=example.com",
        "amazonses.com; spf=pass",
        "amazonses.com; dmarc=pass header.from=evil.example",
        "mx.evil.example; dmarc=pass header.from=example.com",
    ] {
        let r = eval(&Spec {
            auth,
            ..Spec::default()
        });
        assert!(matches!(r, Err(Rejection::Dmarc(_))), "{auth}: {r:?}");
    }
}

#[test]
fn a_forged_authentication_results_below_ses_is_not_trusted() {
    // SES's own (failing) header is on top; the sender's passing one is lower.
    let raw = String::from_utf8(
        Spec {
            auth: "amazonses.com; dmarc=fail header.from=example.com",
            extra: "Authentication-Results: amazonses.com; dmarc=pass header.from=example.com\r\n",
            ..Spec::default()
        }
        .raw(),
    )
    .expect("utf8");
    assert!(matches!(
        evaluate(raw.as_bytes(), &cfg(), "proj"),
        Err(Rejection::Dmarc(_))
    ));
}

#[test]
fn allowlist_takes_addresses_and_domains() {
    let corp = Spec {
        from: "boss@corp.example",
        auth: "amazonses.com; dmarc=pass header.from=corp.example",
        ..Spec::default()
    };
    assert!(eval(&corp).is_ok());
    let stranger = Spec {
        from: "x@other.example",
        auth: "amazonses.com; dmarc=pass header.from=other.example",
        ..Spec::default()
    };
    assert!(matches!(eval(&stranger), Err(Rejection::NotAllowed(_))));
    let sub = Spec {
        from: "x@sub.corp.example",
        auth: "amazonses.com; dmarc=pass header.from=sub.corp.example",
        ..Spec::default()
    };
    assert!(matches!(eval(&sub), Err(Rejection::NotAllowed(_))));
}

#[test]
fn rejects_bad_verdicts_and_auto_replies() {
    let raw = String::from_utf8(Spec::default().raw()).expect("utf8");
    let spam = raw.replace("Spam-Verdict: PASS", "Spam-Verdict: FAIL");
    assert_eq!(
        evaluate(spam.as_bytes(), &cfg(), "proj"),
        Err(Rejection::Verdict)
    );
    let virus = raw.replace("Virus-Verdict: PASS", "Virus-Verdict: FAIL");
    assert_eq!(
        evaluate(virus.as_bytes(), &cfg(), "proj"),
        Err(Rejection::Verdict)
    );
    for extra in [
        "Auto-Submitted: auto-replied\r\n",
        "Precedence: bulk\r\n",
        "Return-Path: <>\r\n",
    ] {
        let r = eval(&Spec {
            extra,
            ..Spec::default()
        });
        assert_eq!(r, Err(Rejection::AutoReply), "{extra}");
    }
    let ok = eval(&Spec {
        extra: "Auto-Submitted: no\r\n",
        ..Spec::default()
    });
    assert!(ok.is_ok());
}

#[test]
fn parses_project_and_task_addresses() {
    let m = eval(&Spec {
        to: "Proj+t-br-1166@dev.branam.us",
        ..Spec::default()
    })
    .expect("ok");
    assert_eq!(m.route.task.as_deref(), Some("br-1166"));
    for to in [
        "other@dev.branam.us",
        "proj@branam.us",
        "proj+x-1@dev.branam.us",
        "proj+t-@dev.branam.us",
        "proj+t-a/b@dev.branam.us",
    ] {
        assert_eq!(
            eval(&Spec {
                to,
                ..Spec::default()
            }),
            Err(Rejection::NotOurs),
            "{to}"
        );
    }
}

#[test]
fn strips_quoted_history_and_signature() {
    let body = "Do X first.\r\n\r\nThanks\r\n\r\n> old text\r\nOn Mon, Jan 1, 2026 at 9:00 AM Bob <b@x.y> wrote:\r\n> quoted\r\n";
    assert_eq!(
        eval(&Spec {
            body,
            ..Spec::default()
        })
        .expect("ok")
        .text,
        "Do X first.\n\nThanks"
    );
    let body = "Ideas here.\r\n-- \r\nJo\r\nCorp Inc\r\n";
    assert_eq!(
        eval(&Spec {
            body,
            ..Spec::default()
        })
        .expect("ok")
        .text,
        "Ideas here."
    );
    let body = "Reply.\r\n\r\nFrom: Bob\r\nSent: Monday\r\nTo: me\r\n\r\nold\r\n";
    assert_eq!(
        eval(&Spec {
            body,
            ..Spec::default()
        })
        .expect("ok")
        .text,
        "Reply."
    );
}

#[test]
fn caps_the_body() {
    let body = "a".repeat(30_000);
    let m = eval(&Spec {
        body: &body,
        ..Spec::default()
    })
    .expect("ok");
    assert!(m.text.contains("[truncated"));
    assert!(m.text.len() < 20_200);
}

#[test]
fn html_only_mail_is_converted_to_text() {
    let raw = String::from_utf8(Spec::default().raw())
        .expect("utf8")
        .replace("text/plain", "text/html")
        .replace(
            "Please do the thing.",
            "<p>Hello <b>there</b><script>alert(1)</script></p>",
        );
    let m = evaluate(raw.as_bytes(), &cfg(), "proj").expect("ok");
    assert!(
        m.text.contains("Hello") && !m.text.contains('<'),
        "{}",
        m.text
    );
}

fn with_attachments() -> Vec<u8> {
    let head = String::from_utf8(Spec::default().raw())
        .expect("utf8")
        .replace(
            "Content-Type: text/plain; charset=utf-8\r\n\r\nPlease do the thing.\r\n",
            "",
        );
    let big = "x".repeat(1_100_000);
    format!(
        "{head}Content-Type: multipart/mixed; boundary=B\r\n\r\n\
         --B\r\nContent-Type: text/plain\r\n\r\nSee attached.\r\n\
         --B\r\nContent-Type: text/markdown; name=\"../plan.md\"\r\nContent-Disposition: attachment; filename=\"../plan.md\"\r\n\r\n# Plan\r\n\
         --B\r\nContent-Type: application/pdf; name=\"a.pdf\"\r\nContent-Disposition: attachment; filename=\"a.pdf\"\r\n\r\n%PDF\r\n\
         --B\r\nContent-Type: text/plain; name=\"big.txt\"\r\nContent-Disposition: attachment; filename=\"big.txt\"\r\n\r\n{big}\r\n\
         --B--\r\n"
    )
    .into_bytes()
}

#[test]
fn keeps_small_text_attachments_and_notes_the_rest() {
    let m = evaluate(&with_attachments(), &cfg(), "proj").expect("ok");
    assert_eq!(m.text, "See attached.");
    assert_eq!(m.attachments.len(), 1);
    assert_eq!(m.attachments[0].name, "plan.md");
    assert!(m.attachments[0].data.starts_with(b"# Plan"));
    assert_eq!(m.dropped.len(), 2, "{:?}", m.dropped);
}

#[derive(Default, Clone)]
struct Recorder(Arc<Mutex<Vec<SendRequest>>>);

#[async_trait]
impl Sink for Recorder {
    async fn send(&self, req: SendRequest) -> anyhow::Result<Vec<String>> {
        self.0.lock().expect("lock").push(req);
        Ok(vec!["m-9000".to_string()])
    }
}

struct Down;

#[async_trait]
impl Sink for Down {
    async fn send(&self, _: SendRequest) -> anyhow::Result<Vec<String>> {
        anyhow::bail!("daemon down")
    }
}

fn bridge(
    store: &Arc<FakeStore>,
    sink: Arc<dyn Sink>,
    cfg: MailConfig,
    dir: &std::path::Path,
) -> Bridge {
    Bridge::new(store.clone(), sink, cfg, "proj".into(), dir.to_path_buf())
}

#[tokio::test]
async fn routes_to_the_orchestrator_or_the_advisor_and_deletes() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = Arc::new(FakeStore::default());
    store.put(
        "inbound/ses1",
        Spec {
            to: "proj+t-tk-1@dev.branam.us",
            ..Spec::default()
        }
        .raw(),
    );
    let rec = Recorder::default();
    let out = bridge(&store, Arc::new(rec.clone()), cfg(), dir.path())
        .poll_once()
        .await
        .expect("poll");
    assert_eq!(
        out[0].1,
        Outcome::Delivered {
            to: "external:orchestrator".into()
        }
    );
    assert!(store.keys().is_empty());
    {
        let sent = rec.0.lock().expect("lock");
        assert_eq!(sent[0].to.as_deref(), Some("external:orchestrator"));
        assert_eq!(sent[0].task.as_deref(), Some("tk-1"));
        assert!(
            sent[0]
                .body
                .starts_with("via email from me@example.com (SES message ses1)")
        );
        assert!(sent[0].body.contains("Please do the thing."));
    }

    store.put("inbound/ses2", Spec::default().raw());
    let out = bridge(&store, Arc::new(rec.clone()), cfg(), dir.path())
        .with_local(Arc::new(FixedLocal {
            owns: true,
            advisor: true,
        }))
        .poll_once()
        .await
        .expect("poll");
    assert_eq!(
        out[0].1,
        Outcome::Delivered {
            to: "external:advisor".into()
        }
    );
}

#[tokio::test]
async fn attachments_land_in_the_inbox_with_the_path_in_the_note() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = Arc::new(FakeStore::default());
    store.put("inbound/ses3", with_attachments());
    let rec = Recorder::default();
    bridge(&store, Arc::new(rec.clone()), cfg(), dir.path())
        .poll_once()
        .await
        .expect("poll");
    let path = dir.path().join("ses3/plan.md");
    assert!(
        std::fs::read_to_string(&path)
            .expect("saved")
            .starts_with("# Plan")
    );
    let body = rec.0.lock().expect("lock")[0].body.clone();
    assert!(body.contains(&path.display().to_string()));
    assert!(body.contains("a.pdf"), "{body}");
}

#[tokio::test]
async fn rejected_and_foreign_mail_is_not_delivered_and_a_down_daemon_keeps_the_object() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = Arc::new(FakeStore::default());
    store.put(
        "inbound/bad",
        Spec {
            from: "x@evil.example",
            auth: "amazonses.com; dmarc=pass header.from=evil.example",
            ..Spec::default()
        }
        .raw(),
    );
    store.put(
        "inbound/foreign",
        Spec {
            to: "other@dev.branam.us",
            ..Spec::default()
        }
        .raw(),
    );
    let rec = Recorder::default();
    let b = bridge(&store, Arc::new(rec.clone()), cfg(), dir.path());
    let out = b.poll_once().await.expect("poll");
    assert_eq!(out.len(), 2);
    assert!(rec.0.lock().expect("lock").is_empty());
    assert_eq!(store.keys().len(), 2, "nothing deleted");
    // The rejected one isn't fetched again; the foreign one is looked at each pass.
    assert_eq!(b.poll_once().await.expect("poll").len(), 1);

    store.put("inbound/good", Spec::default().raw());
    let down = bridge(&store, Arc::new(Down), cfg(), dir.path());
    assert!(
        down.poll_once()
            .await
            .expect("poll")
            .iter()
            .all(|(k, _)| k != "inbound/good")
    );
    assert!(store.keys().contains(&"inbound/good".to_string()));
}

#[test]
fn config_reads_the_mail_table() {
    let c = MailConfig::parse("[mail]\nbucket = \"b\"\nallow = [\"a@b.c\"]\nprojects = [\"x\"]\n")
        .expect("parse");
    assert!(c.answers_unknown("x") && c.allows("a@b.c") && !c.allows("z@b.c"));
    assert!(
        MailConfig::parse("[mail]\nbucket = \"b\"\n").is_err(),
        "empty allow"
    );
}
