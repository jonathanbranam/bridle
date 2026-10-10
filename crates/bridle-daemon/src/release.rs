//! `[daemon] self_upgrade = "release"` (docs/design/agent-host/daemon.md, "Upgrade"): find the
//! newest GitHub release, download this platform's tarball, check it against the release's
//! `SHA256SUMS`, and swap the binary in. Asset names come from the release workflow
//! (`scripts/package-release.sh`): `bridle-<tag>-<target>.tar.gz`. The network sits behind
//! [`ReleaseSource`] so tests never reach GitHub; the real one shells out to `curl`, which the
//! hosts have and which brings the TLS the daemon's HTTP client is built without.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::paths::Workspace;

/// The least time between two looks at GitHub (the unauthenticated API allows 60 an hour per IP).
pub const POLL_EVERY: Duration = Duration::from_secs(30 * 60);
const FETCH_TIMEOUT_SECS: &str = "300";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub assets: Vec<Asset>,
}

/// Blocking: callers run it under `spawn_blocking`.
pub trait ReleaseSource: Send + Sync {
    /// The newest published (non-draft, non-prerelease) release of `repo` (`owner/name`).
    fn latest(&self, repo: &str) -> Result<Release, String>;
    fn download(&self, url: &str) -> Result<Vec<u8>, String>;
}

impl fmt::Debug for dyn ReleaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReleaseSource")
    }
}

pub struct CurlSource;

fn curl(url: &str) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            FETCH_TIMEOUT_SECS,
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: bridle",
            url,
        ])
        .output()
        .map_err(|e| format!("running curl: {e}"))?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    // -f turns a 403/429 (rate limit) into a plain failure; the poll interval is the backoff.
    Err(format!(
        "fetching {url}: {} ({})",
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

impl ReleaseSource for CurlSource {
    fn latest(&self, repo: &str) -> Result<Release, String> {
        let body = curl(&format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ))?;
        parse_release(&body)
    }

    fn download(&self, url: &str) -> Result<Vec<u8>, String> {
        curl(url)
    }
}

pub fn parse_release(body: &[u8]) -> Result<Release, String> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| format!("reading the release: {e}"))?;
    let tag = v["tag_name"]
        .as_str()
        .ok_or("the release has no tag_name")?
        .to_string();
    let assets = v["assets"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| {
                    Some(Asset {
                        name: x["name"].as_str()?.to_string(),
                        url: x["browser_download_url"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Release { tag, assets })
}

/// `vX.Y.Z` or `X.Y.Z`; anything else (a prerelease suffix, a date tag) isn't comparable.
fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.strip_prefix('v').unwrap_or(s).split('.');
    let v = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(v)
}

/// Whether `tag` is a strictly newer version than `current`. An unparsable tag is never newer.
pub fn is_newer(tag: &str, current: &str) -> bool {
    match (parse_version(tag), parse_version(current)) {
        (Some(t), Some(c)) => t > c,
        _ => false,
    }
}

/// The release workflow's target triple for this machine.
pub fn this_target() -> Option<&'static str> {
    target_for(std::env::consts::OS, std::env::consts::ARCH)
}

fn target_for(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        _ => None,
    }
}

/// The tarball for `target` and the checksums file.
fn select_assets<'a>(r: &'a Release, target: &str) -> Result<(&'a Asset, &'a Asset), String> {
    let want = format!("bridle-{}-{target}.tar.gz", r.tag);
    let find = |name: &str| {
        r.assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| format!("release {} has no asset {name}", r.tag))
    };
    Ok((find(&want)?, find("SHA256SUMS")?))
}

/// The hex digest `sha256sum` listed for `name`.
fn expected_sha(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|l| {
        let (hash, file) = l.split_once(char::is_whitespace)?;
        (file.trim().trim_start_matches('*') == name).then(|| hash.to_ascii_lowercase())
    })
}

fn verify(bytes: &[u8], name: &str, sums: &str) -> Result<(), String> {
    let want = expected_sha(sums, name).ok_or_else(|| format!("SHA256SUMS doesn't list {name}"))?;
    let got = hex::encode(Sha256::digest(bytes));
    if got == want {
        Ok(())
    } else {
        Err(format!(
            "checksum mismatch for {name}: expected {want}, got {got}"
        ))
    }
}

/// Unpacks the tarball's `bridle` into `dir`.
fn unpack(tarball: &[u8], dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let file = dir.join("release.tar.gz");
    std::fs::write(&file, tarball).map_err(|e| format!("writing {}: {e}", file.display()))?;
    let out = Command::new("tar")
        .arg("-xzf")
        .arg(&file)
        .arg("-C")
        .arg(dir)
        .output()
        .map_err(|e| format!("running tar: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "unpacking the release: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let bin = dir.join("bridle");
    if bin.is_file() {
        Ok(bin)
    } else {
        Err("the release tarball has no bridle binary".to_string())
    }
}

/// Puts `new_bin` at `exe`: written beside it, then renamed over it, so the path never holds a
/// partial binary. The running binary is kept first for rollback (`rollback.rs`).
pub fn install(ws: &Workspace, new_bin: &Path, exe: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    crate::rollback::stash_previous(ws, exe)
        .map_err(|e| format!("keeping the previous binary: {e:#}"))?;
    let mut name = exe
        .file_name()
        .ok_or("the install path has no name")?
        .to_os_string();
    name.push(".new");
    let staged = exe.with_file_name(name);
    let put = || -> std::io::Result<()> {
        std::fs::copy(new_bin, &staged)?;
        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
        std::fs::rename(&staged, exe)
    };
    put().map_err(|e| {
        let _ = std::fs::remove_file(&staged);
        format!("installing over {}: {e}", exe.display())
    })
}

/// Downloads, verifies and installs `release` for `target` over `exe`. Nothing is touched until
/// the checksum has passed.
pub fn fetch_and_install(
    src: &dyn ReleaseSource,
    ws: &Workspace,
    release: &Release,
    target: &str,
    exe: &Path,
) -> Result<(), String> {
    let (tarball, sums) = select_assets(release, target)?;
    let sums_text = String::from_utf8(src.download(&sums.url)?)
        .map_err(|_| "SHA256SUMS isn't text".to_string())?;
    let bytes = src.download(&tarball.url)?;
    verify(&bytes, &tarball.name, &sums_text)?;
    let dir = ws.state_dir().join("upgrade-release");
    let _ = std::fs::remove_dir_all(&dir);
    let result = unpack(&bytes, &dir).and_then(|bin| install(ws, &bin, exe));
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// `owner/name` from a GitHub remote URL (https or scp-style ssh).
pub fn github_slug(url: &str) -> Option<String> {
    let rest = url.trim().split_once("github.com")?.1;
    let path = rest.trim_start_matches([':', '/']).trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, name) = path.split_once('/')?;
    (!owner.is_empty() && !name.is_empty() && !name.contains('/'))
        .then(|| format!("{owner}/{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Stub {
        files: Vec<(String, Vec<u8>)>,
    }

    impl ReleaseSource for Stub {
        fn latest(&self, _: &str) -> Result<Release, String> {
            Err("unused".into())
        }
        fn download(&self, url: &str) -> Result<Vec<u8>, String> {
            self.files
                .iter()
                .find(|(u, _)| u == url)
                .map(|(_, b)| b.clone())
                .ok_or_else(|| format!("404 {url}"))
        }
    }

    fn tarball(content: &str) -> Vec<u8> {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("bridle"), content).expect("write");
        // A fixed mtime: two tarballs built a second apart must hash the same.
        std::fs::File::options()
            .write(true)
            .open(dir.path().join("bridle"))
            .and_then(|f| f.set_modified(std::time::UNIX_EPOCH))
            .expect("mtime");
        let out = dir.path().join("o.tar.gz");
        let st = Command::new("tar")
            .arg("-C")
            .arg(dir.path())
            .arg("-czf")
            .arg(&out)
            .arg("bridle")
            .status()
            .expect("tar");
        assert!(st.success());
        std::fs::read(out).expect("read")
    }

    fn setup(sums_hash_of: &[u8]) -> (Stub, Release) {
        let tar = tarball("new binary");
        let name = "bridle-v9.9.9-aarch64-apple-darwin.tar.gz";
        let sums = format!("{}  {name}\n", hex::encode(Sha256::digest(sums_hash_of)));
        let release = Release {
            tag: "v9.9.9".into(),
            assets: vec![
                Asset {
                    name: name.into(),
                    url: "u/tar".into(),
                },
                Asset {
                    name: "SHA256SUMS".into(),
                    url: "u/sums".into(),
                },
            ],
        };
        let stub = Stub {
            files: vec![("u/tar".into(), tar), ("u/sums".into(), sums.into_bytes())],
        };
        (stub, release)
    }

    fn ws(dir: &Path) -> Workspace {
        Workspace::new(dir.join("repo"), Some(dir.to_path_buf()))
    }

    #[test]
    fn versions_compare_numerically() {
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(is_newer("v1.0.0", "0.4.0"));
        assert!(!is_newer("v0.4.0", "0.4.0"));
        assert!(!is_newer("v0.3.9", "0.4.0"));
        assert!(!is_newer("v0.5.0-rc1", "0.4.0"));
        assert!(!is_newer("nightly", "0.4.0"));
    }

    #[test]
    fn asset_chosen_per_platform() {
        let (_, release) = setup(b"");
        assert_eq!(target_for("macos", "aarch64"), Some("aarch64-apple-darwin"));
        assert_eq!(
            target_for("linux", "x86_64"),
            Some("x86_64-unknown-linux-gnu")
        );
        assert_eq!(target_for("windows", "x86_64"), None);
        let (t, s) = select_assets(&release, "aarch64-apple-darwin").expect("assets");
        assert_eq!((t.url.as_str(), s.url.as_str()), ("u/tar", "u/sums"));
        let err = select_assets(&release, "x86_64-unknown-linux-gnu").expect_err("no linux asset");
        assert!(
            err.contains("bridle-v9.9.9-x86_64-unknown-linux-gnu.tar.gz"),
            "{err}"
        );
    }

    #[test]
    fn parses_github_release_json_and_remote_urls() {
        let r = parse_release(
            br#"{"tag_name":"v1.2.3","assets":[{"name":"a","browser_download_url":"http://x/a"}]}"#,
        )
        .expect("parse");
        assert_eq!(r.tag, "v1.2.3");
        assert_eq!(r.assets[0].url, "http://x/a");
        for u in [
            "https://github.com/o/bridle.git",
            "git@github.com:o/bridle.git",
            "https://github.com/o/bridle",
        ] {
            assert_eq!(github_slug(u).as_deref(), Some("o/bridle"), "{u}");
        }
        assert_eq!(github_slug("/local/path"), None);
    }

    #[test]
    fn installs_and_keeps_the_previous_binary() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ws(tmp.path());
        std::fs::create_dir_all(ws.state_dir()).expect("state dir");
        let exe = tmp.path().join("bridle");
        std::fs::write(&exe, "old binary").expect("write");
        let (stub, release) = setup(&tarball("new binary"));
        fetch_and_install(&stub, &ws, &release, "aarch64-apple-darwin", &exe).expect("install");
        assert_eq!(std::fs::read_to_string(&exe).expect("read"), "new binary");
        let prev = std::fs::read_to_string(ws.state_dir().join("bridle.prev")).expect("prev");
        assert_eq!(prev, "old binary");
        assert!(!tmp.path().join("bridle.new").exists());
    }

    #[test]
    fn checksum_mismatch_leaves_the_binary_alone() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ws(tmp.path());
        std::fs::create_dir_all(ws.state_dir()).expect("state dir");
        let exe = tmp.path().join("bridle");
        std::fs::write(&exe, "old binary").expect("write");
        let (stub, release) = setup(b"something else");
        let err = fetch_and_install(&stub, &ws, &release, "aarch64-apple-darwin", &exe)
            .expect_err("mismatch");
        assert!(err.contains("checksum mismatch"), "{err}");
        assert_eq!(std::fs::read_to_string(&exe).expect("read"), "old binary");
        assert!(!ws.state_dir().join("bridle.prev").exists());
    }
}
