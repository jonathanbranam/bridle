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

/// The shallow clone of `tag`. No prompt ever: a failure is an error in the log, not a hang.
fn clone_command(url: &str, tag: &str, dest: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["clone", "--quiet", "--depth", "1", "--branch", tag, url])
        .arg(dest)
        .env("GIT_TERMINAL_PROMPT", "0");
    cmd
}

/// Shallow-clones `tag` of `url` into `<home>/workflow/<tag>` unless it's already there, then
/// drops all but the newest [`KEEP`] checkouts. The clone lands in a per-call scratch directory
/// first, then is renamed into place, so a failed or interrupted fetch never leaves a half checkout
/// under a tag's name. Several daemons on one machine may run this at once (ticket br-ysmy): each
/// has its own scratch dir, and if another one's rename won, the target is already complete and
/// that is success.
pub fn ensure(home: &Path, url: &str, tag: &str) -> Result<(), String> {
    let dest = root(home).join(tag);
    if !dest.join("workflow/base").is_dir() {
        std::fs::create_dir_all(root(home)).map_err(|e| format!("creating workflow dir: {e}"))?;
        let scratch = tempfile::Builder::new()
            .prefix(&format!(".fetching-{tag}-"))
            .tempdir_in(root(home))
            .map_err(|e| format!("making a scratch dir: {e}"))?;
        let clone = scratch.path().join("clone");
        let out = clone_command(url, tag, &clone)
            .output()
            .map_err(|e| format!("running git: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "fetching workflow {tag} from {url}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        if !clone.join("workflow/base").is_dir() {
            return Err(format!("{url} at {tag} has no workflow/base"));
        }
        let _ = std::fs::remove_dir_all(clone.join(".git"));
        // A rename onto a non-empty dir fails: either another daemon placed its checkout first, or
        // an incomplete dir sits under the tag name (an interrupted earlier run). The latter is
        // moved aside into our own scratch dir, so it is never half-deleted, then the rename is
        // retried once.
        let mut placed = std::fs::rename(&clone, &dest);
        if placed.is_err() && !dest.join("workflow/base").is_dir() {
            std::fs::rename(&dest, scratch.path().join("stale"))
                .map_err(|e| format!("moving aside the incomplete {tag}: {e}"))?;
            placed = std::fs::rename(&clone, &dest);
        }
        placed.or_else(|e| {
            if dest.join("workflow/base").is_dir() {
                Ok(())
            } else {
                Err(format!("placing {tag}: {e}"))
            }
        })?;
        // `scratch` drops here and removes what is left of its own temp dir.
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
pub fn url_for(home: &Path, repo_slug: &str) -> String {
    crate::config::Config::machine_workflow_url(Some(home))
        .ok()
        .flatten()
        .unwrap_or_else(|| format!("https://github.com/{repo_slug}.git"))
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
    fn clone_never_prompts() {
        let cmd = clone_command("u", "v1.0.0", Path::new("/d"));
        let env: Vec<_> = cmd.get_envs().collect();
        assert!(env.contains(&(
            std::ffi::OsStr::new("GIT_TERMINAL_PROMPT"),
            Some(std::ffi::OsStr::new("0"))
        )));
    }

    #[test]
    fn url_defaults_to_bridles_repo() {
        let home = tempfile::tempdir().expect("home");
        let slug = crate::release::release_repo(None);
        assert_eq!(
            url_for(home.path(), &slug),
            "https://github.com/jonathanbranam/bridle.git"
        );
        assert_eq!(crate::release::release_repo(Some("a/b")), "a/b");
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

    /// An incomplete dir under the tag name (an interrupted earlier run) is moved aside and
    /// replaced, not left to fail every later fetch (br-ysmy review).
    #[test]
    fn an_incomplete_dir_under_the_tag_is_replaced() {
        let (_o, url) = origin(&["v0.1.0"]);
        let home = tempfile::tempdir().expect("home");
        let stale = root(home.path()).join("v0.1.0");
        std::fs::create_dir_all(&stale).expect("mkdir");
        std::fs::write(stale.join("stray"), "x").expect("write");
        ensure(home.path(), &url, "v0.1.0").expect("ensure");
        assert_eq!(
            read(managed_root(home.path(), "0.1.0").expect("root")),
            "v0.1.0"
        );
        assert!(!stale.join("stray").exists());
    }

    /// Daemons on one machine share `<home>/workflow` and fetch the same tag at once (br-ysmy):
    /// every call succeeds, the checkout is whole, and no scratch dir is left behind.
    #[test]
    fn concurrent_ensure_of_one_tag_all_succeed() {
        let (_o, url) = origin(&["v0.1.0"]);
        let home = tempfile::tempdir().expect("home");
        let results: Vec<_> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..6)
                .map(|_| s.spawn(|| ensure(home.path(), &url, "v0.1.0")))
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("thread"))
                .collect()
        });
        for r in &results {
            assert!(r.is_ok(), "{r:?}");
        }
        assert_eq!(
            read(managed_root(home.path(), "0.1.0").expect("root")),
            "v0.1.0"
        );
        let left: Vec<_> = std::fs::read_dir(root(home.path()))
            .expect("workflow dir")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(left, ["v0.1.0"], "no scratch dir left");
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
