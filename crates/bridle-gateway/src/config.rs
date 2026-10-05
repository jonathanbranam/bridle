//! The optional `[gateway]` section of `<bridle home>/config.toml`.
//! See docs/design/human-web-ui.md. A machine that runs no gateway has no such section, and
//! nothing but `bridle gateway` ever reads it.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

use crate::ui::{OnMismatch, UiConfig};

/// Loopback, so a gateway with no config is never reachable from another machine.
pub const DEFAULT_BIND: &str = "127.0.0.1:7878";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parsing [gateway] in {path}: {source}")]
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("[gateway] bind {value:?} is not an address like 127.0.0.1:7878: {source}")]
    BadBind {
        value: String,
        source: std::net::AddrParseError,
    },
    #[error(
        "[gateway] bind {0} listens on every interface; bind the loopback or Tailscale address, \
         or set `allow_any_interface = true` to mean it"
    )]
    AnyInterface(SocketAddr),
    #[error("[gateway] ui_version_mismatch is {0:?}; use \"warn\" or \"refuse\"")]
    BadMismatch(String),
    #[error("[interactions] {key} is {value:?}; use a number and s, m or h, like \"10m\"")]
    BadDuration { key: String, value: String },
    #[error("[gateway] needs both `username` and `password_hash`, or neither")]
    HalfLogin,
    #[error(
        "[gateway] password_hash is not an argon2 hash; make one with `bridle gateway hash-password`"
    )]
    BadHash,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewaySection {
    bind: Option<String>,
    /// Default true; false makes `bridle gateway` exit 0 without starting.
    enabled: Option<bool>,
    #[serde(default)]
    allow_any_interface: bool,
    username: Option<String>,
    /// An argon2 PHC string; `bridle gateway hash-password` makes one.
    password_hash: Option<String>,
    /// Folder of the built UI; default `<bridle home>/ui`.
    ui_dir: Option<PathBuf>,
    /// "warn" (default) or "refuse" when the UI's recorded API version differs.
    ui_version_mismatch: Option<String>,
    /// Base URL of the UI as the human opens it; read by `bridle link`, not the gateway.
    #[allow(dead_code)]
    public_url: Option<String>,
}

/// The `[interactions]` section: the knobs of the human-time interval rules.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct InteractionsSection {
    gap: Option<String>,
    tail: Option<String>,
    lead: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct File {
    gateway: Option<GatewaySection>,
    interactions: Option<InteractionsSection>,
}

/// How prompts and replies turn into human time (docs/design/human-web-ui.md "Human time").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionsConfig {
    /// A prompt within this long of the previous reply finishing continues the run.
    pub gap: Duration,
    /// What the last prompt or reply of a run counts after it.
    pub tail: Duration,
    /// What the first prompt of a run counts before it (composing it).
    pub lead: Duration,
}

impl Default for InteractionsConfig {
    fn default() -> Self {
        Self {
            gap: Duration::from_secs(600),
            tail: Duration::from_secs(120),
            lead: Duration::from_secs(60),
        }
    }
}

/// Parses "10m"-style durations: an integer followed by `s`, `m` or `h`.
fn parse_duration(key: &str, s: &str) -> Result<Duration, ConfigError> {
    let bad = || ConfigError::BadDuration {
        key: key.to_string(),
        value: s.to_string(),
    };
    let unit = s.chars().last().ok_or_else(bad)?;
    let n: u64 = s[..s.len() - 1].parse().map_err(|_| bad())?;
    let per = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        _ => return Err(bad()),
    };
    Ok(Duration::from_secs(n * per))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    pub bind: SocketAddr,
    pub enabled: bool,
    /// `None` is the safe default: the gateway then answers nothing but health.
    pub login: Option<Login>,
    pub ui: UiConfig,
    pub interactions: InteractionsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub username: String,
    pub password_hash: String,
}

impl GatewayConfig {
    /// Loads `<home>/config.toml`; an absent file or section gives the defaults.
    pub fn load(home: &Path) -> Result<Self, ConfigError> {
        let path = home.join("config.toml");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(source) => return Err(ConfigError::Read { path, source }),
        };
        Self::parse(&text, home).map_err(|e| match e {
            // Parse errors need the path; the rest already say what is wrong.
            ConfigError::Parse { source, .. } => ConfigError::Parse { path, source },
            other => other,
        })
    }

    fn parse(text: &str, home: &Path) -> Result<Self, ConfigError> {
        let file: File = toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: PathBuf::new(),
            source: Box::new(source),
        })?;
        let section = file.gateway.unwrap_or_default();
        let value = section.bind.unwrap_or_else(|| DEFAULT_BIND.to_string());
        let bind: SocketAddr = value
            .parse()
            .map_err(|source| ConfigError::BadBind { value, source })?;
        if bind.ip().is_unspecified() && !section.allow_any_interface {
            return Err(ConfigError::AnyInterface(bind));
        }
        let login = match (section.username, section.password_hash) {
            (None, None) => None,
            (Some(username), Some(password_hash)) => {
                if !crate::auth::is_valid_hash(&password_hash) {
                    return Err(ConfigError::BadHash);
                }
                Some(Login {
                    username,
                    password_hash,
                })
            }
            _ => return Err(ConfigError::HalfLogin),
        };
        let on_mismatch = match section.ui_version_mismatch.as_deref() {
            None | Some("warn") => OnMismatch::Warn,
            Some("refuse") => OnMismatch::Refuse,
            Some(other) => return Err(ConfigError::BadMismatch(other.to_string())),
        };
        let ui = UiConfig {
            dir: section.ui_dir.unwrap_or_else(|| home.join("ui")),
            on_mismatch,
        };
        let mut interactions = InteractionsConfig::default();
        if let Some(i) = file.interactions {
            for (key, text, slot) in [
                ("gap", i.gap, &mut interactions.gap),
                ("tail", i.tail, &mut interactions.tail),
                ("lead", i.lead, &mut interactions.lead),
            ] {
                if let Some(text) = text {
                    *slot = parse_duration(key, &text)?;
                }
            }
        }
        Ok(Self {
            bind,
            enabled: section.enabled.unwrap_or(true),
            login,
            ui,
            interactions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_at(text: &str) -> Result<GatewayConfig, ConfigError> {
        GatewayConfig::parse(text, Path::new("/home"))
    }

    #[test]
    fn enabled_defaults_to_true() {
        assert!(parse_at("").expect("parse").enabled);
        assert!(
            !parse_at("[gateway]\nenabled = false\n")
                .expect("parse")
                .enabled
        );
    }

    #[test]
    fn ui_defaults_and_overrides() {
        let c = parse_at("").expect("parse");
        assert_eq!(c.ui.dir, Path::new("/home/ui"));
        assert_eq!(c.ui.on_mismatch, OnMismatch::Warn);
        let c = parse_at("[gateway]\nui_dir = \"/x\"\nui_version_mismatch = \"refuse\"\n")
            .expect("parse");
        assert_eq!(c.ui.dir, Path::new("/x"));
        assert_eq!(c.ui.on_mismatch, OnMismatch::Refuse);
        let e = parse_at("[gateway]\nui_version_mismatch = \"x\"\n").expect_err("bad");
        assert!(matches!(e, ConfigError::BadMismatch(_)), "{e}");
    }

    #[test]
    fn absent_file_and_section_give_loopback_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let c = GatewayConfig::load(dir.path()).expect("load");
        assert_eq!(c.bind, DEFAULT_BIND.parse().expect("addr"));
        assert!(c.bind.ip().is_loopback());
        let c = parse_at("[budget]\nx = 1\n").expect("other sections ignored");
        assert!(c.bind.ip().is_loopback());
    }

    #[test]
    fn interactions_defaults_and_overrides() {
        let c = parse_at("").expect("parse");
        assert_eq!(c.interactions, InteractionsConfig::default());
        let c = parse_at("[interactions]\ngap = \"5m\"\ntail = \"30s\"\nlead = \"1h\"\n")
            .expect("parse");
        assert_eq!(c.interactions.gap, Duration::from_secs(300));
        assert_eq!(c.interactions.tail, Duration::from_secs(30));
        assert_eq!(c.interactions.lead, Duration::from_secs(3600));
        let e = parse_at("[interactions]\ngap = \"soon\"\n").expect_err("bad");
        assert!(matches!(e, ConfigError::BadDuration { .. }), "{e}");
    }

    #[test]
    fn bind_is_read() {
        let c = parse_at("[gateway]\nbind = \"127.0.0.1:9000\"\n").expect("parse");
        assert_eq!(c.bind.port(), 9000);
    }

    #[test]
    fn login_needs_both_halves_and_a_real_hash() {
        let c = parse_at("").expect("parse");
        assert!(c.login.is_none());
        let e = parse_at("[gateway]\nusername = \"a\"\n").expect_err("half");
        assert!(matches!(e, ConfigError::HalfLogin), "{e}");
        let e =
            parse_at("[gateway]\nusername = \"a\"\npassword_hash = \"x\"\n").expect_err("bad hash");
        assert!(matches!(e, ConfigError::BadHash), "{e}");
        let hash = crate::auth::hash_password("pw").expect("hash");
        let c = parse_at(&format!(
            "[gateway]\nusername = \"a\"\npassword_hash = \"{hash}\"\n"
        ))
        .expect("ok");
        assert_eq!(c.login.expect("login").username, "a");
    }

    #[test]
    fn bad_bind_is_rejected() {
        let e = parse_at("[gateway]\nbind = \"nope\"\n").expect_err("bad");
        assert!(matches!(e, ConfigError::BadBind { .. }), "{e}");
    }

    #[test]
    fn any_interface_needs_explicit_opt_in() {
        let e = parse_at("[gateway]\nbind = \"0.0.0.0:7878\"\n").expect_err("refused");
        assert!(matches!(e, ConfigError::AnyInterface(_)), "{e}");
        let c = parse_at("[gateway]\nbind = \"0.0.0.0:7878\"\nallow_any_interface = true\n")
            .expect("allowed");
        assert!(c.bind.ip().is_unspecified());
    }

    #[test]
    fn malformed_file_names_the_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("config.toml"), "[gateway\n").expect("write");
        let e = GatewayConfig::load(dir.path()).expect_err("malformed");
        assert!(e.to_string().contains("config.toml"), "{e}");
    }
}
