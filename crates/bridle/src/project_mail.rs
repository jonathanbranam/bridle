//! The mail half of `bridle project start` (kx5u K6): when `[mail] projects` names the project,
//! this machine needs the `[mail]` section and, so replies to questions mailed from the old
//! machine still verify, the old machine's `mail.key`. The unit install and start are the
//! caller's; this only moves config and key.

use std::io::Write;
use std::path::Path;

use bridle_mail::MailConfig;
use toml_edit::DocumentMut;

use crate::project_sync::ssh;

/// The previous owner, as far as mail needs it. A trait so tests need no ssh.
pub trait Old {
    /// Its `config.toml` text (empty when it has none).
    fn config(&self) -> Result<String, String>;
    /// Its `mail.key` as hex, or `None` when it has none.
    fn key_hex(&self) -> Result<Option<String>, String>;
}

pub struct OldMachine {
    pub host: String,
}

impl Old for OldMachine {
    fn config(&self) -> Result<String, String> {
        ssh(
            &self.host,
            "f=\"$HOME/.bridle/config.toml\"; if [ -f \"$f\" ]; then cat \"$f\"; fi",
            None,
        )
    }

    fn key_hex(&self) -> Result<Option<String>, String> {
        let hex = ssh(
            &self.host,
            "f=\"$HOME/.bridle/mail.key\"; if [ -f \"$f\" ]; then od -An -v -tx1 \"$f\" | tr -d ' \\n'; fi",
            None,
        )?;
        Ok(Some(hex.trim().to_string()).filter(|h| !h.is_empty()))
    }
}

pub fn mails(config: &str, project: &str) -> bool {
    MailConfig::parse(config).is_ok_and(|c| c.projects.iter().any(|p| p == project))
}

/// Makes this machine's `config.toml` and `mail.key` ready for the project's bridge. `Ok(false)`
/// when the project has no `[mail]` entry here or on the old machine: nothing to do, and a
/// project without mail moves as before.
pub fn bring(
    old: Option<&dyn Old>,
    home: &Path,
    project: &str,
    out: &mut dyn Write,
) -> Result<bool, String> {
    let config_path = home.join("config.toml");
    let local = std::fs::read_to_string(&config_path).unwrap_or_default();
    if !mails(&local, project) {
        let theirs = match old {
            Some(o) => o.config()?,
            None => return Ok(false),
        };
        if !mails(&theirs, project) {
            return Ok(false);
        }
        let section = theirs
            .parse::<DocumentMut>()
            .map_err(|e| format!("the old machine's config.toml: {e}"))?
            .get("mail")
            .cloned()
            .ok_or("the old machine has no [mail] section")?;
        let mut doc: DocumentMut = local
            .parse()
            .map_err(|e| format!("invalid config.toml: {e}"))?;
        doc["mail"] = section;
        std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
        std::fs::write(&config_path, doc.to_string()).map_err(|e| e.to_string())?;
        let _ = writeln!(
            out,
            "{project}: copied [mail] into {}",
            config_path.display()
        );
    }

    let key_path = home.join("mail.key");
    let theirs = match old {
        Some(o) => o.key_hex()?.and_then(|h| hex::decode(h).ok()),
        None => None,
    };
    match (std::fs::read(&key_path).ok(), theirs) {
        (None, Some(key)) => {
            write_key(&key_path, &key)?;
            let _ = writeln!(out, "{project}: copied mail.key (0600)");
        }
        (Some(mine), Some(key)) if mine != key => {
            let _ = writeln!(
                out,
                "{project}: this machine has its own mail.key; replies to mail sent from the old machine will fail verification"
            );
        }
        (None, None) => {
            let _ = writeln!(
                out,
                "{project}: no mail.key from the old machine; replies to mail sent from it will fail verification"
            );
        }
        _ => {}
    }
    Ok(true)
}

fn write_key(path: &Path, key: &[u8]) -> Result<(), String> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .and_then(|mut f| f.write_all(key))
        .map_err(|e| format!("writing {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    struct Fake {
        config: String,
        key: Option<Vec<u8>>,
    }

    impl Old for Fake {
        fn config(&self) -> Result<String, String> {
            Ok(self.config.clone())
        }
        fn key_hex(&self) -> Result<Option<String>, String> {
            Ok(self.key.as_ref().map(hex::encode))
        }
    }

    const MAIL: &str = "[mail]\nbucket = \"b\"\nallow = [\"me@example.com\"]\nprojects = [\"p\"]\n";

    #[test]
    fn mail_section_and_key_follow_the_project() {
        let new = tempfile::tempdir().unwrap();
        std::fs::write(
            new.path().join("config.toml"),
            "[machine]\nname = \"nuc\"\n",
        )
        .unwrap();
        let old = Fake {
            config: format!("[machine]\nname = \"mbp\"\n{MAIL}"),
            key: Some(vec![7; 32]),
        };
        let mut out = Vec::new();
        assert!(bring(Some(&old), new.path(), "p", &mut out).unwrap());
        let text = std::fs::read_to_string(new.path().join("config.toml")).unwrap();
        assert!(text.contains("name = \"nuc\""), "{text}");
        assert!(mails(&text, "p"));
        let key = new.path().join("mail.key");
        assert_eq!(std::fs::read(&key).unwrap(), vec![7; 32]);
        assert_eq!(
            std::fs::metadata(&key).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn a_project_with_no_mail_entry_is_left_alone() {
        let new = tempfile::tempdir().unwrap();
        let old = Fake {
            config: MAIL.into(),
            key: Some(vec![7; 32]),
        };
        let mut out = Vec::new();
        assert!(!bring(Some(&old), new.path(), "other", &mut out).unwrap());
        assert!(!bring(None, new.path(), "other", &mut out).unwrap());
        assert!(!new.path().join("config.toml").exists());
        assert!(!new.path().join("mail.key").exists());
    }

    #[test]
    fn no_key_to_copy_says_replies_will_fail() {
        let new = tempfile::tempdir().unwrap();
        std::fs::write(new.path().join("config.toml"), MAIL).unwrap();
        let mut out = Vec::new();
        assert!(bring(None, new.path(), "p", &mut out).unwrap());
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("fail verification")
        );
    }

    #[test]
    fn an_existing_different_key_is_kept_and_reported() {
        let new = tempfile::tempdir().unwrap();
        std::fs::write(new.path().join("config.toml"), MAIL).unwrap();
        std::fs::write(new.path().join("mail.key"), [1u8; 32]).unwrap();
        let old = Fake {
            config: MAIL.into(),
            key: Some(vec![7; 32]),
        };
        let mut out = Vec::new();
        bring(Some(&old), new.path(), "p", &mut out).unwrap();
        assert_eq!(
            std::fs::read(new.path().join("mail.key")).unwrap(),
            vec![1; 32]
        );
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("fail verification")
        );
    }
}
