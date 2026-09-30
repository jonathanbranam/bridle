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
    /// Route mail to `external:advisor` instead of `external:orchestrator`. A switch until
    /// advisor liveness is detected (the pid file in the rs7p ticket).
    pub advisor_running: bool,
    /// Longest body kept, in characters; the rest is cut and marked.
    pub max_body_chars: usize,
    /// Largest attachment kept, in bytes.
    pub max_attachment_bytes: usize,
    pub poll_secs: u64,
}

impl Default for MailConfig {
    fn default() -> Self {
        MailConfig {
            bucket: String::new(),
            prefix: "inbound/".to_string(),
            region: None,
            domain: "dev.branam.us".to_string(),
            allow: Vec::new(),
            advisor_running: false,
            max_body_chars: 20_000,
            max_attachment_bytes: 1024 * 1024,
            poll_secs: 30,
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
        Ok(cfg)
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = bridle_api::discovery::bridle_home().join("config.toml");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))
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
