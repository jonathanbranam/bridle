//! `bridle migrate`: apply the project migrations this binary ships that a project hasn't had
//! yet (ticket xebc, docs/design/migrations.md). Never run automatically.
//!
//! The applied list lives in the project (`.bridle/migrations.toml`), so it travels with the
//! repo; each applied migration is also appended to `.bridle/migrations.log` for the human.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use bridle_api::types::MigrationRecord;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

// Read by real migrations (the baseline ignores its context), and by the tests.
#[cfg_attr(not(test), expect(dead_code))]
pub struct Ctx<'a> {
    /// The project's repository root. A migration touches only bridle's own files under it.
    pub repo: &'a Path,
    /// Report what would change; write nothing.
    pub dry_run: bool,
}

pub struct Report {
    /// Paths relative to the repo root.
    pub files: Vec<String>,
    pub summary: String,
}

pub struct Migration {
    /// `NNNN-short-name`; applied in list order.
    pub id: &'static str,
    pub description: &'static str,
    /// Must be idempotent, and under `ctx.dry_run` must write nothing.
    pub run: fn(&Ctx) -> anyhow::Result<Report>,
}

/// Append only; ids never change or move once shipped.
pub const MIGRATIONS: &[Migration] = &[Migration {
    id: "0000-baseline",
    description: "Start tracking migrations in this project; changes nothing else.",
    run: |_| {
        Ok(Report {
            files: vec![],
            summary: "baseline: nothing to change".into(),
        })
    },
}];

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    #[serde(default)]
    applied: Vec<Applied>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Applied {
    id: String,
    applied_at: DateTime<Utc>,
    bridle_version: String,
}

fn state_path(repo: &Path) -> PathBuf {
    repo.join(".bridle/migrations.toml")
}

fn log_path(repo: &Path) -> PathBuf {
    repo.join(".bridle/migrations.log")
}

fn load_state(repo: &Path) -> anyhow::Result<State> {
    let path = state_path(repo);
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).with_context(|| format!("parsing {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// The migrations not yet applied to `repo`, in order.
pub fn pending<'a>(repo: &Path, all: &'a [Migration]) -> anyhow::Result<Vec<&'a Migration>> {
    let state = load_state(repo)?;
    Ok(all
        .iter()
        .filter(|m| !state.applied.iter().any(|a| a.id == m.id))
        .collect())
}

pub struct Outcome {
    /// What ran (or, in a dry run, would run), in order.
    pub done: Vec<MigrationRecord>,
    /// The migration that failed; nothing is recorded for it and nothing after it ran.
    pub failed: Option<(String, anyhow::Error)>,
}

/// Apply every pending migration in order, recording each (state file, then log) as it
/// succeeds so a later failure doesn't lose earlier work. A dry run writes nothing.
pub fn apply(
    repo: &Path,
    all: &[Migration],
    dry_run: bool,
    now: DateTime<Utc>,
) -> anyhow::Result<Outcome> {
    if !repo.join(".bridle").is_dir() {
        bail!(
            "{} has no .bridle directory: not a bridle project",
            repo.display()
        );
    }
    let mut out = Outcome {
        done: vec![],
        failed: None,
    };
    let ctx = Ctx { repo, dry_run };
    for m in pending(repo, all)? {
        let report = match (m.run)(&ctx) {
            Ok(r) => r,
            Err(e) => {
                out.failed = Some((m.id.to_string(), e));
                break;
            }
        };
        let rec = MigrationRecord {
            id: m.id.to_string(),
            files: report.files,
            summary: report.summary,
        };
        if !dry_run {
            record(repo, m.description, &rec, now)?;
        }
        out.done.push(rec);
    }
    Ok(out)
}

fn record(
    repo: &Path,
    description: &str,
    rec: &MigrationRecord,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let mut state = load_state(repo)?;
    state.applied.push(Applied {
        id: rec.id.clone(),
        applied_at: now,
        bridle_version: env!("CARGO_PKG_VERSION").to_string(),
    });
    let path = state_path(repo);
    std::fs::write(&path, toml::to_string(&state)?)
        .with_context(|| format!("writing {}", path.display()))?;

    let path = log_path(repo);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening {}", path.display()))?;
    let files = if rec.files.is_empty() {
        "no files changed".to_string()
    } else {
        format!("files: {}", rec.files.join(", "))
    };
    writeln!(
        f,
        "{} {} (bridle {}): {description} {}; {files}",
        now.to_rfc3339_opts(SecondsFormat::Secs, true),
        rec.id,
        env!("CARGO_PKG_VERSION"),
        rec.summary,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join(".bridle")).unwrap();
        d
    }

    fn ok_migration(id: &'static str) -> Migration {
        Migration {
            id,
            description: "test",
            run: |ctx| {
                if !ctx.dry_run {
                    std::fs::write(ctx.repo.join("touched"), "x")?;
                }
                Ok(Report {
                    files: vec!["touched".into()],
                    summary: "touched".into(),
                })
            },
        }
    }

    fn failing(id: &'static str) -> Migration {
        Migration {
            id,
            description: "fails",
            run: |_| bail!("boom"),
        }
    }

    fn ids(v: &[MigrationRecord]) -> Vec<&str> {
        v.iter().map(|r| r.id.as_str()).collect()
    }

    #[test]
    fn pending_comes_from_migrations_toml() {
        let p = project();
        let all = [ok_migration("0000-a"), ok_migration("0001-b")];
        assert_eq!(pending(p.path(), &all).unwrap().len(), 2);
        std::fs::write(
            state_path(p.path()),
            "[[applied]]\nid = \"0000-a\"\napplied_at = \"2026-01-01T00:00:00Z\"\nbridle_version = \"0.1.0\"\n",
        )
        .unwrap();
        let left = pending(p.path(), &all).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].id, "0001-b");
    }

    #[test]
    fn applies_in_order_records_and_second_run_is_a_noop() {
        let p = project();
        let all = [ok_migration("0000-a"), ok_migration("0001-b")];
        let out = apply(p.path(), &all, false, Utc::now()).unwrap();
        assert_eq!(ids(&out.done), ["0000-a", "0001-b"]);
        assert!(out.failed.is_none());
        let state = load_state(p.path()).unwrap();
        assert_eq!(state.applied.len(), 2);
        assert_eq!(state.applied[0].id, "0000-a");
        let log = std::fs::read_to_string(log_path(p.path())).unwrap();
        assert_eq!(log.lines().count(), 2);
        assert!(log.contains("0001-b") && log.contains("files: touched"));

        let again = apply(p.path(), &all, false, Utc::now()).unwrap();
        assert!(again.done.is_empty());
        assert_eq!(load_state(p.path()).unwrap().applied.len(), 2);
        assert_eq!(
            std::fs::read_to_string(log_path(p.path()))
                .unwrap()
                .lines()
                .count(),
            2
        );
    }

    #[test]
    fn dry_run_writes_nothing() {
        let p = project();
        let all = [ok_migration("0000-a")];
        let out = apply(p.path(), &all, true, Utc::now()).unwrap();
        assert_eq!(ids(&out.done), ["0000-a"]);
        assert!(!state_path(p.path()).exists());
        assert!(!log_path(p.path()).exists());
        assert!(!p.path().join("touched").exists());
    }

    #[test]
    fn a_failure_records_nothing_for_itself_and_stops() {
        let p = project();
        let all = [
            ok_migration("0000-a"),
            failing("0001-b"),
            ok_migration("0002-c"),
        ];
        let out = apply(p.path(), &all, false, Utc::now()).unwrap();
        assert_eq!(ids(&out.done), ["0000-a"]);
        assert_eq!(out.failed.as_ref().unwrap().0, "0001-b");
        let state = load_state(p.path()).unwrap();
        assert_eq!(state.applied.len(), 1);
        assert_eq!(state.applied[0].id, "0000-a");
    }

    #[test]
    fn two_projects_are_independent() {
        let (a, b) = (project(), project());
        let all = [ok_migration("0000-a")];
        apply(a.path(), &all, false, Utc::now()).unwrap();
        assert!(pending(a.path(), &all).unwrap().is_empty());
        assert_eq!(pending(b.path(), &all).unwrap().len(), 1);
    }

    #[test]
    fn refuses_a_directory_that_is_not_a_project() {
        let d = tempfile::tempdir().unwrap();
        assert!(apply(d.path(), MIGRATIONS, false, Utc::now()).is_err());
    }

    #[test]
    fn shipped_ids_are_ordered_and_unique() {
        let ids: Vec<_> = MIGRATIONS.iter().map(|m| m.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids, sorted);
    }
}
