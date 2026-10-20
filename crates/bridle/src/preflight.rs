//! `bridle project preflight` (br-3ch7, kx5u K1): read-only checks that a project can move to
//! another machine. A list of named checks, each pass or fail with a message; `--to` adds the
//! new machine's (`preflight_remote`, br-bb2n).
//! The absolute-path scan is shared with `bridle daemon doctor`.

use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::cli::{Cli, PreflightArgs};
use crate::error::CliError;
use crate::render;

/// Keys whose value is a path that must mean the same on every machine.
const PATH_KEYS: [&str; 2] = ["workflow", "system_prompt"];

#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    pub file: &'static str,
    pub line: usize,
    pub key: String,
    pub value: String,
}

/// `workflow` / `system_prompt` values in the project's `.bridle/config.toml` that are absolute
/// or start with `~`: they name a directory on one machine only. A line scan, not a TOML parse,
/// because the finding needs the line number.
pub fn absolute_path_findings(repo: &Path) -> Vec<Finding> {
    const FILE: &str = ".bridle/config.toml";
    let Ok(text) = std::fs::read_to_string(repo.join(FILE)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !PATH_KEYS.contains(&key) {
            continue;
        }
        let value = rest.trim().trim_matches(['"', '\'']);
        let b = value.as_bytes();
        let absolute = value.starts_with(['/', '~', '\\'])
            || (b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':');
        if absolute {
            out.push(Finding {
                file: FILE,
                line: i + 1,
                key: key.to_string(),
                value: value.to_string(),
            });
        }
    }
    out
}

pub fn describe(f: &Finding) -> String {
    format!("{}:{} {} = {:?}", f.file, f.line, f.key, f.value)
}

pub const ABSOLUTE_FIX: &str = "edit it by hand to a repo-relative path (or unset it): where the workflow comes from is a decision, so bridle does not rewrite it";

#[derive(Debug, serde::Serialize)]
pub struct Outcome {
    pub name: &'static str,
    pub pass: bool,
    pub message: String,
}

type PreflightCheck = (&'static str, fn(&Path) -> (bool, String));

/// The checks, in order. The stable-project check (K3) is added by `run_checks`: it also takes
/// `--leave`.
const CHECKS: &[PreflightCheck] = &[("absolute paths", absolute_paths)];

fn absolute_paths(repo: &Path) -> (bool, String) {
    let found = absolute_path_findings(repo);
    if found.is_empty() {
        return (true, "no absolute workflow or system_prompt".into());
    }
    let list: Vec<String> = found.iter().map(describe).collect();
    (
        false,
        format!(
            "{}; fix by hand before the move: {ABSOLUTE_FIX}",
            list.join("; ")
        ),
    )
}

pub fn run_checks(repo: &Path, leave: &[String]) -> Vec<Outcome> {
    let mut out: Vec<Outcome> = CHECKS
        .iter()
        .map(|(name, f)| {
            let (pass, message) = f(repo);
            Outcome {
                name,
                pass,
                message,
            }
        })
        .collect();
    let (pass, message) = crate::stable::check(repo, leave);
    out.push(Outcome {
        name: "stable project",
        pass,
        message,
    });
    out
}

pub fn run(cli: &Cli, args: &PreflightArgs) -> Result<(), CliError> {
    let repo: PathBuf = match &args.repo {
        Some(p) => p.clone(),
        None => std::env::current_dir().context("current directory")?,
    };
    let mut results = run_checks(&repo, &args.leave);
    if let Some(machine) = &args.to {
        let home = bridle_api::discovery::bridle_home();
        let map =
            bridle_api::machines::MachineMap::load(&home).map_err(|e| CliError::Other(e.into()))?;
        let confirm = |q: &str| {
            use std::io::{BufRead, Write};
            if cli.json {
                return false;
            }
            print!("{q} [y/N] ");
            let _ = std::io::stdout().flush();
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).is_ok()
                && line.trim().eq_ignore_ascii_case("y")
        };
        let files = crate::project_sync::Real { home };
        let remote = crate::preflight_remote::run_to(
            &repo,
            machine,
            args.path.as_deref(),
            &map,
            &crate::preflight_remote::Ssh,
            &files,
            &confirm,
        )
        .map_err(|e| CliError::Other(anyhow::anyhow!(e)))?;
        results.extend(remote);
    }
    if cli.json {
        render::print_json(&results)?;
    } else {
        for r in &results {
            println!(
                "{:<4}  {}: {}",
                if r.pass { "ok" } else { "FAIL" },
                r.name,
                r.message
            );
        }
    }
    if results.iter().any(|r| !r.pass) {
        return Err(CliError::Other(anyhow::anyhow!(
            "preflight refused the move"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_with(config: &str) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join(".bridle")).unwrap();
        std::fs::write(d.path().join(".bridle/config.toml"), config).unwrap();
        d
    }

    #[test]
    fn absolute_workflow_is_refused_with_key_and_line() {
        let d = repo_with("[x]\nname = \"a\"\nworkflow = \"/Volumes/Data/work/workflow\"\n");
        let f = absolute_path_findings(d.path());
        assert_eq!(f.len(), 1);
        assert_eq!((f[0].line, f[0].key.as_str()), (3, "workflow"));
        let r = run_checks(d.path(), &[]);
        assert!(!r[0].pass);
        assert!(r[0].message.contains(".bridle/config.toml:3 workflow"));
    }

    #[test]
    fn tilde_prompt_is_refused() {
        let d = repo_with("[roles.w]\nsystem_prompt = '~/p.md'\n");
        assert_eq!(absolute_path_findings(d.path()).len(), 1);
    }

    #[test]
    fn relative_unset_and_commented_pass() {
        for c in [
            "workflow = \".bridle/workflow\"\n",
            "name = \"a\"\n",
            "# workflow = \"/abs\"\n",
        ] {
            let d = repo_with(c);
            assert!(run_checks(d.path(), &[])[0].pass, "{c}");
        }
    }
}
