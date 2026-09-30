use std::time::Duration;

use serde::Deserialize;

/// `[mail]` in `~/.bridle/config.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MailConfig {
    pub bucket: String,
    /// Where SES writes (the receipt rule's object key prefix).
    pub prefix: String,
    pub region: Option<String>,
    /// The subdomain mail arrives on; only recipients here are considered.
    pub domain: String,
    /// Addresses (`me@example.com`) and domains (`@example.com`). A domain entry still needs
    /// DMARC `pass` for that domain, like every sender.
    pub allow: Vec<String>,
    /// Every project name mail may be addressed to, on any machine. The bridge of the first
    /// one replies to a mail for a name not listed here (with the list); empty: no such reply.
    pub projects: Vec<String>,
    /// How long mail may wait for its project's owner before the sender is told, in seconds.
    pub not_delivered_after_secs: u64,
    /// Longest body kept, in characters; the rest is cut and marked.
    pub max_body_chars: usize,
    /// Largest attachment kept, in bytes.
    pub max_attachment_bytes: usize,
    pub poll_secs: u64,
    /// Where question mails and the digest go (the apex is Google Workspace, not SES).
    pub to: String,
    /// The daily digest's time, `HH:MM`, on the bridge machine's clock (US Eastern for the
    /// human's machines, as the budget schedule assumes).
    pub digest_at: String,
}

impl Default for MailConfig {
    fn default() -> Self {
        MailConfig {
            bucket: String::new(),
            prefix: "inbound/".to_string(),
            region: None,
            domain: "dev.branam.us".to_string(),
            allow: Vec::new(),
            projects: Vec::new(),
            not_delivered_after_secs: 3600,
            max_body_chars: 20_000,
            max_attachment_bytes: 1024 * 1024,
            poll_secs: 30,
            to: "dev@branam.us".to_string(),
            digest_at: "06:30".to_string(),
        }
    }
}

impl MailConfig {
    /// Reads the `[mail]` table out of a `config.toml`'s text.
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        #[derive(Deserialize)]
        struct File {
            mail: Option<MailConfig>,
        }
        let file: File = toml::from_str(text)?;
        let cfg = file.mail.unwrap_or_default();
        anyhow::ensure!(!cfg.bucket.is_empty(), "[mail] bucket is not set");
        anyhow::ensure!(
            !cfg.allow.is_empty(),
            "[mail] allow is empty: no sender could ever be accepted"
        );
        cfg.digest_time()?;
        Ok(cfg)
    }

    pub fn digest_time(&self) -> anyhow::Result<chrono::NaiveTime> {
        chrono::NaiveTime::parse_from_str(&self.digest_at, "%H:%M")
            .map_err(|e| anyhow::anyhow!("[mail] digest_at {:?} is not HH:MM: {e}", self.digest_at))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = bridle_api::discovery::bridle_home().join("config.toml");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))
    }

    /// The names mail is valid for: `projects` plus the bridge's own.
    pub fn valid_projects(&self, own: &str) -> Vec<String> {
        let mut all = self.projects.clone();
        if !all.iter().any(|p| p.eq_ignore_ascii_case(own)) {
            all.push(own.to_string());
        }
        all
    }

    /// Whether this bridge is the one that answers mail to an unknown project name.
    pub fn answers_unknown(&self, own: &str) -> bool {
        self.projects
            .first()
            .is_some_and(|p| p.eq_ignore_ascii_case(own))
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_secs.max(1))
    }

    /// The `allow` list matches `address` (already lowercased) by address or by `@domain`.
    pub fn allows(&self, address: &str) -> bool {
        let domain = address.rsplit_once('@').map(|(_, d)| d).unwrap_or("");
        self.allow.iter().any(|entry| {
            let e = entry.trim().to_ascii_lowercase();
            e == address || (e.starts_with('@') && e[1..] == *domain)
        })
    }
}
