//! The governor's primary usage source: one HTTPS GET of the OAuth usage
//! endpoint that `/usage` and `get_usage` read underneath. No `claude`
//! process and no model call. The endpoint and the token's location are
//! undocumented Claude Code internals (docs/design/usage-and-budget.md, "Seeing the
//! windows"); any failure here just sends the poll to the probe.
//!
//! The request goes through the system `curl` rather than reqwest: the
//! workspace builds reqwest without TLS, and curl brings the platform's
//! certificate handling for free. The token travels on curl's stdin
//! (`--config -`), never in argv where `ps` would show it.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub const DEFAULT_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const TIMEOUT: Duration = Duration::from_secs(10);

/// Where to ask and with which token. `None` token means "look it up in
/// Claude Code's credential store on every poll" (it refreshes the token
/// itself, so a cached copy would go stale).
#[derive(Debug, Clone)]
pub struct UsageHttp {
    pub url: String,
    pub token: Option<String>,
}

impl Default for UsageHttp {
    fn default() -> Self {
        UsageHttp {
            url: DEFAULT_URL.to_string(),
            token: None,
        }
    }
}

impl UsageHttp {
    /// Fetches the usage and returns it shaped like a `get_usage` response
    /// (`{"rate_limits": {...}}`) so one parser serves both sources.
    pub async fn fetch(&self) -> Result<Value, String> {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => oauth_token().await?,
        };
        let body = curl(&self.url, &token).await?;
        let v: Value = serde_json::from_str(&body).map_err(|e| format!("bad usage JSON: {e}"))?;
        if !v.is_object() || v.get("five_hour").is_none() {
            return Err("usage response has no five_hour window".to_string());
        }
        Ok(json!({ "rate_limits": v }))
    }
}

async fn curl(url: &str, token: &str) -> Result<String, String> {
    let mut child = Command::new("curl")
        .args(["--silent", "--show-error", "--fail", "--max-time"])
        .arg(TIMEOUT.as_secs().to_string())
        .args(["--config", "-"])
        .arg(url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("curl: {e}"))?;
    let config = format!(
        "header = \"Authorization: Bearer {token}\"\nheader = \"anthropic-beta: oauth-2025-04-20\"\n"
    );
    let mut stdin = child.stdin.take().ok_or("curl: no stdin")?;
    stdin
        .write_all(config.as_bytes())
        .await
        .map_err(|e| format!("curl: {e}"))?;
    drop(stdin);
    let out = child
        .wait_with_output()
        .await
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("curl {}: {}", out.status, err.trim()));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("curl: {e}"))
}

/// Claude Code's OAuth access token: `~/.claude/.credentials.json`, else
/// (macOS) the login keychain item `Claude Code-credentials`.
async fn oauth_token() -> Result<String, String> {
    if let Some(path) =
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".claude").join(".credentials.json"))
        && let Ok(text) = tokio::fs::read_to_string(&path).await
    {
        return token_from_credentials(&text);
    }
    if cfg!(target_os = "macos") {
        let out = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "Claude Code-credentials",
                "-w",
            ])
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(|e| format!("security: {e}"))?;
        if out.status.success() {
            return token_from_credentials(&String::from_utf8_lossy(&out.stdout));
        }
    }
    Err("no Claude Code OAuth credentials found".to_string())
}

fn token_from_credentials(text: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(text.trim()).map_err(|e| format!("credentials: {e}"))?;
    v.pointer("/claudeAiOauth/accessToken")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "credentials have no claudeAiOauth.accessToken".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_comes_out_of_the_credentials_json() {
        let t = token_from_credentials(r#"{"claudeAiOauth":{"accessToken":"abc","x":1}}"#);
        assert_eq!(t.as_deref(), Ok("abc"));
        assert!(token_from_credentials("{}").is_err());
    }
}
