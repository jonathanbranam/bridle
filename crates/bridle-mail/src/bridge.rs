use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use async_trait::async_trait;
use bridle_api::client::Client;
use bridle_api::types::{
    Agent, Message, MessageKind, MessageQuery, OpenQuestion, SendRequest, Task, When,
};
use chrono::NaiveDateTime;

use crate::config::MailConfig;
use crate::outbound::{
    Feed, Mailer, Sent, digest_due, digest_mail, is_open_question, question_mail,
};
use crate::parse::{Accepted, Rejection, evaluate};
use crate::store::MailStore;
use crate::token::Tokens;

/// Where accepted mail goes: the daemon's send endpoint. A trait so tests need no daemon.
#[async_trait]
pub trait Sink: Send + Sync {
    async fn send(&self, req: SendRequest) -> anyhow::Result<()>;
}

/// The daemon, as `external:mail`.
pub struct ClientSink(pub Client);

#[async_trait]
impl Sink for ClientSink {
    async fn send(&self, req: SendRequest) -> anyhow::Result<()> {
        self.0.send(&req).await?;
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
            outbound: None,
        }
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
        let mut done = Vec::new();
        for key in self.store.list().await? {
            if self.rejected.lock().expect("rejected lock").contains(&key) {
                continue;
            }
            match self.handle(&key).await {
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

    async fn handle(&self, key: &str) -> anyhow::Result<Outcome> {
        let raw = self.store.get(key).await?;
        let mail = match evaluate(&raw, &self.cfg, &self.project) {
            Ok(m) => m,
            Err(Rejection::NotOurs) => return Ok(Outcome::Skipped),
            Err(why) => return Ok(self.reject(key, why)),
        };
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
        } else if self.cfg.advisor_running {
            "external:advisor"
        } else {
            "external:orchestrator"
        };
        self.sink
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
            .await
            .context("delivering to the daemon")?;
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
