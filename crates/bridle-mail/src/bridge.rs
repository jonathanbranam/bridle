use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use async_trait::async_trait;
use bridle_api::client::Client;
use bridle_api::types::{
    Agent, Message, MessageKind, MessageQuery, OpenQuestion, SendRequest, Task, When,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;

use crate::config::MailConfig;
use crate::local::{FixedLocal, Local};
use crate::outbound::{
    Feed, Mailer, OutMail, Sent, digest_due, digest_mail, is_open_question, question_mail,
};
use crate::parse::{Accepted, Rejection, evaluate};
use crate::store::MailStore;
use crate::token::Tokens;

/// Where accepted mail goes: the daemon's send endpoint. A trait so tests need no daemon.
#[async_trait]
pub trait Sink: Send + Sync {
    /// Returns the ids of the messages made.
    async fn send(&self, req: SendRequest) -> anyhow::Result<Vec<String>>;

    /// Unread messages sent to the bridge itself: the recipients' "got it" replies.
    async fn inbox(&self) -> anyhow::Result<Vec<Message>> {
        Ok(Vec::new())
    }

    async fn mark_read(&self, _id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

/// The daemon, as `external:mail`.
pub struct ClientSink(pub Client);

#[async_trait]
impl Sink for ClientSink {
    async fn send(&self, req: SendRequest) -> anyhow::Result<Vec<String>> {
        Ok(self.0.send(&req).await?.into_iter().map(|m| m.id).collect())
    }

    async fn inbox(&self) -> anyhow::Result<Vec<Message>> {
        Ok(self
            .0
            .list_messages(&MessageQuery {
                to: Some("me".to_string()),
                unread: true,
                ..MessageQuery::default()
            })
            .await?)
    }

    async fn mark_read(&self, id: &str) -> anyhow::Result<()> {
        self.0.mark_read(id).await?;
        Ok(())
    }
}

#[async_trait]
impl Feed for ClientSink {
    async fn human_messages(&self) -> anyhow::Result<Vec<Message>> {
        Ok(self
            .0
            .list_messages(&MessageQuery {
                to: Some("human".to_string()),
                limit: Some(500),
                ..MessageQuery::default()
            })
            .await?)
    }

    async fn agents(&self) -> anyhow::Result<Vec<Agent>> {
        Ok(self.0.list_agents().await?)
    }

    async fn tasks(&self) -> anyhow::Result<Vec<Task>> {
        Ok(self.0.list_tasks().await?)
    }

    async fn open_questions(&self) -> anyhow::Result<Vec<OpenQuestion>> {
        Ok(self.0.list_open_questions().await?)
    }
}

/// Everything the outbound half needs; without it the bridge is inbound-only.
pub struct Outbound {
    pub mailer: Arc<dyn Mailer>,
    pub feed: Arc<dyn Feed>,
    pub tokens: Tokens,
    pub sent: Sent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Delivered {
        to: String,
    },
    /// Dropped silently; the object stays for the bucket's lifecycle rule.
    Rejected(Rejection),
    /// Another project's mail, left for its bridge.
    Skipped,
    /// Ours, but this machine doesn't own the project (or the daemon wouldn't take it): left in
    /// the bucket for the owner.
    Waiting,
    /// Written to a name that is no project; the sender was sent the valid names.
    UnknownProject(String),
}

pub struct Bridge {
    store: Arc<dyn MailStore>,
    sink: Arc<dyn Sink>,
    cfg: MailConfig,
    project: String,
    /// Attachments are written to `<dir>/<ses message id>/<name>`.
    attachments_dir: PathBuf,
    /// Rejected keys, so a poll doesn't fetch and log the same stranger again.
    rejected: Mutex<HashSet<String>>,
    /// When each waiting mail was first seen, to time the "not delivered yet" reply.
    waiting_since: Mutex<HashMap<String, DateTime<Utc>>>,
    local: Arc<dyn Local>,
    outbound: Option<Outbound>,
}

impl Bridge {
    pub fn new(
        store: Arc<dyn MailStore>,
        sink: Arc<dyn Sink>,
        cfg: MailConfig,
        project: String,
        attachments_dir: PathBuf,
    ) -> Self {
        Bridge {
            store,
            sink,
            cfg,
            project,
            attachments_dir,
            rejected: Mutex::new(HashSet::new()),
            waiting_since: Mutex::new(HashMap::new()),
            local: Arc::new(FixedLocal {
                owns: true,
                aide: false,
            }),
            outbound: None,
        }
    }

    /// Where ownership and the aide's liveness come from; the default owns the project and
    /// has no aide.
    pub fn with_local(mut self, local: Arc<dyn Local>) -> Self {
        self.local = local;
        self
    }

    /// Adds question mails, the digest, and reply handling (which needs the token key).
    pub fn with_outbound(mut self, outbound: Outbound) -> Self {
        self.outbound = Some(outbound);
        self
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            if let Err(e) = self.poll_once().await {
                tracing::warn!("mail poll failed: {e:#}");
            }
            if let Err(e) = self.poll_outbound(chrono::Local::now().naive_local()).await {
                tracing::warn!("mail outbound failed: {e:#}");
            }
            tokio::time::sleep(self.cfg.poll_interval()).await;
        }
    }

    /// One pass over the bucket. A failure on one object (daemon down, bad object) is logged
    /// and leaves it in place for the next pass; it doesn't stop the others.
    pub async fn poll_once(&self) -> anyhow::Result<Vec<(String, Outcome)>> {
        self.poll_once_at(Utc::now()).await
    }

    pub async fn poll_once_at(&self, now: DateTime<Utc>) -> anyhow::Result<Vec<(String, Outcome)>> {
        let mut done = Vec::new();
        for key in self.store.list().await? {
            if self.rejected.lock().expect("rejected lock").contains(&key) {
                continue;
            }
            match self.handle(&key, now).await {
                Ok(outcome) => done.push((key, outcome)),
                Err(e) => tracing::warn!("mail {key}: {e:#}"),
            }
        }
        Ok(done)
    }

    /// Mails each new open question once, then the digest if it's due. `now` is the bridge
    /// machine's local time. A failure to send leaves the item unmarked, so the next pass retries.
    pub async fn poll_outbound(&self, now: NaiveDateTime) -> anyhow::Result<()> {
        let Some(out) = &self.outbound else {
            return Ok(());
        };
        self.relay_replies(out).await?;
        let messages = out.feed.human_messages().await?;
        let mut names = None;
        for q in messages.iter().filter(|m| is_open_question(m)) {
            if out.sent.question_mailed(&q.id) {
                continue;
            }
            if names.is_none() {
                names = Some(out.feed.agents().await.unwrap_or_default());
            }
            let name = names
                .iter()
                .flatten()
                .find(|a| a.id == q.from)
                .map_or(q.from.as_str(), |a| a.name.as_str());
            out.mailer
                .send(question_mail(
                    q,
                    name,
                    &self.project,
                    &self.cfg,
                    &out.tokens,
                ))
                .await
                .with_context(|| format!("mailing question {}", q.id))?;
            out.sent.mark_question(&q.id)?;
            tracing::info!("question {} mailed", q.id);
        }
        if digest_due(now, self.cfg.digest_time()?, out.sent.last_digest()) {
            let open = out.feed.open_questions().await?;
            let tasks = out.feed.tasks().await?;
            out.mailer
                .send(digest_mail(
                    &self.project,
                    &self.cfg,
                    &messages,
                    &open,
                    &tasks,
                    now,
                ))
                .await
                .context("mailing the digest")?;
            out.sent.mark_digest(now.date())?;
            tracing::info!("digest mailed");
        }
        Ok(())
    }

    /// A recipient's reply to a delivered mail ("got it: ...") goes back to its sender.
    async fn relay_replies(&self, out: &Outbound) -> anyhow::Result<()> {
        for m in self.sink.inbox().await? {
            let Some((from, subject)) = m.reply_to.as_deref().and_then(|id| out.sent.sender_of(id))
            else {
                continue;
            };
            let subject = if subject.to_ascii_lowercase().starts_with("re:") {
                subject
            } else {
                format!("Re: {subject}")
            };
            out.mailer
                .send(OutMail {
                    from: self.address(),
                    to: from,
                    reply_to: None,
                    subject,
                    body: crate::parse::cap(m.body.trim(), self.cfg.max_body_chars),
                })
                .await
                .with_context(|| format!("relaying {} to its sender", m.id))?;
            self.sink.mark_read(&m.id).await?;
            tracing::info!("reply {} relayed", m.id);
        }
        Ok(())
    }

    fn address(&self) -> String {
        format!("{}@{}", self.project, self.cfg.domain)
    }

    /// Mail this machine can't deliver: after `not_delivered_after_secs` the sender is told once.
    /// The object stays in the bucket either way.
    async fn wait(&self, key: &str, mail: &Accepted, now: DateTime<Utc>) -> anyhow::Result<()> {
        let Some(out) = &self.outbound else {
            return Ok(());
        };
        let since = *self
            .waiting_since
            .lock()
            .expect("waiting lock")
            .entry(key.to_string())
            .or_insert(now);
        let limit = chrono::Duration::seconds(self.cfg.not_delivered_after_secs as i64);
        if now - since < limit || out.sent.notified(key) {
            return Ok(());
        }
        out.mailer
            .send(OutMail {
                from: self.address(),
                to: mail.from.clone(),
                reply_to: None,
                subject: format!("Re: {}", mail.subject),
                body: format!(
                    "not delivered yet: no machine is running {}. Your mail is kept and will \
                     be delivered when one is.",
                    self.project
                ),
            })
            .await
            .context("sending the not-delivered notice")?;
        out.sent.mark_notified(key)?;
        tracing::info!("mail {key}: {} told it is waiting", mail.from);
        Ok(())
    }

    /// A mail to a name that is no project: the sender gets the valid names, and the mail is
    /// done with.
    async fn unknown(&self, key: &str, name: &str, from: &str) -> anyhow::Result<Outcome> {
        let Some(out) = &self.outbound else {
            return Ok(Outcome::Skipped);
        };
        out.mailer
            .send(OutMail {
                from: self.address(),
                to: from.to_string(),
                reply_to: None,
                subject: format!("Re: your mail to {name}@{}", self.cfg.domain),
                body: format!(
                    "There is no project named {name}. The projects are: {}.",
                    self.cfg.valid_projects(&self.project).join(", ")
                ),
            })
            .await
            .context("answering an unknown project")?;
        self.store.delete(key).await?;
        tracing::info!("mail {key} from {from} to unknown project {name} answered");
        Ok(Outcome::UnknownProject(name.to_string()))
    }

    fn reject(&self, key: &str, why: Rejection) -> Outcome {
        tracing::warn!("mail {key} dropped: {why}");
        self.rejected
            .lock()
            .expect("rejected lock")
            .insert(key.to_string());
        Outcome::Rejected(why)
    }

    /// An answer to a question mail: the token must be ours for that question, and the question
    /// still open. Returns the agent to answer, and the question id.
    async fn check_reply(&self, id: &str, tag: &str) -> Result<Message, Rejection> {
        let out = self.outbound.as_ref().ok_or(Rejection::BadToken)?;
        if !out.tokens.verify(&self.project, id, tag) {
            return Err(Rejection::BadToken);
        }
        let messages = out.feed.human_messages().await.map_err(|e| {
            tracing::warn!("reading the human's inbox: {e:#}");
            Rejection::BadToken
        })?;
        messages
            .into_iter()
            .find(|m| m.id == id && is_open_question(m))
            .ok_or_else(|| Rejection::Replayed(id.to_string()))
    }

    async fn handle(&self, key: &str, now: DateTime<Utc>) -> anyhow::Result<Outcome> {
        let raw = self.store.get(key).await?;
        let mail = match evaluate(&raw, &self.cfg, &self.project) {
            Ok(m) => m,
            Err(Rejection::NotOurs) => return Ok(Outcome::Skipped),
            Err(Rejection::UnknownProject { name, from }) => {
                return self.unknown(key, &name, &from).await;
            }
            Err(why) => return Ok(self.reject(key, why)),
        };
        if !self.local.owns_project().await {
            self.wait(key, &mail, now).await?;
            return Ok(Outcome::Waiting);
        }
        let question = match &mail.route.reply {
            Some((id, tag)) => match self.check_reply(id, tag).await {
                Ok(q) => Some(q),
                Err(why) => return Ok(self.reject(key, why)),
            },
            None => None,
        };
        let ses_id = key.strip_prefix(&self.cfg.prefix).unwrap_or(key);
        let saved = self.save_attachments(ses_id, &mail)?;
        let to = if let Some(q) = &question {
            q.from.as_str()
        } else if self.local.aide_running().await {
            "external:aide"
        } else {
            "external:orchestrator"
        };
        let sent = self
            .sink
            .send(SendRequest {
                to: Some(to.to_string()),
                body: note_body(&mail, ses_id, &saved),
                kind: if question.is_some() {
                    MessageKind::Answer
                } else {
                    MessageKind::Note
                },
                when: When::Idle,
                reply_to: question.as_ref().map(|q| q.id.clone()),
                task: mail.route.task.clone(),
            })
            .await;
        let ids = match sent {
            Ok(ids) => ids,
            Err(e) => {
                // A daemon that stays down is "no machine running the project" to the sender.
                self.wait(key, &mail, now).await?;
                return Err(e.context("delivering to the daemon"));
            }
        };
        if let Some(out) = &self.outbound {
            for id in ids {
                out.sent.remember_sender(&id, &mail.from, &mail.subject)?;
            }
        }
        self.store.delete(key).await?;
        tracing::info!("mail {key} from {} delivered to {to}", mail.from);
        Ok(Outcome::Delivered { to: to.to_string() })
    }

    fn save_attachments(&self, ses_id: &str, mail: &Accepted) -> anyhow::Result<Vec<PathBuf>> {
        if mail.attachments.is_empty() {
            return Ok(Vec::new());
        }
        // The id names a directory; SES ids are plain, but the key is the bucket's to choose.
        let dir = self.attachments_dir.join(ses_id.replace(['/', '\\'], "_"));
        std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        mail.attachments
            .iter()
            .map(|a| {
                let path = dir.join(&a.name);
                std::fs::write(&path, &a.data)
                    .with_context(|| format!("writing {}", path.display()))?;
                Ok(path)
            })
            .collect()
    }
}

fn note_body(mail: &Accepted, ses_id: &str, saved: &[PathBuf]) -> String {
    let mut out = format!("via email from {} (SES message {ses_id})\n", mail.from);
    if !mail.subject.is_empty() {
        out.push_str(&format!("Subject: {}\n", mail.subject));
    }
    out.push('\n');
    out.push_str(&mail.text);
    if !saved.is_empty() {
        out.push_str("\n\nAttachments kept:");
        for p in saved {
            out.push_str(&format!("\n- {}", p.display()));
        }
    }
    if !mail.dropped.is_empty() {
        out.push_str("\n\nAttachments dropped:");
        for d in &mail.dropped {
            out.push_str(&format!("\n- {d}"));
        }
    }
    out
}
