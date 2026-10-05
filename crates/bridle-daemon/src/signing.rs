//! Stable code signing for the installed `bridle` on macOS (ticket p88z). An ad-hoc signature
//! (what the linker adds, `.cargo/config.toml`) is a new identity on every build, so the
//! application firewall forgets its "Allow" after each rebuild. Signing with one self-signed
//! certificate keeps the designated requirement, and so the Allow, the same across rebuilds.
//! The identity is optional: with none in the keychain (CI, other machines) the ad-hoc
//! signature stays.

use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub const DEFAULT_IDENTITY: &str = "bridle local signing";
const IDENTITY_ENV: &str = "BRIDLE_SIGNING_IDENTITY";
const KEYCHAIN: &str = "login.keychain-db";

/// The identity to sign with: `$BRIDLE_SIGNING_IDENTITY`, else [`DEFAULT_IDENTITY`].
pub fn identity_name() -> String {
    std::env::var(IDENTITY_ENV)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_IDENTITY.to_string())
}

/// Whether `security find-identity` output lists an identity called `name`. Lines look like
/// `  1) <sha1> "bridle local signing"`.
pub fn lists_identity(find_identity_output: &str, name: &str) -> bool {
    let quoted = format!("\"{name}\"");
    find_identity_output
        .lines()
        .any(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()) && l.contains(&quoted))
}

/// Whether the login keychain holds the identity. Without `-v`, so a certificate macOS does
/// not (yet) trust still counts: `codesign` signs with it all the same.
pub fn identity_exists(name: &str) -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }
    Command::new("security")
        .args(["find-identity", "-p", "codesigning"])
        .output()
        .map(|o| lists_identity(&String::from_utf8_lossy(&o.stdout), name))
        .unwrap_or(false)
}

/// Re-signs `path` with the identity when it exists. `Ok(true)` if it did, `Ok(false)` when
/// there is no identity and the ad-hoc signature stays.
pub fn sign(path: &Path) -> Result<bool> {
    let name = identity_name();
    if !identity_exists(&name) {
        return Ok(false);
    }
    let out = Command::new("codesign")
        .args(["--force", "-s", &name])
        .arg(path)
        .output()
        .context("running codesign")?;
    if !out.status.success() {
        bail!(
            "codesign with \"{name}\" failed ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(true)
}

/// One-time, idempotent: creates the self-signed code-signing certificate in the login
/// keychain and lets `codesign` use its key without a prompt (so launchd-started daemons can
/// sign). Uses only the `security` and `openssl` CLIs, so it works over SSH. `password` is the
/// login keychain's, needed once for `set-key-partition-list`. Returns progress lines.
pub fn setup(password: &str) -> Result<Vec<String>> {
    if !cfg!(target_os = "macos") {
        bail!("signing setup is macOS only");
    }
    let name = identity_name();
    let mut said = Vec::new();
    if identity_exists(&name) {
        said.push(format!(
            "identity \"{name}\" already exists; refreshing key access"
        ));
    } else {
        create_identity(&name, password)?;
        said.push(format!("created identity \"{name}\" in the login keychain"));
    }
    let o = Command::new("security")
        .args([
            "set-key-partition-list",
            "-S",
            "apple-tool:,apple:,codesign:",
            "-s",
            "-k",
        ])
        .arg(password)
        .arg(KEYCHAIN)
        .output()
        .context("running security set-key-partition-list")?;
    if !o.status.success() {
        bail!(
            "set-key-partition-list failed (wrong keychain password?): {}",
            String::from_utf8_lossy(&o.stderr).trim()
        );
    }
    said.push("codesign may use the key without a prompt".into());
    Ok(said)
}

fn create_identity(name: &str, password: &str) -> Result<()> {
    let dir = tempfile::tempdir().context("temp dir")?;
    let (key, cert, p12) = (
        dir.path().join("key.pem"),
        dir.path().join("cert.pem"),
        dir.path().join("id.p12"),
    );
    let conf = dir.path().join("openssl.cnf");
    std::fs::write(
        &conf,
        format!(
            "[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN={name}\n\
             [ext]\nbasicConstraints=critical,CA:false\nkeyUsage=critical,digitalSignature\n\
             extendedKeyUsage=critical,codeSigning\n"
        ),
    )?;
    run(
        Command::new("openssl")
            .args([
                "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "3650",
            ])
            .arg("-config")
            .arg(&conf)
            .arg("-keyout")
            .arg(&key)
            .arg("-out")
            .arg(&cert),
        "openssl req",
    )?;
    run(
        Command::new("openssl")
            .args(["pkcs12", "-export", "-inkey"])
            .arg(&key)
            .arg("-in")
            .arg(&cert)
            .arg("-out")
            .arg(&p12)
            .arg("-passout")
            .arg(format!("pass:{password}")),
        "openssl pkcs12",
    )?;
    run(
        Command::new("security").arg("import").arg(&p12).args([
            "-k",
            KEYCHAIN,
            "-P",
            password,
            "-T",
            "/usr/bin/codesign",
        ]),
        "security import",
    )?;
    // Trust is best effort: codesign signs with an untrusted self-signed cert, and the
    // user-domain trust setting can demand a GUI authorization that SSH cannot give.
    let _ = Command::new("security")
        .args(["add-trusted-cert", "-p", "codeSign", "-k", KEYCHAIN])
        .arg(&cert)
        .stdin(Stdio::null())
        .output();
    Ok(())
}

fn run(cmd: &mut Command, what: &str) -> Result<()> {
    let o = cmd.output().with_context(|| format!("running {what}"))?;
    if !o.status.success() {
        bail!(
            "{what} failed ({}): {}",
            o.status,
            String::from_utf8_lossy(&o.stderr).trim()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = "  Policy: Code Signing\n  Matching identities\n  \
        1) 0123456789ABCDEF0123456789ABCDEF01234567 \"bridle local signing\" (CSSMERR_TP_NOT_TRUSTED)\n  \
        2) 89ABCDEF0123456789ABCDEF0123456789ABCDEF \"Apple Development: x\"\n     2 identities found\n";

    #[test]
    fn identity_listed_or_not() {
        assert!(lists_identity(LISTING, "bridle local signing"));
        assert!(!lists_identity(LISTING, "other"));
        assert!(!lists_identity(
            "     0 identities found\n",
            "bridle local signing"
        ));
        // A name only mentioned in prose does not count.
        assert!(!lists_identity(
            "note: \"bridle local signing\" missing",
            "bridle local signing"
        ));
    }
}
