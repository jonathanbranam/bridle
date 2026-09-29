//! `bridle spec id`: write stable ids into spec headings (docs/design/specs.md,
//! "Stable ids"). Local, no daemon call.

use std::collections::{BTreeMap, BTreeSet};
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use bridle_spec::Assigned;

use crate::cli::{Cli, SpecIdArgs};
use crate::error::CliError;
use crate::render;

#[derive(serde::Serialize)]
struct FileReport {
    file: String,
    assigned: Vec<AssignedRow>,
}

#[derive(serde::Serialize)]
struct AssignedRow {
    line: usize,
    id: String,
    title: String,
}

#[derive(serde::Serialize)]
struct Report {
    dry_run: bool,
    files: Vec<FileReport>,
    /// Ids added to the ledger: the new ones and any found in files but missing from it.
    ledger_added: Vec<String>,
}

pub fn run(cli: &Cli, args: &SpecIdArgs) -> Result<(), CliError> {
    let root = args
        .root
        .clone()
        .unwrap_or_else(|| PathBuf::from("design/specs"));
    let report = assign(
        &root,
        &args.paths,
        args.ledger.as_deref(),
        args.dry_run,
        &mut random,
    )?;
    if cli.json {
        render::print_json(&report)?;
    } else {
        let verb = if report.dry_run {
            "would assign"
        } else {
            "assigned"
        };
        for f in &report.files {
            for a in &f.assigned {
                println!("{}:{}: {verb} {} to {:?}", f.file, a.line, a.id, a.title);
            }
        }
        let n: usize = report.files.iter().map(|f| f.assigned.len()).sum();
        println!(
            "{n} id(s) {}, {} ledger entr(ies) {}",
            if report.dry_run {
                "to assign"
            } else {
                "assigned"
            },
            report.ledger_added.len(),
            if report.dry_run { "to add" } else { "added" },
        );
    }
    Ok(())
}

/// A fresh 64-bit value per call from std's randomly keyed hasher.
fn random() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
}

/// Every `*.md` under `path` (or `path` itself), sorted.
fn spec_files(path: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if path.is_dir() {
        let mut entries = std::fs::read_dir(path)
            .and_then(|d| {
                d.map(|e| e.map(|e| e.path()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .with_context(|| format!("reading {}", path.display()))?;
        entries.sort();
        for e in entries {
            if e.is_dir() || e.extension().is_some_and(|x| x == "md") {
                spec_files(&e, out)?;
            }
        }
    } else if path.exists() {
        out.push(path.to_path_buf());
    } else {
        anyhow::bail!("no such file or directory: {}", path.display());
    }
    Ok(())
}

fn assign(
    root: &Path,
    paths: &[PathBuf],
    ledger: Option<&Path>,
    dry_run: bool,
    rng: &mut impl FnMut() -> u64,
) -> anyhow::Result<Report> {
    let ledger_path = ledger
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(".ids"));
    let ledger_text = match std::fs::read_to_string(&ledger_path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(anyhow!(e).context(format!("reading {}", ledger_path.display()))),
    };
    let in_ledger: BTreeSet<String> = ledger_text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();

    let mut files = Vec::new();
    if paths.is_empty() {
        spec_files(root, &mut files)?;
    } else {
        for p in paths {
            spec_files(p, &mut files)?;
        }
    }

    // Read and parse everything first: ids already in files count as taken
    // (and must be unique across the set) before any new one is drawn.
    let mut texts = Vec::new();
    let mut existing: BTreeMap<String, String> = BTreeMap::new();
    let mut problems = Vec::new();
    for f in &files {
        let name = f.display().to_string();
        let text = std::fs::read_to_string(f).with_context(|| format!("reading {name}"))?;
        match bridle_spec::parse_str(&name, &text) {
            Ok(spec) => {
                let ids = spec.requirements.iter().flat_map(|r| {
                    r.id.iter().map(|i| (i, r.line)).chain(
                        r.scenarios
                            .iter()
                            .filter_map(|s| s.id.as_ref().map(|i| (i, s.line))),
                    )
                });
                for (id, line) in ids {
                    if let Some(prev) = existing.insert(id.clone(), name.clone()) {
                        problems.push(format!(
                            "{name}:{line}: duplicate id {id:?}, also used in {prev}"
                        ));
                    }
                }
            }
            Err(ds) => problems.extend(ds.iter().map(ToString::to_string)),
        }
        texts.push((f, name, text));
    }
    if !problems.is_empty() {
        anyhow::bail!("{}\nno files changed", problems.join("\n"));
    }

    let mut taken: BTreeSet<String> = in_ledger.clone();
    taken.extend(existing.keys().cloned());
    let mut ledger_added: Vec<String> = existing
        .keys()
        .filter(|i| !in_ledger.contains(*i))
        .cloned()
        .collect();

    let mut edits = Vec::new();
    let mut report_files = Vec::new();
    for (path, name, text) in &texts {
        let (out, assigned) =
            bridle_spec::assign_ids(name, text, &mut taken, rng).map_err(|ds| {
                anyhow!(
                    ds.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })?;
        if assigned.is_empty() {
            continue;
        }
        ledger_added.extend(assigned.iter().map(|a| a.id.clone()));
        edits.push((path, out));
        report_files.push(FileReport {
            file: name.clone(),
            assigned: assigned
                .into_iter()
                .map(|Assigned { line, id, title }| AssignedRow { line, id, title })
                .collect(),
        });
    }

    if !dry_run {
        // Ledger first: a crash between the writes leaves ids reserved, never reused.
        if !ledger_added.is_empty() {
            let mut new = ledger_text;
            if !new.is_empty() && !new.ends_with('\n') {
                new.push('\n');
            }
            for id in &ledger_added {
                new.push_str(id);
                new.push('\n');
            }
            if let Some(dir) = ledger_path.parent() {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("creating {}", dir.display()))?;
            }
            std::fs::write(&ledger_path, new)
                .with_context(|| format!("writing {}", ledger_path.display()))?;
        }
        for (path, out) in edits {
            std::fs::write(path, out).with_context(|| format!("writing {}", path.display()))?;
        }
    }
    Ok(Report {
        dry_run,
        files: report_files,
        ledger_added,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "# Cap\n\n### Requirement: One\nThe system SHALL.\n\n#### Scenario: A\n*Verification*: **non-executable**\nprose\n";

    fn counter() -> impl FnMut() -> u64 {
        let mut n = 0u64;
        move || {
            n += 1;
            n << 48
        }
    }

    fn setup(text: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("design/specs");
        std::fs::create_dir_all(&root).expect("mkdir");
        std::fs::write(root.join("cap.md"), text).expect("write");
        (dir, root)
    }

    #[test]
    fn assigns_writes_ledger_and_is_idempotent() {
        let (_d, root) = setup(SPEC);
        let r = assign(&root, &[], None, false, &mut counter()).expect("assign");
        assert_eq!(r.ledger_added, ["r-0001", "s-0002"]);
        let got = std::fs::read_to_string(root.join("cap.md")).expect("read");
        assert_eq!(
            got,
            SPEC.replace("One\n", "One  {#r-0001}\n")
                .replace("A\n", "A  {#s-0002}\n")
        );
        assert_eq!(
            std::fs::read_to_string(root.join(".ids")).expect("ledger"),
            "r-0001\ns-0002\n"
        );
        let again = assign(&root, &[], None, false, &mut counter()).expect("assign");
        assert!(again.files.is_empty() && again.ledger_added.is_empty());
        assert_eq!(
            std::fs::read_to_string(root.join("cap.md")).expect("read"),
            got
        );
    }

    #[test]
    fn deleted_ids_are_not_reused() {
        let (_d, root) = setup(SPEC);
        assign(&root, &[], None, false, &mut counter()).expect("assign");
        // Delete the requirement, then add a new one; the generator would
        // offer r-0001 again first.
        std::fs::write(root.join("cap.md"), SPEC).expect("write");
        let r = assign(&root, &[], None, false, &mut counter()).expect("assign");
        assert_eq!(r.ledger_added, ["r-00010", "s-00020"]);
    }

    #[test]
    fn ids_in_files_missing_from_the_ledger_are_added() {
        let (_d, root) = setup(
            "### Requirement: One {#r-abcd}\nSHALL.\n#### Scenario: A {#s-1234}\n*Verification*: **non-executable**\n",
        );
        std::fs::write(root.join(".ids"), "r-abcd").expect("ledger");
        let r = assign(&root, &[], None, false, &mut counter()).expect("assign");
        assert!(r.files.is_empty());
        assert_eq!(
            std::fs::read_to_string(root.join(".ids")).expect("ledger"),
            "r-abcd\ns-1234\n"
        );
    }

    #[test]
    fn dry_run_changes_nothing() {
        let (_d, root) = setup(SPEC);
        let r = assign(&root, &[], None, true, &mut counter()).expect("assign");
        assert_eq!(r.files[0].assigned.len(), 2);
        assert_eq!(
            std::fs::read_to_string(root.join("cap.md")).expect("read"),
            SPEC
        );
        assert!(!root.join(".ids").exists());
    }

    #[test]
    fn duplicate_ids_across_files_are_an_error() {
        let (_d, root) = setup(
            "### Requirement: One {#r-abcd}\nSHALL.\n#### Scenario: A\n*Verification*: **non-executable**\n",
        );
        std::fs::copy(root.join("cap.md"), root.join("other.md")).expect("copy");
        let e = assign(&root, &[], None, false, &mut counter()).expect_err("dup");
        assert!(e.to_string().contains("duplicate id"), "{e}");
        assert!(!root.join(".ids").exists());
    }
}
