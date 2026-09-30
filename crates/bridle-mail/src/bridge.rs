use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use async_trait::async_trait;
use bridle_api::client::Client;
use bridle_api::types::{MessageKind, SendRequest, When};

use crate::config::MailConfig;
use crate::parse::{Accepted, Rejection, evaluate};
use crate::store::MailStore;

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
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            if let Err(e) = self.poll_once().await {
                tracing::warn!("mail poll failed: {e:#}");
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

    async fn handle(&self, key: &str) -> anyhow::Result<Outcome> {
        let raw = self.store.get(key).await?;
        let mail = match evaluate(&raw, &self.cfg, &self.project) {
            Ok(m) => m,
            Err(Rejection::NotOurs) => return Ok(Outcome::Skipped),
            Err(why) => {
                tracing::warn!("mail {key} dropped: {why}");
                self.rejected
                    .lock()
                    .expect("rejected lock")
                    .insert(key.to_string());
                return Ok(Outcome::Rejected(why));
            }
        };
        let ses_id = key.strip_prefix(&self.cfg.prefix).unwrap_or(key);
        let saved = self.save_attachments(ses_id, &mail)?;
        let to = if self.cfg.advisor_running {
            "external:advisor"
        } else {
            "external:orchestrator"
        };
        self.sink
            .send(SendRequest {
                to: Some(to.to_string()),
                body: note_body(&mail, ses_id, &saved),
                kind: MessageKind::Note,
                when: When::Idle,
                reply_to: None,
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
