//! Stable code signing for the installed `bridle` on macOS (ticket p88z). An ad-hoc signature
//! (what the linker adds, `.cargo/config.toml`) is a new identity on every build, so the
//! application firewall forgets its "Allow" after each rebuild. Signing with one self-signed
//! certificate keeps the designated requirement, and so the Allow, the same across rebuilds.
//! The identity is optional: with none in the keychain (CI, other machines) the ad-hoc
//! signature stays.

use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// Choose which openssl binary to use: /usr/bin/openssl (macOS system LibreSSL) when it exists,
/// else "openssl" (PATH lookup). On macOS with Homebrew OpenSSL 3+, the Homebrew binary's
/// pkcs12 -export defaults are unreadable by macOS security import (MAC verification failed).
fn pick_openssl(exists: bool) -> &'static str {
    if exists {
        "/usr/bin/openssl"
    } else {
        "openssl"
    }
}

fn openssl_path() -> &'static str {
    pick_openssl(Path::new("/usr/bin/openssl").exists())
}

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

/// The identifier every installed bridle is signed with. The firewall's entry is keyed on
/// `identifier bridle and certificate leaf = ...`; `codesign` otherwise takes the identifier
/// from the file name (a copy, a `.new` file) and the linker's signature uses `bridle-<hash>`,
/// and a launch with another identifier is blocked until a dialog is answered (ticket 7bs2).
pub const IDENTIFIER: &str = "bridle";

/// Re-signs the installed `path` in place with the identity when it exists, through
/// [`install`] so the live path never holds a half-signed file. `Ok(true)` if it did, `Ok(false)`
/// when there is no identity and the file stays as it is.
pub fn sign(path: &Path) -> Result<bool> {
    let name = identity_name();
    if !identity_exists(&name) {
        return Ok(false);
    }
    install_with(Some(&name), path, path)?;
    Ok(true)
}

/// Puts `src` at `dest` atomically: signs a copy named `bridle` in a temp dir beside `dest`
/// (same filesystem) with [`IDENTIFIER`], checks the copy carries that identifier, then renames
/// it over `dest`. `dest` is therefore always a complete binary, never linker-signed, unsigned
/// or misnamed in between, whatever starts it (launchd KeepAlive, an agent). With no identity the
/// copy keeps its ad-hoc signature and is renamed all the same. `Ok(true)` when signed.
pub fn install(src: &Path, dest: &Path) -> Result<bool> {
    let name = identity_name();
    let id = identity_exists(&name).then_some(name);
    install_with(id.as_deref(), src, dest)?;
    Ok(id.is_some())
}

/// [`install`] with the identity chosen (`"-"` is ad hoc; `None` copies without signing).
fn install_with(identity: Option<&str>, src: &Path, dest: &Path) -> Result<()> {
    let parent = dest
        .parent()
        .context("the install path has no parent directory")?;
    let work = tempfile::Builder::new()
        .prefix(".bridle-sign-")
        .tempdir_in(parent)
        .with_context(|| format!("making a temp dir in {}", parent.display()))?;
    let copy = work.path().join(IDENTIFIER);
    std::fs::copy(src, &copy).with_context(|| format!("copying {}", src.display()))?;
    if let Some(name) = identity {
        let out = Command::new("codesign")
            .args(["--force", "-i", IDENTIFIER, "-s", name])
            .arg(&copy)
            .output()
            .context("running codesign")?;
        if !out.status.success() {
            bail!(
                "codesign with \"{name}\" failed ({}): {}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        let got = describe(&copy);
        if identifier_of(&got.verbose) != Some(IDENTIFIER) {
            bail!(
                "the signed copy's identifier is not \"{IDENTIFIER}\" ({}); not installing it",
                got.verbose.trim()
            );
        }
        if dest.exists() {
            let live = describe(dest);
            if live.requirement != got.requirement {
                tracing::warn!(
                    live = %live.requirement,
                    new = %got.requirement,
                    "the new binary's designated requirement differs from the installed one; \
                     the firewall's Allow will not carry over"
                );
            }
        }
    }
    std::fs::rename(&copy, dest).with_context(|| format!("renaming into {}", dest.display()))?;
    Ok(())
}

/// What `codesign` says about a file: the designated requirement and the verbose block.
pub struct Described {
    pub requirement: String,
    pub verbose: String,
}

/// `codesign -dr-` and `-dvv` of `path`, for the log and the checks; never fails (an error
/// reads as the text).
pub fn describe(path: &Path) -> Described {
    let run = |args: &[&str]| {
        Command::new("codesign")
            .args(args)
            .arg(path)
            .output()
            .map(|o| {
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                )
            })
            .unwrap_or_else(|e| format!("codesign: {e}"))
    };
    let dr = run(&["-d", "-r-"]);
    let requirement = dr
        .lines()
        .find_map(|l| l.strip_prefix("designated => "))
        .unwrap_or(dr.trim())
        .to_string();
    Described {
        requirement,
        verbose: run(&["-dvv"]),
    }
}

/// The `Identifier=` line of `codesign -dv` output.
pub fn identifier_of(verbose: &str) -> Option<&str> {
    verbose.lines().find_map(|l| l.strip_prefix("Identifier="))
}

/// Logs the installed binary's signature just before a restart, so the next firewall outage
/// names the launch that did not match (ticket 7bs2).
pub fn log_signature(exe: &Path) {
    if !cfg!(target_os = "macos") {
        return;
    }
    let d = describe(exe);
    tracing::info!(
        exe = %exe.display(),
        identifier = identifier_of(&d.verbose).unwrap_or("?"),
        requirement = %d.requirement,
        "installed binary signature before restart"
    );
}

/// `(pid, label)` of the running `dev.bridle.*` jobs in `launchctl list` output
/// (`PID<TAB>status<TAB>label`; a `-` pid is not running).
pub fn running_bridle_jobs(launchctl_list: &str) -> Vec<(u32, String)> {
    launchctl_list
        .lines()
        .filter_map(|l| {
            let mut f = l.split_whitespace();
            let pid = f.next()?.parse().ok()?;
            f.next()?;
            let label = f.next()?;
            label
                .starts_with("dev.bridle.")
                .then(|| (pid, label.to_string()))
        })
        .collect()
}

/// Restarts the launchd jobs that run `exe` (gateway, mail bridge, ...) except this process,
/// which execs itself. Called after the new binary is in place, so they start from it. Returns
/// the labels restarted. macOS only; failures are logged, not fatal.
pub fn restart_launchd_jobs(exe: &Path) -> Vec<String> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }
    let Ok(list) = Command::new("launchctl").arg("list").output() else {
        return Vec::new();
    };
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let mut done = Vec::new();
    for (pid, label) in running_bridle_jobs(&String::from_utf8_lossy(&list.stdout)) {
        if pid == std::process::id() {
            continue;
        }
        let comm = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if Path::new(&comm) != exe {
            continue;
        }
        match Command::new("launchctl")
            .args(["kickstart", "-k", &format!("gui/{uid}/{label}")])
            .output()
        {
            Ok(o) if o.status.success() => done.push(label),
            Ok(o) => {
                tracing::warn!(%label, "launchctl kickstart: {}", String::from_utf8_lossy(&o.stderr).trim())
            }
            Err(e) => tracing::warn!(%label, "launchctl kickstart: {e}"),
        }
    }
    done
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
        Command::new(openssl_path())
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
        Command::new(openssl_path())
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

    #[test]
    fn identifier_is_read_from_verbose_output() {
        let v = "Executable=/x/bridle\nIdentifier=bridle\nFormat=Mach-O\n";
        assert_eq!(identifier_of(v), Some("bridle"));
        assert_eq!(
            identifier_of("Identifier=bridle-8a9cb0\n"),
            Some("bridle-8a9cb0")
        );
        assert_eq!(identifier_of("nothing"), None);
    }

    #[test]
    fn launchctl_list_gives_running_bridle_jobs() {
        let out = "PID\tStatus\tLabel\n46148\t1\tdev.bridle.gateway\n-\t0\tdev.bridle.up\n\
                   97498\t-15\tdev.bridle.mail.bridle\n662\t0\tcom.other\n";
        assert_eq!(
            running_bridle_jobs(out),
            vec![
                (46148, "dev.bridle.gateway".to_string()),
                (97498, "dev.bridle.mail.bridle".to_string())
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn install_without_identity_replaces_atomically_and_leaves_no_temp() {
        let t = tempfile::tempdir().unwrap();
        let (src, dest) = (t.path().join("built-elsewhere"), t.path().join("bridle"));
        std::fs::write(&src, "new").unwrap();
        std::fs::write(&dest, "old").unwrap();
        install_with(None, &src, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "new");
        let left: Vec<_> = std::fs::read_dir(t.path()).unwrap().flatten().collect();
        assert_eq!(left.len(), 2, "only src and dest remain");
    }

    /// Whatever the source file is called, the installed file's identifier is `bridle`.
    #[cfg(target_os = "macos")]
    #[test]
    fn install_signs_with_identifier_bridle_regardless_of_file_name() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("some-other-name");
        std::fs::copy("/usr/bin/true", &src).unwrap();
        let dest = t.path().join("bridle");
        // "-" is ad hoc: no keychain identity needed, same -i path as the real one.
        install_with(Some("-"), &src, &dest).unwrap();
        assert_eq!(identifier_of(&describe(&dest).verbose), Some("bridle"));
        // Replacing an existing, differently signed file works too.
        install_with(Some("-"), &src, &dest).unwrap();
        assert_eq!(identifier_of(&describe(&dest).verbose), Some("bridle"));
    }

    #[test]
    fn choose_openssl_binary() {
        assert_eq!(pick_openssl(true), "/usr/bin/openssl");
        assert_eq!(pick_openssl(false), "openssl");
    }
}
