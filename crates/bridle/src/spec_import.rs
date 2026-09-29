//! `bridle spec import openspec`: move OpenSpec capability specs into
//! `design/specs` and assign ids (docs/design/specs.md, "Migration from
//! OpenSpec"). Local, no daemon call.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow, bail};

use crate::cli::{Cli, SpecImportArgs, SpecImportOpenspecArgs, SpecImportSource};
use crate::error::CliError;
use crate::render;
use crate::specid::{self, Report};

#[derive(Debug, serde::Serialize)]
struct Import {
    dry_run: bool,
    /// `[from, to]` per moved spec.
    moved: Vec<[String; 2]>,
    /// Files still under `--from` afterwards (generated `.feature`s and the like).
    left: Vec<String>,
    ids: Option<Report>,
}

pub fn run(cli: &Cli, args: &SpecImportArgs) -> Result<(), CliError> {
    let SpecImportSource::Openspec(a) = &args.source;
    let out = import(a, &mut specid::random)?;
    if cli.json {
        render::print_json(&out)?;
        return Ok(());
    }
    let verb = if out.dry_run { "would move" } else { "moved" };
    for [f, t] in &out.moved {
        println!("{verb} {f} -> {t}");
    }
    if let Some(r) = &out.ids {
        let n: usize = r.files.iter().map(|f| f.assigned.len()).sum();
        println!(
            "{n} id(s) {}",
            if out.dry_run { "to assign" } else { "assigned" }
        );
    }
    if out.moved.is_empty() {
        println!("nothing to import");
    }
    if !out.left.is_empty() {
        println!(
            "left in place: {} other file(s) under {} (e.g. .feature files); changes, config and schemas untouched",
            out.left.len(),
            a.from.display()
        );
    }
    Ok(())
}

/// `<from>/<capability>/spec.md` for each capability directory, sorted.
fn sources(from: &Path) -> anyhow::Result<Vec<(PathBuf, String)>> {
    let entries = match std::fs::read_dir(from) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(anyhow!(e).context(format!("reading {}", from.display()))),
    };
    let mut out = Vec::new();
    for e in entries {
        let dir = e?.path();
        let spec = dir.join("spec.md");
        if dir.is_dir() && spec.is_file() {
            let cap = dir
                .file_name()
                .and_then(|s| s.to_str())
                .context("non-UTF-8 capability name")?
                .to_string();
            out.push((spec, cap));
        }
    }
    out.sort();
    Ok(out)
}

fn files_under(dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            files_under(&p, out);
        } else {
            out.push(p.display().to_string());
        }
    }
    out.sort();
}

/// `git mv` for a tracked file, else a plain rename.
fn move_file(from: &Path, to: &Path) -> anyhow::Result<()> {
    let (f, t) = (std::path::absolute(from)?, std::path::absolute(to)?);
    let dir = f.parent().unwrap_or(Path::new("."));
    let tracked = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["ls-files", "--error-unmatch"])
        .arg(&f)
        .output()
        .is_ok_and(|o| o.status.success());
    if !tracked {
        return std::fs::rename(from, to).with_context(|| format!("moving {}", from.display()));
    }
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("mv")
        .arg(&f)
        .arg(&t)
        .output()
        .context("running git mv")?;
    if !o.status.success() {
        bail!(
            "git mv {} failed: {}",
            from.display(),
            String::from_utf8_lossy(&o.stderr).trim()
        );
    }
    Ok(())
}

fn import(a: &SpecImportOpenspecArgs, rng: &mut impl FnMut() -> u64) -> anyhow::Result<Import> {
    let srcs = sources(&a.from)?;

    // Validate everything before touching anything.
    let mut problems = Vec::new();
    let mut plan = Vec::new();
    for (src, cap) in &srcs {
        let name = src.display().to_string();
        let text = std::fs::read_to_string(src).with_context(|| format!("reading {name}"))?;
        if let Err(ds) = bridle_spec::parse_str(&name, &text) {
            problems.extend(ds.iter().map(ToString::to_string));
        }
        let dest = a.to.join(format!("{cap}.md"));
        if dest.exists() {
            problems.push(format!(
                "{}: target exists, refusing to overwrite",
                dest.display()
            ));
        }
        plan.push((src.clone(), dest));
    }
    if !problems.is_empty() {
        bail!("{}\nno files changed", problems.join("\n"));
    }

    let moved = plan
        .iter()
        .map(|(s, d)| [s.display().to_string(), d.display().to_string()])
        .collect();
    let ledger = a.to.join(".ids");
    let mut left = Vec::new();
    let ids = if plan.is_empty() {
        None
    } else if a.dry_run {
        let srcs: Vec<PathBuf> = plan.iter().map(|(s, _)| s.clone()).collect();
        files_under(&a.from, &mut left);
        left.retain(|f| !srcs.iter().any(|s| s.display().to_string() == *f));
        Some(specid::assign(&a.to, &srcs, Some(&ledger), true, rng)?)
    } else {
        std::fs::create_dir_all(&a.to).with_context(|| format!("creating {}", a.to.display()))?;
        for (s, d) in &plan {
            move_file(s, d)?;
        }
        let dests: Vec<PathBuf> = plan.into_iter().map(|(_, d)| d).collect();
        files_under(&a.from, &mut left);
        Some(specid::assign(&a.to, &dests, Some(&ledger), false, rng)?)
    };
    Ok(Import {
        dry_run: a.dry_run,
        moved,
        left,
        ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: &str = "# A\n\n### Requirement: One\nThe system SHALL.\n\n#### Scenario: S\n*Verification*: **non-executable**\nprose\n";
    const IDED: &str = "# B\n\n### Requirement: Two  {#r-abcd}\nThe system SHALL.\n\n#### Scenario: T  {#s-1234}\n*Verification*: **non-executable**\nprose\n";

    fn counter() -> impl FnMut() -> u64 {
        let mut n = 0u64;
        move || {
            n += 1;
            n << 48
        }
    }

    fn tree(b: &str) -> (tempfile::TempDir, SpecImportOpenspecArgs) {
        let d = tempfile::tempdir().expect("tempdir");
        let from = d.path().join("openspec/specs");
        for (cap, text) in [("alpha", PLAIN), ("beta", b)] {
            std::fs::create_dir_all(from.join(cap)).expect("mkdir");
            std::fs::write(from.join(cap).join("spec.md"), text).expect("write");
        }
        std::fs::write(from.join("alpha/alpha.feature"), "Feature: A\n").expect("feature");
        let a = SpecImportOpenspecArgs {
            from,
            to: d.path().join("design/specs"),
            dry_run: false,
        };
        (d, a)
    }

    #[test]
    fn moves_assigns_ids_and_leaves_features() {
        let (_d, a) = tree(IDED);
        let out = import(&a, &mut counter()).expect("import");
        assert_eq!(out.moved.len(), 2);
        assert!(!a.from.join("alpha/spec.md").exists());
        assert!(a.from.join("alpha/alpha.feature").exists());
        assert_eq!(out.left.len(), 1);
        let alpha = std::fs::read_to_string(a.to.join("alpha.md")).expect("alpha");
        assert!(
            alpha.contains("One  {#r-0001}") && alpha.contains("S  {#s-0002}"),
            "{alpha}"
        );
        assert_eq!(
            std::fs::read_to_string(a.to.join("beta.md")).expect("beta"),
            IDED
        );
        let ledger = std::fs::read_to_string(a.to.join(".ids")).expect("ledger");
        for id in ["r-abcd", "s-1234", "r-0001", "s-0002"] {
            assert!(ledger.contains(id), "{ledger}");
        }
        let again = import(&a, &mut counter()).expect("again");
        assert!(again.moved.is_empty() && again.ids.is_none());
    }

    #[test]
    fn dry_run_writes_nothing() {
        let (_d, mut a) = tree(IDED);
        a.dry_run = true;
        let out = import(&a, &mut counter()).expect("import");
        assert_eq!(out.moved.len(), 2);
        assert!(a.from.join("alpha/spec.md").exists());
        assert!(!a.to.exists());
    }

    #[test]
    fn parse_error_changes_nothing() {
        let (_d, a) = tree(
            "### Requirement: Bad\nno keyword here\n#### Scenario: X\n*Verification*: **bogus**\n",
        );
        let e = import(&a, &mut counter()).expect_err("parse error");
        assert!(e.to_string().contains("no files changed"), "{e}");
        assert!(a.from.join("alpha/spec.md").exists());
        assert!(!a.to.exists());
    }

    #[test]
    fn existing_target_is_refused() {
        let (_d, a) = tree(IDED);
        std::fs::create_dir_all(&a.to).expect("mkdir");
        std::fs::write(a.to.join("beta.md"), "x").expect("write");
        let e = import(&a, &mut counter()).expect_err("exists");
        assert!(e.to_string().contains("target exists"), "{e}");
        assert!(a.from.join("alpha/spec.md").exists());
    }
}
