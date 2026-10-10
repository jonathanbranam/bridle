//! The daemon's own copy of the base workflow, at the binary's tag (docs/design/agent-host/
//! daemon.md, "Upgrade"; ticket chvf step 3). A `self_upgrade = "release"` daemon keeps
//! `<home>/workflow/vX.Y.Z/` (a shallow clone of that tag; read-only by convention) and points the
//! base layer at its `workflow/` directory, so a workflow never names a command the binary lacks
//! and a `git pull` elsewhere can't change agents' workflow under it. An explicit `workflow` path
//! (machine, project or vendored) always wins; this is only the default for release daemons.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Checkouts kept: the running tag and the one before it, for the fallback.
const KEEP: usize = 2;

fn root(home: &Path) -> PathBuf {
    home.join("workflow")
}

pub fn tag_for(version: &str) -> String {
    format!("v{}", version.trim_start_matches('v'))
}

fn parse_tag(name: &str) -> Option<(u64, u64, u64)> {
    let mut p = name.strip_prefix('v')?.split('.');
    let v = (
        p.next()?.parse().ok()?,
        p.next()?.parse().ok()?,
        p.next()?.parse().ok()?,
    );
    p.next().is_none().then_some(v)
}

/// Checked-out tags, newest first, that have a usable `workflow/base`.
fn checkouts(home: &Path) -> Vec<((u64, u64, u64), String)> {
    let mut tags: Vec<_> = std::fs::read_dir(root(home))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let v = parse_tag(&name)?;
            e.path().join("workflow/base").is_dir().then_some((v, name))
        })
        .collect();
    tags.sort_by(|a, b| b.cmp(a));
    tags
}

/// The `workflow` directory to use for a daemon running `version`: its own tag's checkout, else
/// the newest older one (a failed fetch falls back to the previous tag), else none.
pub fn managed_root(home: &Path, version: &str) -> Option<PathBuf> {
    let want = parse_tag(&tag_for(version))?;
    let (_, name) = checkouts(home).into_iter().find(|(v, _)| *v <= want)?;
    Some(root(home).join(name).join("workflow"))
}

/// Whether `version`'s own checkout is there.
pub fn has(home: &Path, version: &str) -> bool {
    root(home)
        .join(tag_for(version))
        .join("workflow/base")
        .is_dir()
}

/// Whether `workflow` (as `Config` holds it) is unset or already one of ours, i.e. nothing
/// explicit is in the way of the managed checkout.
pub fn is_unpinned(home: &Path, workflow: Option<&str>) -> bool {
    workflow.is_none_or(|w| Path::new(w).starts_with(root(home)))
}

/// Shallow-clones `tag` of `url` into `<home>/workflow/<tag>` unless it's already there, then
/// drops all but the newest [`KEEP`] checkouts. The clone lands in a scratch directory first so a
/// failed or interrupted fetch never leaves a half checkout under a tag's name.
pub fn ensure(home: &Path, url: &str, tag: &str) -> Result<(), String> {
    let dest = root(home).join(tag);
    if !dest.join("workflow/base").is_dir() {
        std::fs::create_dir_all(root(home)).map_err(|e| format!("creating workflow dir: {e}"))?;
        let scratch = root(home).join(format!(".fetching-{tag}"));
        let _ = std::fs::remove_dir_all(&scratch);
        let _ = std::fs::remove_dir_all(&dest);
        let out = Command::new("git")
            .args(["clone", "--quiet", "--depth", "1", "--branch", tag, url])
            .arg(&scratch)
            .output()
            .map_err(|e| format!("running git: {e}"))?;
        let result = if !out.status.success() {
            Err(format!(
                "fetching workflow {tag} from {url}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        } else if !scratch.join("workflow/base").is_dir() {
            Err(format!("{url} at {tag} has no workflow/base"))
        } else {
            let _ = std::fs::remove_dir_all(scratch.join(".git"));
            std::fs::rename(&scratch, &dest).map_err(|e| format!("placing {tag}: {e}"))
        };
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&scratch);
        }
        result?;
    }
    prune(home);
    Ok(())
}

fn prune(home: &Path) {
    for (_, name) in checkouts(home).into_iter().skip(KEEP) {
        let _ = std::fs::remove_dir_all(root(home).join(name));
    }
}

/// Where to fetch from: the machine's `workflow_url`, else the GitHub repo releases come from.
pub fn url_for(home: &Path, repo_slug: Option<&str>) -> Option<String> {
    crate::config::Config::machine_workflow_url(Some(home))
        .ok()
        .flatten()
        .or_else(|| repo_slug.map(|s| format!("https://github.com/{s}.git")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, a: &[&str]) {
        let st = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(a)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .expect("git");
        assert!(st.success(), "git {a:?}");
    }

    /// A bare origin with `workflow/base/rules/a.md` tagged v0.1.0 .. v0.3.0 (content = the tag).
    fn origin(tags: &[&str]) -> (tempfile::TempDir, String) {
        let d = tempfile::tempdir().expect("tmp");
        let work = d.path().join("work");
        std::fs::create_dir_all(work.join("workflow/base/rules")).expect("mkdir");
        git(&work, &["init", "-q", "-b", "main"]);
        for t in tags {
            std::fs::write(work.join("workflow/base/rules/a.md"), t).expect("write");
            git(&work, &["add", "-A"]);
            git(&work, &["commit", "-q", "-m", t]);
            git(&work, &["tag", t]);
        }
        let bare = d.path().join("origin.git");
        let st = Command::new("git")
            .args(["clone", "-q", "--bare"])
            .arg(&work)
            .arg(&bare)
            .status()
            .expect("clone");
        assert!(st.success());
        let url = bare.to_string_lossy().into_owned();
        (d, url)
    }

    fn read(p: PathBuf) -> String {
        std::fs::read_to_string(p.join("base/rules/a.md")).expect("rule")
    }

    #[test]
    fn fetches_a_tag_and_points_the_base_layer_at_it() {
        let (_o, url) = origin(&["v0.1.0", "v0.2.0"]);
        let home = tempfile::tempdir().expect("home");
        assert_eq!(managed_root(home.path(), "0.2.0"), None);
        ensure(home.path(), &url, "v0.2.0").expect("ensure");
        assert!(has(home.path(), "0.2.0"));
        let root = managed_root(home.path(), "0.2.0").expect("root");
        assert_eq!(read(root), "v0.2.0");
        assert!(!home.path().join("workflow/v0.2.0/.git").exists());
    }

    #[test]
    fn missing_checkout_falls_back_to_the_previous_tag() {
        let (_o, url) = origin(&["v0.1.0"]);
        let home = tempfile::tempdir().expect("home");
        ensure(home.path(), &url, "v0.1.0").expect("ensure");
        let err = ensure(home.path(), &url, "v0.2.0").expect_err("no such tag");
        assert!(err.contains("v0.2.0"), "{err}");
        assert!(!has(home.path(), "0.2.0"));
        assert!(!root(home.path()).join(".fetching-v0.2.0").exists());
        let root = managed_root(home.path(), "0.2.0").expect("fallback");
        assert_eq!(read(root), "v0.1.0");
        // A newer checkout than the running binary (after a rollback) isn't used.
        assert_eq!(managed_root(home.path(), "0.0.9"), None);
    }

    #[test]
    fn keeps_the_last_two_tags() {
        let (_o, url) = origin(&["v0.1.0", "v0.2.0", "v0.10.0"]);
        let home = tempfile::tempdir().expect("home");
        for t in ["v0.1.0", "v0.2.0", "v0.10.0"] {
            ensure(home.path(), &url, t).expect("ensure");
        }
        let left: Vec<_> = checkouts(home.path()).into_iter().map(|c| c.1).collect();
        assert_eq!(left, ["v0.10.0", "v0.2.0"]);
    }

    #[test]
    fn an_explicit_workflow_path_wins() {
        let home = Path::new("/h");
        assert!(is_unpinned(home, None));
        assert!(is_unpinned(home, Some("/h/workflow/v0.1.0/workflow")));
        assert!(!is_unpinned(home, Some("/code/bridle/workflow")));
        assert!(!is_unpinned(home, Some(".bridle/workflow")));
    }
}
