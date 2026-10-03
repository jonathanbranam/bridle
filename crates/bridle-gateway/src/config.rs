//! The optional `[gateway]` section of `<bridle home>/config.toml`.
//! See docs/design/human-web-ui.md. A machine that runs no gateway has no such section, and
//! nothing but `bridle gateway` ever reads it.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

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
    #[serde(default)]
    allow_any_interface: bool,
    username: Option<String>,
    /// An argon2 PHC string; `bridle gateway hash-password` makes one.
    password_hash: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct File {
    gateway: Option<GatewaySection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    pub bind: SocketAddr,
    /// `None` is the safe default: the gateway then answers nothing but health.
    pub login: Option<Login>,
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
        Self::parse(&text).map_err(|e| match e {
            // Parse errors need the path; the rest already say what is wrong.
            ConfigError::Parse { source, .. } => ConfigError::Parse { path, source },
            other => other,
        })
    }

    fn parse(text: &str) -> Result<Self, ConfigError> {
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
        Ok(Self { bind, login })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_file_and_section_give_loopback_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let c = GatewayConfig::load(dir.path()).expect("load");
        assert_eq!(c.bind, DEFAULT_BIND.parse().expect("addr"));
        assert!(c.bind.ip().is_loopback());
        let c = GatewayConfig::parse("[budget]\nx = 1\n").expect("other sections ignored");
        assert!(c.bind.ip().is_loopback());
    }

    #[test]
    fn bind_is_read() {
        let c = GatewayConfig::parse("[gateway]\nbind = \"127.0.0.1:9000\"\n").expect("parse");
        assert_eq!(c.bind.port(), 9000);
    }

    #[test]
    fn login_needs_both_halves_and_a_real_hash() {
        let c = GatewayConfig::parse("").expect("parse");
        assert!(c.login.is_none());
        let e = GatewayConfig::parse("[gateway]\nusername = \"a\"\n").expect_err("half");
        assert!(matches!(e, ConfigError::HalfLogin), "{e}");
        let e = GatewayConfig::parse("[gateway]\nusername = \"a\"\npassword_hash = \"x\"\n")
            .expect_err("bad hash");
        assert!(matches!(e, ConfigError::BadHash), "{e}");
        let hash = crate::auth::hash_password("pw").expect("hash");
        let c = GatewayConfig::parse(&format!(
            "[gateway]\nusername = \"a\"\npassword_hash = \"{hash}\"\n"
        ))
        .expect("ok");
        assert_eq!(c.login.expect("login").username, "a");
    }

    #[test]
    fn bad_bind_is_rejected() {
        let e = GatewayConfig::parse("[gateway]\nbind = \"nope\"\n").expect_err("bad");
        assert!(matches!(e, ConfigError::BadBind { .. }), "{e}");
    }

    #[test]
    fn any_interface_needs_explicit_opt_in() {
        let e = GatewayConfig::parse("[gateway]\nbind = \"0.0.0.0:7878\"\n").expect_err("refused");
        assert!(matches!(e, ConfigError::AnyInterface(_)), "{e}");
        let c = GatewayConfig::parse(
            "[gateway]\nbind = \"0.0.0.0:7878\"\nallow_any_interface = true\n",
        )
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
