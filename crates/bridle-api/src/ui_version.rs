//! Which bridle-ui build is installed and how far it is behind bridle-ui main (cyzr). Local
//! only: no network. Main is read from this machine's bridle-ui clone, found through the
//! daemon registry; without one, "behind" is unknown and says why.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::{UiSource, UiVersion};

/// The UI folder: `[gateway] ui_dir` in `<home>/config.toml`, else `<home>/ui`.
pub fn ui_dir(home: &Path) -> PathBuf {
    std::fs::read_to_string(home.join("config.toml"))
        .ok()
        .and_then(|t| t.parse::<toml::Table>().ok())
        .and_then(|t| Some(PathBuf::from(t.get("gateway")?.get("ui_dir")?.as_str()?)))
        .unwrap_or_else(|| home.join("ui"))
}

/// Reads `<dir>/bridle-version` (`<tag> bridle-ui <sha>`, written by release.yml). Runs git, so
/// async callers wrap it in `spawn_blocking`.
pub fn read(dir: &Path, home: &Path) -> UiVersion {
    let mut v = UiVersion {
        source: UiSource::Missing,
        release: None,
        commit: None,
        behind: None,
        note: None,
    };
    let file = dir.join("bridle-version");
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // `npm run install-ui` copies a plain build, which has no bridle-version.
            if dir.join("index.html").is_file() || dir.join("api-version").is_file() {
                v.source = UiSource::Other;
            }
            return v;
        }
        Err(e) => {
            v.source = UiSource::Unreadable;
            v.note = Some(format!("{}: {e}", file.display()));
            return v;
        }
    };
    match text.split_whitespace().collect::<Vec<_>>()[..] {
        [release, "bridle-ui", commit]
            if commit.len() >= 4 && commit.chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            v.source = UiSource::Release;
            v.release = Some(release.to_string());
            v.commit = Some(commit.to_string());
            match behind_main(home, commit) {
                Ok(n) => v.behind = Some(n),
                Err(why) => v.note = Some(why),
            }
        }
        _ => {
            v.source = UiSource::Unreadable;
            v.note = Some(format!(
                "{}: not `<tag> bridle-ui <sha>`: {:?}",
                file.display(),
                text.trim().chars().take(80).collect::<String>()
            ));
        }
    }
    v
}

/// Commits on the local bridle-ui clone's main that `commit` lacks.
fn behind_main(home: &Path, commit: &str) -> Result<u32, String> {
    // The file is read directly: listing the registry would prune it, and a stopped daemon's
    // clone is still there.
    let repo = std::fs::read(home.join("daemons").join("bridle-ui.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|j| j.get("repo")?.as_str().map(PathBuf::from))
        .ok_or("no bridle-ui project registered on this machine")?;
    let out = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["rev-list", "--count"])
        .arg(format!("{commit}..refs/heads/main"))
        .env_remove("GIT_DIR")
        .env("GIT_NO_LAZY_FETCH", "1")
        .output()
        .map_err(|e| format!("running git: {e}"))?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .ok()
        .filter(|_| out.status.success())
        .ok_or_else(|| {
            format!(
                "the clone at {} cannot compare {} with main",
                repo.display(),
                short(commit)
            )
        })
}

fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}

/// The text after `ui` in `bridle status`.
pub fn describe(v: &UiVersion) -> String {
    let note = v.note.as_deref().unwrap_or("no reason given");
    match v.source {
        UiSource::Missing => "none installed".to_string(),
        UiSource::Other => "installed without a bridle-version (not from a release, e.g. \
                            `npm run install-ui`); version and behind main unknown"
            .to_string(),
        UiSource::Unreadable => format!("version unreadable: {note}"),
        UiSource::Release => {
            let behind = match v.behind {
                Some(0) => "up to date with bridle-ui main".to_string(),
                Some(n) => format!("{n} commit(s) behind bridle-ui main"),
                None => format!("behind main unknown: {note}"),
            };
            format!(
                "{} bridle-ui {}, {behind}",
                v.release.as_deref().unwrap_or("?"),
                short(v.commit.as_deref().unwrap_or("?"))
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home_with_ui() -> (tempfile::TempDir, PathBuf) {
        let home = tempfile::tempdir().unwrap();
        let ui = home.path().join("ui");
        std::fs::create_dir(&ui).unwrap();
        std::fs::write(ui.join("index.html"), "").unwrap();
        (home, ui)
    }

    fn git(repo: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .env_remove("GIT_DIR")
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn no_ui_folder_and_a_build_without_a_version_file_differ() {
        let home = tempfile::tempdir().unwrap();
        let none = read(&home.path().join("ui"), home.path());
        assert_eq!(none.source, UiSource::Missing);
        let (home, ui) = home_with_ui();
        let other = read(&ui, home.path());
        assert_eq!(other.source, UiSource::Other);
        assert!(describe(&other).contains("install-ui"));
    }

    #[test]
    fn present_without_a_clone_is_behind_unknown() {
        let (home, ui) = home_with_ui();
        std::fs::write(ui.join("bridle-version"), "v0.8.0 bridle-ui b0f3dbe\n").unwrap();
        let v = read(&ui, home.path());
        assert_eq!(v.source, UiSource::Release);
        assert_eq!(v.release.as_deref(), Some("v0.8.0"));
        assert_eq!(v.commit.as_deref(), Some("b0f3dbe"));
        assert_eq!(v.behind, None);
        assert!(describe(&v).contains("unknown: no bridle-ui project"));
    }

    #[test]
    fn behind_counts_commits_on_the_clone_main() {
        let (home, ui) = home_with_ui();
        let repo = home.path().join("clone");
        std::fs::create_dir(&repo).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "a"]);
        let installed = git(&repo, &["rev-parse", "HEAD"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "b"]);
        std::fs::create_dir(home.path().join("daemons")).unwrap();
        std::fs::write(
            home.path().join("daemons/bridle-ui.json"),
            serde_json::json!({ "repo": repo }).to_string(),
        )
        .unwrap();
        std::fs::write(
            ui.join("bridle-version"),
            format!("v1 bridle-ui {installed}"),
        )
        .unwrap();
        assert_eq!(read(&ui, home.path()).behind, Some(1));
        std::fs::write(ui.join("bridle-version"), "v1 bridle-ui deadbeef").unwrap();
        let unknown = read(&ui, home.path());
        assert_eq!(unknown.behind, None);
        assert!(unknown.note.unwrap().contains("cannot compare"));
    }

    #[test]
    fn unreadable_or_garbled_version_files_say_so() {
        let (home, ui) = home_with_ui();
        // A directory fails to read even for root, unlike a mode 000 file.
        std::fs::create_dir(ui.join("bridle-version")).unwrap();
        let v = read(&ui, home.path());
        assert_eq!(v.source, UiSource::Unreadable);
        assert!(v.note.is_some());
        std::fs::remove_dir(ui.join("bridle-version")).unwrap();
        std::fs::write(ui.join("bridle-version"), "nonsense").unwrap();
        assert_eq!(read(&ui, home.path()).source, UiSource::Unreadable);
    }

    #[test]
    fn ui_dir_follows_the_gateway_config() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(ui_dir(home.path()), home.path().join("ui"));
        std::fs::write(
            home.path().join("config.toml"),
            "[gateway]\nui_dir = \"/x\"\n",
        )
        .unwrap();
        assert_eq!(ui_dir(home.path()), PathBuf::from("/x"));
    }
}
