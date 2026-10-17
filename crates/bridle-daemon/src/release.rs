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
    /// Puts the workflow at `tag` of `repo` in `<home>/workflow/<tag>` (chvf step 3). Sources
    /// with no git remote to fetch from do nothing.
    fn fetch_workflow(&self, _home: &Path, _repo: &str, _tag: &str) -> Result<(), String> {
        Ok(())
    }
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

    fn fetch_workflow(&self, home: &Path, repo: &str, tag: &str) -> Result<(), String> {
        let url = crate::workflow_checkout::url_for(home, repo);
        crate::workflow_checkout::ensure(home, &url, tag)
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
/// partial binary. The running binary is kept first for rollback (`rollback.rs`). On macOS the
/// copy is signed `-i bridle` first (`signing::install`), as the commit path does, so the
/// firewall's Allow survives the upgrade (ticket tvrp).
pub fn install(ws: &Workspace, new_bin: &Path, exe: &Path) -> Result<(), String> {
    // Not under test: a stand-in "binary" can't be codesigned on a Mac that has the identity.
    install_signed(cfg!(target_os = "macos") && !cfg!(test), ws, new_bin, exe)
}

/// [`install`] with the platform choice made by the caller, so tests can take either branch.
/// A failed sign leaves `exe` as it was (the rename comes last) and the stash for rollback.
fn install_signed(sign: bool, ws: &Workspace, new_bin: &Path, exe: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    crate::rollback::stash_previous(ws, exe)
        .map_err(|e| format!("keeping the previous binary: {e:#}"))?;
    if sign {
        let signed = crate::signing::install(new_bin, exe)
            .map_err(|e| format!("signing the release binary over {}: {e:#}", exe.display()))?;
        if !signed {
            tracing::warn!(
                "self-upgrade: no signing identity found; the release binary keeps its ad-hoc \
                 signature and the firewall may ask again (run `just sign-setup`)"
            );
        }
        return Ok(());
    }
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

/// Replaces `ui_dir` with the tarball's `ui/` (t3dr): copied to `<ui_dir>.tmp`, then renamed in,
/// so the gateway never serves a half-written folder. A tarball without `ui/` (a tag from before
/// the UI shipped) leaves the current folder alone.
fn install_ui(unpacked: &Path, ui_dir: &Path) -> Result<(), String> {
    let src = unpacked.join("ui");
    if !src.is_dir() {
        return Ok(());
    }
    let sibling = |suffix: &str| {
        let mut name = ui_dir.file_name().unwrap_or_default().to_os_string();
        name.push(suffix);
        ui_dir.with_file_name(name)
    };
    let (tmp, old) = (sibling(".tmp"), sibling(".old"));
    let fail = |what: &str, e: std::io::Error| format!("{what} {}: {e}", ui_dir.display());
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::remove_dir_all(&old);
    if let Some(parent) = ui_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| fail("creating the parent of", e))?;
    }
    let out = Command::new("cp")
        .arg("-R")
        .arg(&src)
        .arg(&tmp)
        .output()
        .map_err(|e| format!("running cp: {e}"))?;
    if !out.status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(format!(
            "copying the UI: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    // A directory can't be renamed over a non-empty one, so the old folder steps aside first.
    let had_old = ui_dir.exists();
    if had_old {
        std::fs::rename(ui_dir, &old).map_err(|e| fail("moving aside", e))?;
    }
    if let Err(e) = std::fs::rename(&tmp, ui_dir) {
        if had_old {
            let _ = std::fs::rename(&old, ui_dir);
        }
        return Err(fail("installing the UI at", e));
    }
    let _ = std::fs::remove_dir_all(&old);
    Ok(())
}

/// Downloads, verifies and installs `release` for `target` over `exe`, and the tarball's `ui/`
/// (if it has one) over `ui_dir`. Nothing is touched until the checksum has passed. A UI that
/// fails to install doesn't undo the binary: it is logged and the old UI stays.
pub fn fetch_and_install(
    src: &dyn ReleaseSource,
    ws: &Workspace,
    release: &Release,
    target: &str,
    exe: &Path,
    ui_dir: &Path,
) -> Result<(), String> {
    let (tarball, sums) = select_assets(release, target)?;
    let sums_text = String::from_utf8(src.download(&sums.url)?)
        .map_err(|_| "SHA256SUMS isn't text".to_string())?;
    let bytes = src.download(&tarball.url)?;
    verify(&bytes, &tarball.name, &sums_text)?;
    let dir = ws.state_dir().join("upgrade-release");
    let _ = std::fs::remove_dir_all(&dir);
    let result = unpack(&bytes, &dir).and_then(|bin| install(ws, &bin, exe));
    if result.is_ok()
        && let Err(e) = install_ui(&dir, ui_dir)
    {
        tracing::warn!(error = %e, "self-upgrade: the UI was not updated");
    }
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Bridle's own GitHub repo: where a release daemon looks for releases and the workflow unless
/// `[daemon] release_repo` says otherwise. Never the project's origin (ticket ajp5).
pub const BRIDLE_REPO: &str = "jonathanbranam/bridle";

/// The repo to take releases and the workflow from: the explicit setting, else bridle's own.
pub fn release_repo(configured: Option<&str>) -> String {
    configured.unwrap_or(BRIDLE_REPO).to_string()
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
        tarball_with_ui(content, None)
    }

    fn tarball_with_ui(content: &str, ui_marker: Option<&str>) -> Vec<u8> {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("bridle"), content).expect("write");
        let mut members = vec!["bridle"];
        if let Some(m) = ui_marker {
            std::fs::create_dir_all(dir.path().join("ui/assets")).expect("ui dir");
            std::fs::write(dir.path().join("ui/index.html"), m).expect("index");
            std::fs::write(dir.path().join("ui/assets/a.js"), m).expect("asset");
            members.push("ui");
        }
        // A fixed mtime for reproducible entries.
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
            .args(&members)
            .status()
            .expect("tar");
        assert!(st.success());
        std::fs::read(out).expect("read")
    }

    /// `sums_hash_of` is what SHA256SUMS vouches for; `None` means the tarball itself.
    /// (Building the tarball twice isn't safe: the gzip header carries a timestamp.)
    fn setup(sums_hash_of: Option<&[u8]>) -> (Stub, Release) {
        setup_with(tarball("new binary"), sums_hash_of)
    }

    fn setup_with(tar: Vec<u8>, sums_hash_of: Option<&[u8]>) -> (Stub, Release) {
        let sums_hash_of = sums_hash_of.unwrap_or(&tar);
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
        let (_, release) = setup(None);
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
        let (stub, release) = setup(None);
        let ui = tmp.path().join("ui");
        fetch_and_install(&stub, &ws, &release, "aarch64-apple-darwin", &exe, &ui)
            .expect("install");
        assert_eq!(std::fs::read_to_string(&exe).expect("read"), "new binary");
        assert!(!ui.exists(), "no ui/ in the tarball, no folder made");
        let prev = std::fs::read_to_string(ws.state_dir().join("bridle.prev")).expect("prev");
        assert_eq!(prev, "old binary");
        assert!(!tmp.path().join("bridle.new").exists());
    }

    #[test]
    fn a_failed_sign_leaves_the_installed_binary_and_the_stash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ws(tmp.path());
        std::fs::create_dir_all(ws.state_dir()).expect("state dir");
        let exe = tmp.path().join("bridle");
        std::fs::write(&exe, "old binary").expect("write");
        // The signing path fails here (no such source), as a codesign failure would.
        let err = install_signed(true, &ws, &tmp.path().join("missing"), &exe).expect_err("fails");
        assert!(err.contains("signing the release binary"), "{err}");
        assert_eq!(std::fs::read_to_string(&exe).expect("read"), "old binary");
        let prev = std::fs::read_to_string(ws.state_dir().join("bridle.prev")).expect("prev");
        assert_eq!(prev, "old binary");
    }

    #[test]
    fn without_signing_the_binary_is_copied_in() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ws(tmp.path());
        std::fs::create_dir_all(ws.state_dir()).expect("state dir");
        let exe = tmp.path().join("bridle");
        std::fs::write(&exe, "old").expect("write");
        let new = tmp.path().join("new");
        std::fs::write(&new, "new").expect("write");
        install_signed(false, &ws, &new, &exe).expect("install");
        assert_eq!(std::fs::read_to_string(&exe).expect("read"), "new");
    }

    #[test]
    fn checksum_mismatch_leaves_the_binary_alone() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ws = ws(tmp.path());
        std::fs::create_dir_all(ws.state_dir()).expect("state dir");
        let exe = tmp.path().join("bridle");
        std::fs::write(&exe, "old binary").expect("write");
        let (stub, release) = setup(Some(b"something else"));
        let ui = tmp.path().join("ui");
        let err = fetch_and_install(&stub, &ws, &release, "aarch64-apple-darwin", &exe, &ui)
            .expect_err("mismatch");
        assert!(err.contains("checksum mismatch"), "{err}");
        assert_eq!(std::fs::read_to_string(&exe).expect("read"), "old binary");
        assert!(!ws.state_dir().join("bridle.prev").exists());
    }

    fn upgrade_with_ui(tarball: Vec<u8>, tmp: &Path) -> PathBuf {
        let ws = ws(tmp);
        std::fs::create_dir_all(ws.state_dir()).expect("state dir");
        let exe = tmp.join("bridle");
        std::fs::write(&exe, "old binary").expect("write");
        let ui = tmp.join("home/ui");
        let (stub, release) = setup_with(tarball, None);
        fetch_and_install(&stub, &ws, &release, "aarch64-apple-darwin", &exe, &ui)
            .expect("install");
        ui
    }

    #[test]
    fn ui_from_the_tarball_replaces_the_old_folder() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ui = tmp.path().join("home/ui");
        std::fs::create_dir_all(&ui).expect("old ui");
        std::fs::write(ui.join("stale.js"), "old").expect("stale");
        let got = upgrade_with_ui(tarball_with_ui("new binary", Some("v2")), tmp.path());
        assert_eq!(got, ui);
        assert_eq!(
            std::fs::read_to_string(ui.join("index.html")).expect("index"),
            "v2"
        );
        assert_eq!(
            std::fs::read_to_string(ui.join("assets/a.js")).expect("asset"),
            "v2"
        );
        assert!(!ui.join("stale.js").exists());
        assert!(!tmp.path().join("home/ui.tmp").exists());
        assert!(!tmp.path().join("home/ui.old").exists());
    }

    #[test]
    fn ui_is_made_when_there_was_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ui = upgrade_with_ui(tarball_with_ui("new binary", Some("v1")), tmp.path());
        assert!(ui.join("index.html").is_file());
    }

    #[test]
    fn tarball_without_ui_leaves_the_folder_alone() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ui = tmp.path().join("home/ui");
        std::fs::create_dir_all(&ui).expect("old ui");
        std::fs::write(ui.join("index.html"), "mine").expect("index");
        upgrade_with_ui(tarball("new binary"), tmp.path());
        assert_eq!(
            std::fs::read_to_string(ui.join("index.html")).expect("index"),
            "mine"
        );
    }
}
