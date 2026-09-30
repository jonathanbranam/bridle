//! The outbound half: question mails and the daily digest. Pure content and scheduling here;
//! the bridge does the I/O through [`Mailer`] and [`Feed`], so tests need neither SES nor a daemon.
//!
//! Only questions are mailed as they arrive; everything else the human's inbox holds waits for
//! the digest. Mail is not confidential, so bodies are the question text and counts and titles
//! only: never tokens, environment or diffs.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Context;
use async_trait::async_trait;
use bridle_api::types::{Agent, Message, MessageKind, MessageState, OpenQuestion, Task};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::config::MailConfig;
use crate::token::Tokens;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutMail {
    pub from: String,
    pub to: String,
    pub reply_to: Option<String>,
    pub subject: String,
    pub body: String,
}

/// Sends mail. The real one is SES; tests use [`FakeMailer`].
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, mail: OutMail) -> anyhow::Result<()>;
}

/// What the outbound half reads from the daemon.
#[async_trait]
pub trait Feed: Send + Sync {
    /// Every message addressed to the human, read or not.
    async fn human_messages(&self) -> anyhow::Result<Vec<Message>>;
    async fn agents(&self) -> anyhow::Result<Vec<Agent>>;
    async fn tasks(&self) -> anyhow::Result<Vec<Task>>;
    async fn open_questions(&self) -> anyhow::Result<Vec<OpenQuestion>>;
}

pub struct SesMailer {
    client: aws_sdk_sesv2::Client,
}

impl SesMailer {
    pub async fn connect(region: Option<&str>) -> Self {
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(r) = region {
            loader = loader.region(aws_config::Region::new(r.to_string()));
        }
        SesMailer {
            client: aws_sdk_sesv2::Client::new(&loader.load().await),
        }
    }
}

#[async_trait]
impl Mailer for SesMailer {
    async fn send(&self, mail: OutMail) -> anyhow::Result<()> {
        use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
        let text = |s: &str| Content::builder().data(s).charset("UTF-8").build();
        let message = Message::builder()
            .subject(text(&mail.subject)?)
            .body(Body::builder().text(text(&mail.body)?).build())
            .build();
        let mut req = self
            .client
            .send_email()
            .from_email_address(&mail.from)
            .destination(Destination::builder().to_addresses(&mail.to).build())
            .content(EmailContent::builder().simple(message).build());
        if let Some(r) = &mail.reply_to {
            req = req.reply_to_addresses(r);
        }
        req.send().await.context("SES SendEmail")?;
        Ok(())
    }
}

/// Records what would have been sent.
#[derive(Default)]
pub struct FakeMailer {
    sent: Mutex<Vec<OutMail>>,
}

impl FakeMailer {
    pub fn sent(&self) -> Vec<OutMail> {
        self.sent.lock().expect("fake mailer lock").clone()
    }
}

#[async_trait]
impl Mailer for FakeMailer {
    async fn send(&self, mail: OutMail) -> anyhow::Result<()> {
        self.sent.lock().expect("fake mailer lock").push(mail);
        Ok(())
    }
}

/// What has been mailed, kept on disk so a restart doesn't repeat it: the ids of mailed
/// questions (one per line) and the date of the last digest.
pub struct Sent {
    dir: PathBuf,
    questions: Mutex<HashSet<String>>,
}

impl Sent {
    pub fn open(dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        let questions = std::fs::read_to_string(dir.join("questions"))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect();
        Ok(Sent {
            dir: dir.to_path_buf(),
            questions: Mutex::new(questions),
        })
    }

    pub fn question_mailed(&self, id: &str) -> bool {
        self.questions.lock().expect("sent lock").contains(id)
    }

    pub fn mark_question(&self, id: &str) -> anyhow::Result<()> {
        use std::io::Write;
        self.questions
            .lock()
            .expect("sent lock")
            .insert(id.to_string());
        let path = self.dir.join("questions");
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(f, "{id}").with_context(|| format!("writing {}", path.display()))
    }

    /// Whether the sender was already told this mail is waiting.
    pub fn notified(&self, key: &str) -> bool {
        self.lines("notified").iter().any(|l| l == key)
    }

    pub fn mark_notified(&self, key: &str) -> anyhow::Result<()> {
        self.append("notified", key)
    }

    /// Remembers who wrote the mail delivered as message `id`, so the recipient's "got it"
    /// reply (a message with `reply_to: id`) can go back to them.
    pub fn remember_sender(&self, id: &str, from: &str, subject: &str) -> anyhow::Result<()> {
        self.append(
            "senders",
            &format!("{id}\t{from}\t{}", subject.replace(['\t', '\n', '\r'], " ")),
        )
    }

    /// `(from, subject)` of the mail delivered as message `id`.
    pub fn sender_of(&self, id: &str) -> Option<(String, String)> {
        self.lines("senders").into_iter().find_map(|l| {
            let mut f = l.splitn(3, '\t');
            (f.next()? == id).then(|| Some((f.next()?.to_string(), f.next()?.to_string())))?
        })
    }

    fn lines(&self, name: &str) -> Vec<String> {
        std::fs::read_to_string(self.dir.join(name))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn append(&self, name: &str, line: &str) -> anyhow::Result<()> {
        use std::io::Write;
        let path = self.dir.join(name);
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(f, "{line}").with_context(|| format!("writing {}", path.display()))
    }

    pub fn last_digest(&self) -> Option<NaiveDate> {
        std::fs::read_to_string(self.dir.join("digest"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    pub fn mark_digest(&self, day: NaiveDate) -> anyhow::Result<()> {
        std::fs::write(self.dir.join("digest"), day.to_string()).context("writing the digest date")
    }
}

/// Whether today's digest is owed: it's past the time and none went today. A bridge started
/// after 6:30 sends one straight away; that's better than skipping a day.
pub fn digest_due(now: NaiveDateTime, at: NaiveTime, last: Option<NaiveDate>) -> bool {
    now.time() >= at && last != Some(now.date())
}

fn first_line(body: &str) -> &str {
    body.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    format!("{cut}…")
}

/// A question that is still waiting for an answer.
pub fn is_open_question(m: &Message) -> bool {
    m.kind == MessageKind::Question
        && m.to == "human"
        && m.state != MessageState::Read
        && m.answered_by.is_none()
}

pub fn question_mail(
    q: &Message,
    agent_name: &str,
    project: &str,
    cfg: &MailConfig,
    tokens: &Tokens,
) -> OutMail {
    let body = format!(
        "{}\n\n--\nReply to this mail to answer. Your reply is recorded as you, via email.\n\
         Question {} from {agent_name} in {project}.\n",
        clip(&q.body, cfg.max_body_chars),
        q.id
    );
    OutMail {
        from: format!("{project}@{}", cfg.domain),
        to: cfg.to.clone(),
        reply_to: Some(format!(
            "{}@{}",
            tokens.reply_local(project, &q.id),
            cfg.domain
        )),
        subject: format!(
            "[bridle/{project}] {agent_name}: {}",
            clip(first_line(&q.body), 80)
        ),
        body,
    }
}

pub fn digest_mail(
    project: &str,
    cfg: &MailConfig,
    messages: &[Message],
    open: &[OpenQuestion],
    tasks: &[Task],
    now: NaiveDateTime,
) -> OutMail {
    let mut out = format!(
        "bridle digest for {project}, {}\n",
        now.format("%A %Y-%m-%d")
    );
    let questions: Vec<_> = messages.iter().filter(|m| is_open_question(m)).collect();
    out.push_str(&format!(
        "\nOpen questions for you ({}):\n",
        questions.len()
    ));
    for q in &questions {
        out.push_str(&format!(
            "- {} from {}: {}\n",
            q.id,
            q.from,
            clip(first_line(&q.body), 120)
        ));
    }
    if !open.is_empty() {
        out.push_str(&format!("\nOpen questions on tasks ({}):\n", open.len()));
        for q in open {
            out.push_str(&format!(
                "- {} ({}): {}\n",
                q.task_id,
                q.asked_by,
                clip(first_line(&q.body), 120)
            ));
        }
    }
    // The inbox is questions, blockers and decisions; whatever isn't a question is one of the
    // other two.
    let others: Vec<_> = messages
        .iter()
        .filter(|m| m.kind != MessageKind::Question && m.state != MessageState::Read)
        .collect();
    out.push_str(&format!(
        "\nBlockers and decisions waiting ({}):\n",
        others.len()
    ));
    for m in &others {
        out.push_str(&format!(
            "- {} from {}: {}\n",
            m.id,
            m.from,
            clip(first_line(&m.body), 120)
        ));
    }
    let mut by_state: BTreeMap<String, usize> = BTreeMap::new();
    for t in tasks {
        *by_state
            .entry(format!("{:?}", t.state).to_lowercase())
            .or_default() += 1;
    }
    out.push_str("\nTasks:");
    if by_state.is_empty() {
        out.push_str(" none");
    }
    for (state, n) in &by_state {
        out.push_str(&format!(" {state} {n};"));
    }
    out.push('\n');
    for t in tasks.iter().filter(|t| t.claimed_by.is_some()) {
        out.push_str(&format!(
            "- {} in progress ({}): {}\n",
            t.id,
            t.claimed_by.as_deref().unwrap_or(""),
            clip(&t.title, 100)
        ));
    }
    OutMail {
        from: format!("{project}@{}", cfg.domain),
        to: cfg.to.clone(),
        reply_to: None,
        subject: format!("[bridle/{project}] digest {}", now.format("%Y-%m-%d")),
        body: out,
    }
}
