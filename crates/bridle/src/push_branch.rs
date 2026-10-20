//! `bridle push-branch`: a worker pushes its own `bridle/<name>` branch to origin (ticket t7ya).
//! Raw `git push` stays denied to workers; this wrapper is the one door, and it is narrow: no
//! force, no refspec, no other branch (not the integration branch, not another agent's), and only
//! from a worktree that has that branch checked out. Pure decision in `check`; `run` does the I/O.

use std::process::Command;

use anyhow::{Context, bail};
use bridle_daemon::config::Config;

use crate::cli::PushBranchArgs;

/// Why the push is refused, or the branch to push. `args` are the words after `push-branch`,
/// `agent` is `$BRIDLE_AGENT_NAME`, `current` the branch checked out here.
pub fn check(
    args: &[String],
    agent: Option<&str>,
    current: &str,
    integration: &str,
) -> Result<String, String> {
    let Some(agent) = agent.filter(|a| !a.is_empty()) else {
        return Err(
            "BRIDLE_AGENT_NAME is not set: only a bridle agent pushes its own branch".into(),
        );
    };
    let own = format!("bridle/{agent}");
    for a in args {
        if a.starts_with('-') {
            return Err(format!(
                "`{a}` is not allowed: push-branch never forces and takes no options"
            ));
        }
        if a.starts_with('+') || a.contains(':') {
            return Err(format!(
                "`{a}` is a refspec: push-branch takes a branch name only, never a force or a rename"
            ));
        }
    }
    if args.len() > 1 {
        return Err("push-branch takes at most one branch name".into());
    }
    let branch = args.first().map(String::as_str).unwrap_or(current);
    if branch == integration {
        return Err(format!(
            "`{branch}` is the integration branch: the manager pushes it (`bridle push`)"
        ));
    }
    if branch != own {
        return Err(format!(
            "`{branch}` is not your branch: you may push only `{own}`"
        ));
    }
    if current != own {
        return Err(format!(
            "run it from your worktree: `{own}` is not checked out here (this is `{current}`)"
        ));
    }
    Ok(own)
}

pub fn run(args: &PushBranchArgs) -> anyhow::Result<()> {
    let out = Command::new("git")
        .args(["symbolic-ref", "--short", "-q", "HEAD"])
        .output()
        .context("running git")?;
    let current = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let top = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git")?;
    let top = String::from_utf8_lossy(&top.stdout).trim().to_string();
    let integration = Config::load(std::path::Path::new(&top))
        .context("loading .bridle/config.toml")?
        .branches
        .integration;
    let agent = std::env::var("BRIDLE_AGENT_NAME").ok();
    let branch = match check(&args.branch, agent.as_deref(), &current, &integration) {
        Ok(b) => b,
        Err(why) => bail!("refusing to push: {why}"),
    };
    // Explicit full refs on both sides, so no push.default or upstream setting can widen it.
    let spec = format!("refs/heads/{branch}:refs/heads/{branch}");
    let status = Command::new("git")
        .args(["push", "origin", &spec])
        .status()
        .context("running git push")?;
    if !status.success() {
        bail!("git push failed ({status})");
    }
    println!("pushed {branch} to origin");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check;

    fn c(args: &[&str], agent: Option<&str>, current: &str) -> Result<String, String> {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        check(&args, agent, current, "main")
    }

    #[test]
    fn accepts_own_branch() {
        assert_eq!(c(&[], Some("w1"), "bridle/w1").unwrap(), "bridle/w1");
        assert_eq!(
            c(&["bridle/w1"], Some("w1"), "bridle/w1").unwrap(),
            "bridle/w1"
        );
    }

    #[test]
    fn refuses_force_forms() {
        for f in ["--force", "-f", "--force-with-lease", "--delete"] {
            assert!(c(&[f], Some("w1"), "bridle/w1").is_err(), "{f}");
        }
        assert!(c(&["bridle/w1", "--force"], Some("w1"), "bridle/w1").is_err());
    }

    #[test]
    fn refuses_refspecs() {
        for r in [
            "+bridle/w1",
            "bridle/w1:main",
            "bridle/w1:bridle/w1",
            ":bridle/w1",
        ] {
            assert!(c(&[r], Some("w1"), "bridle/w1").is_err(), "{r}");
        }
    }

    #[test]
    fn refuses_other_branches() {
        assert!(
            c(&["main"], Some("w1"), "bridle/w1")
                .unwrap_err()
                .contains("integration")
        );
        assert!(
            c(&["bridle/w2"], Some("w1"), "bridle/w1")
                .unwrap_err()
                .contains("not your branch")
        );
        assert!(c(&["bridle/w1", "main"], Some("w1"), "bridle/w1").is_err());
    }

    #[test]
    fn refuses_integration_even_when_checked_out() {
        assert!(c(&[], Some("w1"), "main").is_err());
    }

    #[test]
    fn refuses_another_agents_checkout_and_no_agent() {
        assert!(c(&[], Some("w1"), "bridle/w2").is_err());
        assert!(c(&["bridle/w1"], Some("w1"), "bridle/w2").is_err());
        assert!(c(&[], None, "bridle/w1").is_err());
        assert!(c(&[], Some(""), "bridle/w1").is_err());
    }
}
