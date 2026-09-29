//! `bridle init`: scaffolds `.bridle/config.toml` and `.gitignore` entries in a git repo.
//! Local and additive only: it never overwrites a file, and it doesn't run `sync` or start
//! anything. See docs/design/cli.md.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, bail};

use crate::cli::InitArgs;
use crate::error::CliError;

/// What `doctor` looks for, so a fresh init passes it.
const GITIGNORE_LINES: [&str; 3] = [".bridle/*.db*", ".bridle/daemon.json", ".bridle/cache/"];

pub fn run(args: &InitArgs) -> Result<(), CliError> {
    let repo = match &args.repo {
        Some(p) => p.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    let report = scaffold(&repo, args)?;
    for f in &report.created {
        println!("created  {f}");
    }
    for f in &report.skipped {
        println!("skipped  {f} (already there)");
    }
    println!(
        "\nNext:\n  bridle sync     render the workflow layers into .claude/\n  bridle doctor   check the setup\n  bridle serve    start the daemon"
    );
    if args.stack.is_none()
        && let Some(s) = detect_stack(&repo)
    {
        println!("\nThis looks like a {s} project: rerun with --stack {s} to enable its pack.");
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Report {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn scaffold(repo: &Path, args: &InitArgs) -> anyhow::Result<Report> {
    let head = git_head_branch(repo)
        .with_context(|| format!("{} is not a git repository on a branch", repo.display()))?;
    let mut report = Report::default();

    let config = repo.join(".bridle/config.toml");
    if config.exists() {
        report.skipped.push(".bridle/config.toml".into());
    } else {
        let integration = args.integration.as_deref().unwrap_or(&head);
        if integration.is_empty() || integration.contains(['"', '\n', '\\']) {
            bail!("invalid integration branch {integration:?}");
        }
        std::fs::create_dir_all(repo.join(".bridle")).context("create .bridle")?;
        std::fs::write(&config, render_config(repo, args, integration))
            .context("write .bridle/config.toml")?;
        report.created.push(".bridle/config.toml".into());
    }

    let gi = repo.join(".gitignore");
    let existing = std::fs::read_to_string(&gi).unwrap_or_default();
    let have: Vec<&str> = existing.lines().map(str::trim).collect();
    let missing: Vec<&str> = GITIGNORE_LINES
        .into_iter()
        .filter(|l| !have.contains(l))
        .collect();
    if missing.is_empty() {
        report.skipped.push(".gitignore".into());
    } else {
        let mut out = existing;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("\n# bridle runtime files\n");
        for l in missing {
            out.push_str(l);
            out.push('\n');
        }
        std::fs::write(&gi, out).context("write .gitignore")?;
        report.created.push(".gitignore (appended)".into());
    }
    Ok(report)
}

/// The branch HEAD is on; works in a repo with no commits yet. `None` if not a repo or
/// HEAD is detached.
fn git_head_branch(repo: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|b| !b.is_empty())
}

fn detect_stack(repo: &Path) -> Option<&'static str> {
    [
        ("Cargo.toml", "rust"),
        ("package.json", "typescript"),
        ("pyproject.toml", "python"),
    ]
    .into_iter()
    .find(|(f, _)| repo.join(f).is_file())
    .map(|(_, s)| s)
}

fn detect_check(repo: &Path) -> Option<&'static str> {
    let has = |f: &str| repo.join(f).is_file();
    if has("justfile") || has("Justfile") {
        Some("just check")
    } else if has("Cargo.toml") {
        Some("cargo test")
    } else if has("package.json") {
        Some("npm test")
    } else if has("pyproject.toml") {
        Some("pytest")
    } else {
        None
    }
}

fn render_config(repo: &Path, args: &InitArgs, integration: &str) -> String {
    let mut s = String::new();
    s.push_str("# bridle project config. Reference: docs/design/agent-host/roles-and-config.md\n");
    match &args.name {
        Some(n) => s.push_str(&format!("# project: {}\n", n.replace('\n', " "))),
        None => s.push_str("# project: named after this directory\n"),
    }
    let check = detect_check(repo);
    let check_line = |key: &str| match check {
        Some(c) => format!("{key} = \"{c}\"\n"),
        None => format!("# {key} = \"just check\"\n"),
    };

    // Top-level keys first: they must precede the first table header.
    s.push_str("\n# Where the workflow layers live (repo-relative); roles below then use\n");
    s.push_str("# <workflow>/base/roles/<role>.md as their system prompt.\n");
    if repo.join("workflow/base").is_dir() {
        s.push_str("workflow = \"workflow\"\n");
    } else {
        s.push_str("# workflow = \"workflow\"\n");
    }
    match &args.stack {
        Some(st) => s.push_str(&format!("packs = [\"{st}\"]\n")),
        None => s.push_str("# packs = [\"python\"]   # or typescript, rust\n"),
    }

    s.push_str("\n[branches]\n");
    s.push_str(&format!("integration = \"{integration}\"\n"));
    s.push_str("# release = \"release\"\n");

    s.push_str("\n[roles.manager]\n# model = \"sonnet\"\n");
    s.push_str("\n[roles.worker]\n# model = \"sonnet\"\n");

    s.push_str("\n[commands]\n");
    s.push_str("# What a role runs to check its work.\n");
    s.push_str(&check_line("check"));

    s.push_str("\n[worktrees]\n");
    s.push_str("# Run in each new worktree (install dependencies, build caches).\n");
    s.push_str("# setup = \"npm ci\"\n");
    s.push_str("# Untracked files to copy into each worktree.\n");
    s.push_str("# copy = [\".env\"]\n");

    s.push_str("\n[integration]\n");
    s.push_str("# The gate `bridle land` runs before moving the integration branch.\n");
    s.push_str(&check_line("check"));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::{Status, local_checks};

    fn args(stack: Option<&str>) -> InitArgs {
        InitArgs {
            repo: None,
            name: None,
            integration: None,
            stack: stack.map(String::from),
        }
    }

    fn repo() -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        for a in [
            &["init", "-q", "-b", "trunk"][..],
            &["config", "user.name", "t"],
            &["config", "user.email", "t@example.com"],
            &["commit", "-q", "--allow-empty", "-m", "init"],
        ] {
            let st = Command::new("git")
                .arg("-C")
                .arg(d.path())
                .args(a)
                .status()
                .expect("git runs");
            assert!(st.success(), "git {a:?}");
        }
        d
    }

    #[test]
    fn fresh_init_passes_doctor_and_rerun_changes_nothing() {
        let d = repo();
        std::fs::write(d.path().join("Cargo.toml"), "").expect("write");
        std::fs::write(d.path().join(".gitignore"), "target").expect("write");
        let r = scaffold(d.path(), &args(Some("rust"))).expect("init");
        assert_eq!(r.created.len(), 2);

        let config = std::fs::read_to_string(d.path().join(".bridle/config.toml")).expect("read");
        assert!(config.contains("integration = \"trunk\""), "{config}");
        assert!(config.contains("packs = [\"rust\"]"), "{config}");
        assert!(config.contains("check = \"cargo test\""), "{config}");
        let gi = std::fs::read_to_string(d.path().join(".gitignore")).expect("read");
        assert!(gi.starts_with("target\n"), "{gi}");

        let home = tempfile::tempdir().expect("home");
        let (checks, cfg) = local_checks(d.path(), Some(home.path()));
        assert!(cfg.is_some(), "{checks:#?}");
        assert!(
            checks.iter().all(|c| c.status != Status::Fail),
            "{checks:#?}"
        );

        let r = scaffold(d.path(), &args(Some("rust"))).expect("rerun");
        assert!(r.created.is_empty(), "{r:?}");
        assert_eq!(r.skipped.len(), 2);
        assert_eq!(
            std::fs::read_to_string(d.path().join(".gitignore")).expect("read"),
            gi
        );
    }

    #[test]
    fn existing_config_is_never_touched() {
        let d = repo();
        std::fs::create_dir(d.path().join(".bridle")).expect("mkdir");
        let p = d.path().join(".bridle/config.toml");
        std::fs::write(&p, "# mine\n").expect("write");
        let mut a = args(None);
        a.integration = Some("other".into());
        let r = scaffold(d.path(), &a).expect("init");
        assert_eq!(r.skipped, [".bridle/config.toml"]);
        assert_eq!(std::fs::read_to_string(&p).expect("read"), "# mine\n");
    }

    #[test]
    fn refuses_outside_a_git_repo() {
        let d = tempfile::tempdir().expect("tempdir");
        assert!(scaffold(d.path(), &args(None)).is_err());
        assert!(!d.path().join(".bridle").exists());
    }
}
