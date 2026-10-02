//! `bridle doctor`: checks the project's setup and says what to fix. Local only: it reads
//! the repo, `.bridle/config.toml` and the tools on PATH, and never talks to a daemon or
//! changes anything. See docs/design/cli.md.

use std::path::Path;
use std::process::Command;

use anyhow::Context;
use bridle_daemon::config::Config;
use serde::Serialize;

use crate::cli::{Cli, DoctorArgs};
use crate::error::CliError;
use crate::render;

/// `git merge-tree --write-tree` (used by `bridle probe` and `land`) needs 2.38.
const MIN_GIT: (u32, u32) = (2, 38);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
    /// One line saying what to do; empty when ok.
    pub fix: String,
}

impl Check {
    fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Check {
            name,
            status: Status::Ok,
            detail: detail.into(),
            fix: String::new(),
        }
    }
    fn warn(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Check {
            name,
            status: Status::Warn,
            detail: detail.into(),
            fix: fix.into(),
        }
    }
    fn fail(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Check {
            name,
            status: Status::Fail,
            detail: detail.into(),
            fix: fix.into(),
        }
    }
}

pub fn run(cli: &Cli, args: &DoctorArgs) -> Result<(), CliError> {
    let repo = match &args.repo {
        Some(p) => p.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    let (mut checks, config) = local_checks(&repo, None);
    checks.extend(tool_checks(config.as_ref()));

    let failed = checks.iter().any(|c| c.status == Status::Fail);
    if cli.json {
        render::print_json(&checks)?;
    } else {
        for c in &checks {
            let tag = match c.status {
                Status::Ok => "ok",
                Status::Warn => "warn",
                Status::Fail => "FAIL",
            };
            println!("{tag:<4}  {}: {}", c.name, c.detail);
            if !c.fix.is_empty() {
                println!("      fix: {}", c.fix);
            }
        }
    }
    if failed {
        return Err(CliError::Other(anyhow::anyhow!("doctor found problems")));
    }
    Ok(())
}

fn git(repo: &Path, args: &[&str]) -> Option<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()
}

fn git_ok(repo: &Path, args: &[&str]) -> bool {
    git(repo, args).is_some_and(|o| o.status.success())
}

/// Everything that only needs the repo. `home` overrides the machine-wide bridle home
/// (tests).
pub fn local_checks(repo: &Path, home: Option<&Path>) -> (Vec<Check>, Option<Config>) {
    let mut out = Vec::new();

    let is_repo = git_ok(repo, &["rev-parse", "--git-dir"]);
    if is_repo {
        out.push(Check::ok("git repo", repo.display().to_string()));
    } else {
        out.push(Check::fail(
            "git repo",
            format!("{} is not a git repository", repo.display()),
            "run from the project's clone, or pass --repo",
        ));
    }

    let config = match Config::load_with_home(repo, home) {
        Ok(c) => {
            out.push(Check::ok("config", ".bridle/config.toml loads"));
            Some(c)
        }
        Err(e) => {
            out.push(Check::fail(
                "config",
                e.to_string(),
                "fix .bridle/config.toml",
            ));
            None
        }
    };

    let integration = config
        .as_ref()
        .map_or("main", |c| c.branches.integration.as_str());
    if is_repo {
        let refname = format!("refs/heads/{integration}");
        if git_ok(repo, &["rev-parse", "--verify", "--quiet", &refname]) {
            out.push(Check::ok(
                "integration branch",
                format!("{integration} exists"),
            ));
        } else {
            out.push(Check::fail(
                "integration branch",
                format!("branch {integration:?} does not exist"),
                format!(
                    "create it (`git branch {integration} <base>`) or set [branches] integration in .bridle/config.toml"
                ),
            ));
        }
        out.extend(gitignore_check(repo));
        out.push(state_branch_check(repo));
    }

    out.push(machine_config_check(home));

    if let Some(config) = &config {
        out.extend(config_file_checks(repo, config));
        out.push(ports_check(config));
        if let Some(problem) = &config.tasks_settle_problem {
            out.push(Check::warn(
                "tasks settle",
                format!("{problem}; the daemon uses 5m"),
                "fix [tasks] settle in .bridle/config.toml",
            ));
        }
    }
    (out, config)
}

/// `~/.bridle/config.toml`: each `[[focus]]` / `[[budget.schedule]]` block whose overnight end
/// lacks `+1d` is a failure with its fix. The daemon only warns about these; doctor is where
/// they're errors. A missing file is fine.
fn machine_config_check(home: Option<&Path>) -> Check {
    let home = home
        .map(Path::to_path_buf)
        .unwrap_or_else(bridle_api::discovery::bridle_home);
    match bridle_daemon::config::machine_config_problems(&home) {
        Ok(p) if p.is_empty() => Check::ok("machine config", "config.toml blocks are valid"),
        Ok(p) => Check::fail(
            "machine config",
            p.join("; "),
            "edit the block(s) in ~/.bridle/config.toml as named",
        ),
        Err(e) => Check::fail("machine config", e.to_string(), "fix ~/.bridle/config.toml"),
    }
}

fn gitignore_check(repo: &Path) -> Option<Check> {
    let missing: Vec<&str> = [
        ".bridle/cache/x",
        ".bridle/bridle.db",
        ".bridle/daemon.json",
    ]
    .into_iter()
    .filter(|p| !git_ok(repo, &["check-ignore", "-q", p]))
    .collect();
    Some(if missing.is_empty() {
        Check::ok(".gitignore", "bridle's runtime files are ignored")
    } else {
        Check::warn(
            ".gitignore",
            format!("not ignored: {}", missing.join(", ")),
            "add `.bridle/*.db*`, `.bridle/daemon.json` and `.bridle/cache/` to .gitignore",
        )
    })
}

fn state_branch_check(repo: &Path) -> Check {
    let has_branch = git_ok(
        repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            "refs/heads/bridle/state",
        ],
    );
    // The daemon creates the branch on first serve, so its absence only matters once a
    // database exists to hold tasks.
    let has_db = repo.parent().is_some_and(|w| {
        bridle_api::discovery::state_dir(w)
            .join("bridle.db")
            .exists()
    });
    match (has_branch, has_db) {
        (true, _) => Check::ok("state branch", "bridle/state exists"),
        (false, false) => Check::ok("state branch", "not created yet (made on first serve)"),
        (false, true) => Check::warn(
            "state branch",
            "bridle.db exists but branch bridle/state does not",
            "restore bridle/state, or tasks can't be rebuilt from git",
        ),
    }
}

fn is_remote(s: &str) -> bool {
    s.contains("://") || s.starts_with("git@")
}

fn config_file_checks(repo: &Path, config: &Config) -> Vec<Check> {
    let mut missing: Vec<String> = Vec::new();
    let mut no_prompt: Vec<&str> = Vec::new();

    for (name, role) in &config.roles {
        match &role.system_prompt {
            None => no_prompt.push(name),
            Some(p) if !repo.join(p).is_file() => {
                missing.push(format!("role {name} system_prompt {}", p.display()))
            }
            Some(_) => {}
        }
    }
    if let Some(w) = config.workflow.as_deref().filter(|w| !is_remote(w)) {
        let root = repo.join(w);
        if let Err(e) = config.workflow_root(repo) {
            missing.push(format!("workflow {w} ({e})"));
        } else {
            for pack in &config.packs {
                if !root.join("packs").join(pack).is_dir() {
                    missing.push(format!("pack {w}/packs/{pack}"));
                }
            }
        }
    }
    for (id, comp) in &config.components {
        if let Some(d) = &comp.docs
            && !repo.join(d).exists()
        {
            missing.push(format!("component {id} docs {d}"));
        }
    }

    let mut out = Vec::new();
    out.push(if missing.is_empty() {
        Check::ok("referenced files", "all exist")
    } else {
        Check::fail(
            "referenced files",
            format!("missing: {}", missing.join("; ")),
            "create them or fix the paths in .bridle/config.toml",
        )
    });
    out.push(if no_prompt.is_empty() {
        Check::ok("role prompts", "every role has a system prompt")
    } else {
        Check::warn(
            "role prompts",
            format!("no system prompt: {}", no_prompt.join(", ")),
            "set `workflow` (roles then use <workflow>/base/roles/<role>.md) or a role's system_prompt",
        )
    });
    out
}

fn ports_check(config: &Config) -> Check {
    let (lo, hi) = config.ports.range;
    if lo > hi {
        Check::fail(
            "ports",
            format!("[ports] range {lo}..{hi} is empty"),
            "make the range's first port no greater than its last",
        )
    } else if lo < 1024 {
        Check::warn(
            "ports",
            format!("[ports] range starts at {lo}, in the privileged range"),
            "start the range at 1024 or above",
        )
    } else {
        let free = (u32::from(hi) - u32::from(lo) + 1)
            - config
                .ports
                .reserved
                .iter()
                .filter(|p| (lo..=hi).contains(p))
                .count() as u32;
        if free == 0 {
            Check::fail(
                "ports",
                "every port in [ports] range is reserved",
                "shrink `reserved` or widen `range`",
            )
        } else {
            Check::ok("ports", format!("{lo}..{hi}, {free} allocatable"))
        }
    }
}

/// Checks that shell out to tools on PATH.
pub fn tool_checks(config: Option<&Config>) -> Vec<Check> {
    let mut out = Vec::new();

    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match parse_git_version(&v) {
                Some(ver) if ver >= MIN_GIT => out.push(Check::ok("git", v)),
                Some(_) => out.push(Check::fail(
                    "git",
                    format!("{v}; need 2.38 or newer"),
                    "upgrade git (`git merge-tree --write-tree` is used by probe and land)",
                )),
                None => out.push(Check::warn("git", v, "could not read the git version")),
            }
        }
        _ => out.push(Check::fail(
            "git",
            "not on PATH",
            "install git 2.38 or newer",
        )),
    }

    match Command::new("claude").arg("--version").output() {
        Ok(o) if o.status.success() => out.push(Check::ok(
            "claude",
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )),
        _ => out.push(Check::fail(
            "claude",
            "not on PATH",
            "install Claude Code and make sure `claude` is on PATH",
        )),
    }

    if config.is_some_and(|c| c.ci.github) {
        match Command::new("gh").arg("--version").output() {
            Ok(o) if o.status.success() => out.push(Check::ok("gh", "on PATH")),
            _ => out.push(Check::fail(
                "gh",
                "not on PATH but [ci] github is enabled",
                "install the GitHub CLI (`gh`) or disable [ci]",
            )),
        }
    }
    out
}

/// "git version 2.39.3 (Apple Git-145)" -> (2, 39).
fn parse_git_version(s: &str) -> Option<(u32, u32)> {
    let v = s.split_whitespace().nth(2)?;
    let mut it = v.split('.');
    Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(dir: &Path, args: &[&str]) {
        let st = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .status()
            .expect("git runs");
        assert!(st.success(), "git {args:?}");
    }

    fn repo(config: Option<&str>) -> tempfile::TempDir {
        let d = tempfile::tempdir().expect("tempdir");
        sh(d.path(), &["init", "-q", "-b", "main"]);
        sh(d.path(), &["config", "user.name", "t"]);
        sh(d.path(), &["config", "user.email", "t@example.com"]);
        std::fs::write(
            d.path().join(".gitignore"),
            ".bridle/*.db*\n.bridle/daemon.json\n.bridle/cache/\n",
        )
        .expect("write");
        if let Some(c) = config {
            std::fs::create_dir(d.path().join(".bridle")).expect("mkdir");
            std::fs::write(d.path().join(".bridle/config.toml"), c).expect("write");
        }
        sh(d.path(), &["add", "-A"]);
        sh(d.path(), &["commit", "-q", "-m", "init"]);
        d
    }

    fn get<'a>(checks: &'a [Check], name: &str) -> &'a Check {
        checks
            .iter()
            .find(|c| c.name == name)
            .expect("check present")
    }

    #[test]
    fn healthy_repo_has_no_failures() {
        let d = repo(Some("[commands]\ncheck = \"just check\"\n"));
        let home = tempfile::tempdir().expect("home");
        let (checks, _) = local_checks(d.path(), Some(home.path()));
        assert!(
            checks.iter().all(|c| c.status != Status::Fail),
            "{checks:#?}"
        );
        assert_eq!(get(&checks, "integration branch").status, Status::Ok);
        assert_eq!(get(&checks, ".gitignore").status, Status::Ok);
    }

    fn machine_check(toml: Option<&str>) -> Check {
        let d = repo(Some("[commands]\ncheck = \"just check\"\n"));
        let home = tempfile::tempdir().expect("home");
        if let Some(t) = toml {
            std::fs::write(home.path().join("config.toml"), t).expect("write");
        }
        let (checks, _) = local_checks(d.path(), Some(home.path()));
        checks
            .into_iter()
            .find(|c| c.name == "machine config")
            .expect("check present")
    }

    const BLOCKS: &str = r#"
[[focus]]
name = "evening"
days = "all"
start = "21:30"
end = "00:00"

[[budget.schedule]]
name = "night"
days = "all"
start = "23:00"
end = "08:00"
hold_at = 70
wind_down_at = 80
stop_at = 90

[[budget.schedule]]
name = "preset"
hold_at = 70
wind_down_at = 80
stop_at = 90
"#;

    #[test]
    fn machine_config_names_each_bad_block_with_its_fix() {
        let c = machine_check(Some(BLOCKS));
        assert_eq!(c.status, Status::Fail);
        assert!(
            c.detail
                .contains("night: end 08:00 is before start 23:00; write \"08:00+1d\""),
            "{}",
            c.detail
        );
        assert_eq!(c.detail.matches("before start").count(), 1, "{}", c.detail);
    }

    #[test]
    fn machine_config_passes_when_fixed_or_missing() {
        let fixed = BLOCKS.replace("\"08:00\"", "\"08:00+1d\"");
        assert_eq!(machine_check(Some(&fixed)).status, Status::Ok);
        assert_eq!(machine_check(None).status, Status::Ok);
    }

    #[test]
    fn missing_integration_branch_fails_with_the_fix() {
        let d = repo(Some("[branches]\nintegration = \"dev\"\n"));
        let home = tempfile::tempdir().expect("home");
        let (checks, _) = local_checks(d.path(), Some(home.path()));
        let c = get(&checks, "integration branch");
        assert_eq!(c.status, Status::Fail);
        assert!(c.fix.contains("git branch dev"), "{}", c.fix);
    }

    #[test]
    fn bad_config_reports_the_parse_error() {
        let d = repo(Some("[branches\nintegration = 1\n"));
        let home = tempfile::tempdir().expect("home");
        let (checks, config) = local_checks(d.path(), Some(home.path()));
        assert!(config.is_none());
        let c = get(&checks, "config");
        assert_eq!(c.status, Status::Fail);
        assert!(c.detail.contains("parsing"), "{}", c.detail);
    }

    #[test]
    fn missing_referenced_files_fail() {
        let d = repo(Some("workflow = \"nowhere\"\n"));
        let home = tempfile::tempdir().expect("home");
        let (checks, _) = local_checks(d.path(), Some(home.path()));
        let c = get(&checks, "referenced files");
        assert_eq!(c.status, Status::Fail);
        assert!(c.detail.contains("workflow nowhere"), "{}", c.detail);
    }

    #[test]
    fn machine_workflow_override_satisfies_a_project_path_that_is_missing() {
        let d = repo(Some("workflow = \"/nowhere/on/this/machine\"\n"));
        let home = tempfile::tempdir().expect("home");
        let wf = tempfile::tempdir().expect("wf");
        std::fs::write(
            home.path().join("config.toml"),
            format!("workflow = \"{}\"\n", wf.path().display()),
        )
        .expect("write");
        let (checks, _) = local_checks(d.path(), Some(home.path()));
        let c = get(&checks, "referenced files");
        assert!(!c.detail.contains("workflow"), "{}", c.detail);
    }

    #[test]
    fn missing_gitignore_entries_warn() {
        let d = repo(None);
        std::fs::write(d.path().join(".gitignore"), "").expect("write");
        let home = tempfile::tempdir().expect("home");
        let (checks, _) = local_checks(d.path(), Some(home.path()));
        assert_eq!(get(&checks, ".gitignore").status, Status::Warn);
    }

    #[test]
    fn parses_git_versions() {
        assert_eq!(
            parse_git_version("git version 2.39.3 (Apple Git-145)"),
            Some((2, 39))
        );
        assert_eq!(parse_git_version("nonsense"), None);
    }
}
