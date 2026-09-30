//! Vendoring the base workflow into a project (`bridle init`, `bridle workflow update`;
//! docs/design/workflow-layers.md, "Two modes"). The copy lives at `.bridle/workflow/`, is
//! committed by the human, and changes only when they run an update.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use bridle_daemon::config::{Config, VENDORED_WORKFLOW};

use crate::cli::WorkflowAction;
use crate::error::CliError;

const DEFAULT_URL: &str = "https://github.com/jonathanbranam/bridle.git";

/// The tag matching this binary's own version.
fn own_tag() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

/// The `workflow/` directory of the clone this binary was built from, if it's still there.
fn local_clone() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../workflow");
    p.join("base").is_dir().then_some(p)
}

/// `path -> contents` for every file under `dir`; empty if `dir` doesn't exist.
fn snapshot(dir: &Path) -> anyhow::Result<BTreeMap<PathBuf, Vec<u8>>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) -> anyhow::Result<()> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(());
        };
        for e in entries {
            let p = e?.path();
            if p.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            if p.is_dir() {
                walk(root, &p, out)?;
            } else {
                let rel = p.strip_prefix(root).expect("under root").to_path_buf();
                out.insert(
                    rel,
                    std::fs::read(&p).with_context(|| p.display().to_string())?,
                );
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out)?;
    Ok(out)
}

/// What an install changed, as repo-relative-to-the-workflow paths.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub added: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
}

/// Replaces `<repo>/.bridle/workflow` with the contents of `src` (a `workflow/` directory).
pub fn install(repo: &Path, src: &Path) -> anyhow::Result<Changes> {
    if !src.join("base").is_dir() {
        bail!(
            "{} has no base/ directory: not a bridle workflow",
            src.display()
        );
    }
    let dest = repo.join(VENDORED_WORKFLOW);
    let old = snapshot(&dest)?;
    let new = snapshot(src)?;
    if dest.exists() {
        std::fs::remove_dir_all(&dest).context("remove the old vendored workflow")?;
    }
    for (rel, bytes) in &new {
        let to = dest.join(rel);
        std::fs::create_dir_all(to.parent().expect("has parent")).context("create dir")?;
        std::fs::write(&to, bytes).with_context(|| to.display().to_string())?;
    }
    let mut c = Changes::default();
    for (rel, bytes) in &new {
        match old.get(rel) {
            None => c.added.push(rel.clone()),
            Some(b) if b != bytes => c.changed.push(rel.clone()),
            Some(_) => {}
        }
    }
    c.removed = old
        .keys()
        .filter(|k| !new.contains_key(*k))
        .cloned()
        .collect();
    Ok(c)
}

/// Clones `url` at `tag` (depth 1) into a temp dir; the workflow is `<dir>/workflow`.
fn clone_tag(url: &str, tag: &str) -> anyhow::Result<tempfile::TempDir> {
    let tmp = tempfile::tempdir().context("temp dir")?;
    let out = Command::new("git")
        .args(["clone", "--quiet", "--depth", "1", "--branch", tag, url])
        .arg(tmp.path())
        .output()
        .context("run git")?;
    if !out.status.success() {
        bail!(
            "couldn't fetch the workflow from {url} at {tag}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(tmp)
}

/// Fetches the workflow (from `url` at `tag`, or `local` when no tag is asked for) and
/// installs it into `repo`.
pub fn fetch_and_install(
    repo: &Path,
    local: Option<&Path>,
    url: &str,
    tag: Option<&str>,
) -> anyhow::Result<Changes> {
    if let (None, Some(dir)) = (tag, local) {
        return install(repo, dir);
    }
    let tag = tag.map_or_else(own_tag, String::from);
    let tmp = clone_tag(url, &tag)?;
    install(repo, &tmp.path().join("workflow"))
}

fn url() -> anyhow::Result<String> {
    Ok(Config::machine_workflow_url(None)?.unwrap_or_else(|| DEFAULT_URL.to_string()))
}

/// For `bridle init`: vendors the workflow unless the project already names one (or has a
/// vendored copy). Returns the changes if it installed.
pub fn init_vendor(repo: &Path) -> anyhow::Result<Option<Changes>> {
    if Config::load(repo)?.workflow.is_some() {
        return Ok(None);
    }
    fetch_and_install(repo, local_clone().as_deref(), &url()?, None).map(Some)
}

pub fn run(action: &WorkflowAction) -> Result<(), CliError> {
    let WorkflowAction::Update(args) = action;
    let repo = match &args.repo {
        Some(p) => p.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    if !repo.join(VENDORED_WORKFLOW).join("base").is_dir() {
        return Err(anyhow::anyhow!(
            "{} isn't vendored in this project; run `bridle init` (a project with `workflow = <path>` updates at `bridle sync`)",
            VENDORED_WORKFLOW
        )
        .into());
    }
    let c = fetch_and_install(&repo, local_clone().as_deref(), &url()?, args.to.as_deref())?;
    print_changes(&c);
    Ok(())
}

pub fn print_changes(c: &Changes) {
    for (label, list) in [
        ("added", &c.added),
        ("changed", &c.changed),
        ("removed", &c.removed),
    ] {
        for p in list {
            println!("{label:8} {VENDORED_WORKFLOW}/{}", p.display());
        }
    }
    if *c == Changes::default() {
        println!("{VENDORED_WORKFLOW} is already up to date");
    } else {
        println!("\nReview with git diff, then commit it.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, a: &[&str]) {
        let st = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(a)
            .status()
            .expect("git");
        assert!(st.success(), "git {a:?}");
    }

    /// A local git repo with `workflow/base/rules/a.md` tagged `v1`, then edited and tagged `v2`.
    fn source() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tmp");
        let r = d.path().join("workflow/base/rules");
        std::fs::create_dir_all(&r).expect("mkdir");
        git(d.path(), &["init", "-q", "-b", "main"]);
        git(d.path(), &["config", "user.name", "t"]);
        git(d.path(), &["config", "user.email", "t@example.com"]);
        std::fs::write(r.join("a.md"), "one").expect("w");
        std::fs::write(r.join("gone.md"), "x").expect("w");
        git(d.path(), &["add", "."]);
        git(d.path(), &["commit", "-q", "-m", "1"]);
        git(d.path(), &["tag", "v1"]);
        std::fs::write(r.join("a.md"), "two").expect("w");
        std::fs::write(r.join("b.md"), "new").expect("w");
        std::fs::remove_file(r.join("gone.md")).expect("rm");
        git(d.path(), &["add", "-A"]);
        git(d.path(), &["commit", "-q", "-m", "2"]);
        git(d.path(), &["tag", "v2"]);
        d
    }

    #[test]
    fn install_from_a_tag_then_update_reports_the_diff() {
        let src = source();
        let url = src.path().to_str().expect("utf8");
        let proj = tempfile::tempdir().expect("tmp");

        let c = fetch_and_install(proj.path(), None, url, Some("v1")).expect("v1");
        assert_eq!(c.added.len(), 2, "{c:?}");
        let a = proj.path().join(".bridle/workflow/base/rules/a.md");
        assert_eq!(std::fs::read_to_string(&a).expect("r"), "one");
        assert!(!proj.path().join(".bridle/workflow/.git").exists());

        let c = fetch_and_install(proj.path(), None, url, Some("v2")).expect("v2");
        assert_eq!(c.added, [PathBuf::from("base/rules/b.md")]);
        assert_eq!(c.changed, [PathBuf::from("base/rules/a.md")]);
        assert_eq!(c.removed, [PathBuf::from("base/rules/gone.md")]);
        assert_eq!(std::fs::read_to_string(&a).expect("r"), "two");

        let c = fetch_and_install(proj.path(), None, url, Some("v2")).expect("again");
        assert_eq!(c, Changes::default());
    }

    #[test]
    fn local_clone_is_used_without_a_tag() {
        let src = source();
        let proj = tempfile::tempdir().expect("tmp");
        let local = src.path().join("workflow");
        let c = fetch_and_install(proj.path(), Some(&local), "/nonexistent", None).expect("local");
        assert_eq!(c.added.len(), 2, "{c:?}");
    }

    #[test]
    fn a_missing_tag_or_url_is_a_clear_error() {
        let src = source();
        let proj = tempfile::tempdir().expect("tmp");
        let url = src.path().to_str().expect("utf8");
        let e = fetch_and_install(proj.path(), None, url, Some("v9")).expect_err("no tag");
        assert!(e.to_string().contains("couldn't fetch the workflow"), "{e}");
        assert!(!proj.path().join(".bridle/workflow").exists());
        assert!(fetch_and_install(proj.path(), None, "/nonexistent/x", Some("v1")).is_err());
    }

    #[test]
    fn a_vendored_copy_is_the_workflow_when_none_is_set() {
        let src = source();
        let proj = tempfile::tempdir().expect("tmp");
        install(proj.path(), &src.path().join("workflow")).expect("install");
        let home = tempfile::tempdir().expect("home");
        let c = Config::load_with_home(proj.path(), Some(home.path())).expect("load");
        let root = c.workflow_root(proj.path()).expect("root").expect("some");
        assert!(root.ends_with(".bridle/workflow"), "{root:?}");
    }
}
