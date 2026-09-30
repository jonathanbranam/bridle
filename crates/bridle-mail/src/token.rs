//! Reply tokens: `<project>+r-<msgid>.<tag>@domain`. The tag is an HMAC keyed per machine over
//! the project and the question id, so a reply address answers that one question and can't be
//! made up for another. It binds; it doesn't hide (the address is in the mail).

use std::path::Path;

use anyhow::Context;
use sha2::{Digest, Sha256};

/// Tag length in hex characters (96 bits): plenty against guessing, short in an address.
const TAG_LEN: usize = 24;

#[derive(Clone)]
pub struct Tokens {
    key: Vec<u8>,
}

impl Tokens {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Tokens { key: key.into() }
    }

    /// The machine's key, created (mode 0600) on first use.
    pub fn load_or_create(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read(path) {
            Ok(k) if k.len() >= 16 => return Ok(Tokens::new(k)),
            Ok(_) => anyhow::bail!("{} is too short to be a key", path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
        }
        // No `rand` in the workspace; a v4 UUID is OS randomness (as the daemon's tokens do).
        let mut key = uuid::Uuid::new_v4().into_bytes().to_vec();
        key.extend_from_slice(&uuid::Uuid::new_v4().into_bytes());
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .and_then(|mut f| f.write_all(&key))
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(Tokens::new(key))
    }

    pub fn tag(&self, project: &str, msgid: &str) -> String {
        let mac = hmac_sha256(&self.key, format!("{project}\n{msgid}").as_bytes());
        hex::encode(mac)[..TAG_LEN].to_string()
    }

    pub fn verify(&self, project: &str, msgid: &str, tag: &str) -> bool {
        let want = self.tag(project, msgid);
        // Constant time: no early exit on the first differing byte.
        want.len() == tag.len()
            && want
                .bytes()
                .zip(tag.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0
    }

    /// The Reply-To local part's `<project>+r-<msgid>.<tag>`.
    pub fn reply_local(&self, project: &str, msgid: &str) -> String {
        format!("{project}+r-{msgid}.{}", self.tag(project, msgid))
    }
}

/// RFC 2104 over SHA-256 (the workspace's `sha2` is newer than the `hmac` crate supports).
fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut k = [0u8; BLOCK];
    if key.len() > BLOCK {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let pad = |b: u8| k.map(|x| x ^ b);
    let mut inner = Sha256::new();
    inner.update(pad(0x36));
    inner.update(msg);
    let mut outer = Sha256::new();
    outer.update(pad(0x5c));
    outer.update(inner.finalize());
    outer.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_matches_rfc4231_case_2() {
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex::encode(mac),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }
}
