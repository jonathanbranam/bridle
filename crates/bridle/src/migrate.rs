//! `bridle migrate`: apply the project migrations this binary ships that a project hasn't had
//! yet (ticket xebc, docs/design/migrations.md). `bridle serve` runs the same code at start-up
//! (`run_at_startup`) unless the project sets `[migrations] auto = false`.
//!
//! The applied list lives in the project (`.bridle/migrations.toml`), so it travels with the
//! repo; each applied migration is also appended to `.bridle/migrations.log` for the human.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use bridle_api::types::MigrationRecord;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

pub struct Ctx<'a> {
    /// The project's repository root. A migration touches only bridle's own files under it.
    pub repo: &'a Path,
    /// Report what would change; write nothing.
    pub dry_run: bool,
}

#[derive(Debug)]
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
    /// Opt-in: skipped by the start-up run and by a plain `bridle migrate` (which lists it as
    /// pending-manual); runs only by id (`bridle migrate --only ID`).
    pub manual_only: bool,
}

/// What a migration returns (wrapped in `anyhow`) when the files it edits have uncommitted
/// changes. Not a failure: the start-up run skips and retries at the next start.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Refused(pub String);

pub fn is_refusal(e: &anyhow::Error) -> bool {
    e.downcast_ref::<Refused>().is_some()
}

/// Append only; ids never change or move once shipped.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        id: "0000-baseline",
        description: "Start tracking migrations in this project; changes nothing else.",
        manual_only: false,
        run: |_| {
            Ok(Report {
                files: vec![],
                summary: "baseline: nothing to change".into(),
            })
        },
    },
    Migration {
        id: "0001-rename-product-manager",
        description: "Rename the product-manager role to project-manager (ticket 9j2h).",
        // The human reviews it before it runs on a real project (br-9j2h).
        manual_only: true,
        run: rename_product_manager,
    },
];

const OLD_ROLE: &str = "product-manager";
const NEW_ROLE: &str = "project-manager";

/// Renames the role in `.bridle/config.toml` (the `[roles.*]` table and any path or value that
/// names it) and moves a `.bridle/roles/product-manager.md` override. The config doesn't keep
/// the old name as an alias: the smaller option, since a migration is how a project moves over.
fn rename_product_manager(ctx: &Ctx) -> anyhow::Result<Report> {
    let config_rel = ".bridle/config.toml";
    let old_role_rel = format!(".bridle/roles/{OLD_ROLE}.md");
    let new_role_rel = format!(".bridle/roles/{NEW_ROLE}.md");
    let config_path = ctx.repo.join(config_rel);
    let old_role = ctx.repo.join(&old_role_rel);
    let new_role = ctx.repo.join(&new_role_rel);

    let config = match std::fs::read_to_string(&config_path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("reading {}", config_path.display())),
    };
    let config_changes = config.as_ref().is_some_and(|s| s.contains(OLD_ROLE));
    let role_moves = old_role.exists();
    if role_moves && new_role.exists() {
        bail!("both {old_role_rel} and {new_role_rel} exist; merge them by hand, then rerun");
    }

    let mut files = vec![];
    if config_changes {
        files.push(config_rel.to_string());
    }
    if role_moves {
        files.push(old_role_rel.clone());
        files.push(new_role_rel.clone());
    }
    if files.is_empty() {
        return Ok(Report {
            files,
            summary: "no product-manager role in this project".into(),
        });
    }
    refuse_if_uncommitted(ctx.repo, &files)?;
    if !ctx.dry_run {
        if let (true, Some(s)) = (config_changes, &config) {
            std::fs::write(&config_path, s.replace(OLD_ROLE, NEW_ROLE))
                .with_context(|| format!("writing {}", config_path.display()))?;
        }
        if role_moves {
            std::fs::rename(&old_role, &new_role)
                .with_context(|| format!("moving {}", old_role.display()))?;
        }
    }
    Ok(Report {
        files,
        summary: format!("renamed {OLD_ROLE} to {NEW_ROLE}"),
    })
}

/// Refuses when git reports uncommitted changes in any of `files`, so a migration's edit is
/// never mixed with the human's. Outside a git repository there is nothing to check.
fn refuse_if_uncommitted(repo: &Path, files: &[String]) -> anyhow::Result<()> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain", "--"])
        .args(files)
        .output();
    let Ok(out) = out else { return Ok(()) };
    if !out.status.success() {
        return Ok(());
    }
    let dirty = String::from_utf8_lossy(&out.stdout);
    if !dirty.trim().is_empty() {
        return Err(Refused(format!(
            "uncommitted changes in files this migration edits; commit or stash them first:\n{}",
            dirty.trim_end()
        ))
        .into());
    }
    Ok(())
}

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
    /// Pending opt-in migrations that were left alone.
    pub manual_pending: Vec<String>,
}

/// Apply every pending migration in order, recording each (state file, then log) as it
/// succeeds so a later failure doesn't lose earlier work. A dry run writes nothing. Opt-in
/// migrations run only when named by `only`, which then runs just that one.
pub fn apply(
    repo: &Path,
    all: &[Migration],
    dry_run: bool,
    only: Option<&str>,
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
        manual_pending: vec![],
    };
    if let Some(id) = only
        && !all.iter().any(|m| m.id == id)
    {
        bail!("no migration {id}");
    }
    let ctx = Ctx { repo, dry_run };
    for m in pending(repo, all)? {
        if only.is_some_and(|id| id != m.id) {
            continue;
        }
        if m.manual_only && only.is_none() {
            out.manual_pending.push(m.id.to_string());
            continue;
        }
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

/// The start-up run: apply pending migrations unless `[migrations] auto = false`. Never
/// errors or panics out; `None` means nothing was attempted. A run that applied nothing and
/// failed nothing touched no file.
pub fn run_at_startup(repo: &Path, all: &[Migration]) -> Option<Result<Outcome, String>> {
    let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if !repo.join(".bridle").is_dir() {
            return None;
        }
        // An unreadable config is the daemon's own start-up error to report, not ours.
        if !bridle_daemon::config::Config::load(repo).is_ok_and(|c| c.migrations_auto) {
            return None;
        }
        Some(apply(repo, all, false, None, Utc::now()).map_err(|e| format!("{e:#}")))
    }));
    match attempt {
        Ok(r) => r,
        Err(_) => Some(Err("the migration run panicked".into())),
    }
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
            manual_only: false,
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
            manual_only: false,
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
        let out = apply(p.path(), &all, false, None, Utc::now()).unwrap();
        assert_eq!(ids(&out.done), ["0000-a", "0001-b"]);
        assert!(out.failed.is_none());
        let state = load_state(p.path()).unwrap();
        assert_eq!(state.applied.len(), 2);
        assert_eq!(state.applied[0].id, "0000-a");
        let log = std::fs::read_to_string(log_path(p.path())).unwrap();
        assert_eq!(log.lines().count(), 2);
        assert!(log.contains("0001-b") && log.contains("files: touched"));

        let again = apply(p.path(), &all, false, None, Utc::now()).unwrap();
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
        let out = apply(p.path(), &all, true, None, Utc::now()).unwrap();
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
        let out = apply(p.path(), &all, false, None, Utc::now()).unwrap();
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
        apply(a.path(), &all, false, None, Utc::now()).unwrap();
        assert!(pending(a.path(), &all).unwrap().is_empty());
        assert_eq!(pending(b.path(), &all).unwrap().len(), 1);
    }

    #[test]
    fn refuses_a_directory_that_is_not_a_project() {
        let d = tempfile::tempdir().unwrap();
        assert!(apply(d.path(), MIGRATIONS, false, None, Utc::now()).is_err());
    }

    fn manual(id: &'static str) -> Migration {
        Migration {
            manual_only: true,
            ..ok_migration(id)
        }
    }

    fn refusing(id: &'static str) -> Migration {
        Migration {
            id,
            description: "refuses",
            manual_only: false,
            run: |_| Err(Refused("uncommitted changes in x".into()).into()),
        }
    }

    #[test]
    fn startup_applies_pending_and_records() {
        let p = project();
        let all = [ok_migration("0000-a")];
        let out = run_at_startup(p.path(), &all).unwrap().unwrap();
        assert_eq!(ids(&out.done), ["0000-a"]);
        assert_eq!(load_state(p.path()).unwrap().applied.len(), 1);
        assert!(log_path(p.path()).exists());
    }

    #[test]
    fn startup_with_nothing_pending_touches_no_file() {
        let p = project();
        let all = [ok_migration("0000-a")];
        run_at_startup(p.path(), &all);
        let before = std::fs::read_to_string(state_path(p.path())).unwrap();
        let log = std::fs::read_to_string(log_path(p.path())).unwrap();
        let mtime = std::fs::metadata(state_path(p.path()))
            .unwrap()
            .modified()
            .unwrap();
        let out = run_at_startup(p.path(), &all).unwrap().unwrap();
        assert!(out.done.is_empty() && out.failed.is_none());
        assert_eq!(
            std::fs::read_to_string(state_path(p.path())).unwrap(),
            before
        );
        assert_eq!(std::fs::read_to_string(log_path(p.path())).unwrap(), log);
        assert_eq!(
            std::fs::metadata(state_path(p.path()))
                .unwrap()
                .modified()
                .unwrap(),
            mtime
        );
    }

    #[test]
    fn auto_false_leaves_it_pending() {
        let p = project();
        std::fs::write(
            p.path().join(".bridle/config.toml"),
            "[migrations]\nauto = false\n",
        )
        .unwrap();
        let all = [ok_migration("0000-a")];
        assert!(run_at_startup(p.path(), &all).is_none());
        assert_eq!(pending(p.path(), &all).unwrap().len(), 1);
        assert!(!state_path(p.path()).exists());
    }

    #[test]
    fn startup_skips_opt_in_migrations_and_apply_only_runs_one_by_id() {
        let p = project();
        let all = [
            ok_migration("0000-a"),
            manual("0001-m"),
            ok_migration("0002-c"),
        ];
        let out = run_at_startup(p.path(), &all).unwrap().unwrap();
        assert_eq!(ids(&out.done), ["0000-a", "0002-c"]);
        assert_eq!(out.manual_pending, ["0001-m"]);
        let out = apply(p.path(), &all, false, Some("0001-m"), Utc::now()).unwrap();
        assert_eq!(ids(&out.done), ["0001-m"]);
        assert!(pending(p.path(), &all).unwrap().is_empty());
        assert!(apply(p.path(), &all, false, Some("9999-x"), Utc::now()).is_err());
    }

    #[test]
    fn startup_failure_is_returned_not_raised_and_stops_the_run() {
        let p = project();
        let all = [failing("0000-a"), ok_migration("0001-b")];
        let out = run_at_startup(p.path(), &all).unwrap().unwrap();
        assert!(out.done.is_empty());
        let (id, e) = out.failed.unwrap();
        assert_eq!(id, "0000-a");
        assert!(!is_refusal(&e));
        assert!(!state_path(p.path()).exists());
    }

    #[test]
    fn startup_survives_a_panicking_migration() {
        let p = project();
        let all = [Migration {
            id: "0000-p",
            description: "panics",
            manual_only: false,
            run: |_| panic!("boom"),
        }];
        assert!(run_at_startup(p.path(), &all).unwrap().is_err());
    }

    #[test]
    fn a_refusal_is_recognised_and_recorded_nowhere() {
        let p = project();
        let all = [refusing("0000-a")];
        let out = run_at_startup(p.path(), &all).unwrap().unwrap();
        assert!(is_refusal(&out.failed.unwrap().1));
        assert_eq!(pending(p.path(), &all).unwrap().len(), 1);
    }

    fn git_project(config: &str, role_file: bool) -> tempfile::TempDir {
        let p = project();
        std::fs::write(p.path().join(".bridle/config.toml"), config).unwrap();
        if role_file {
            std::fs::create_dir(p.path().join(".bridle/roles")).unwrap();
            std::fs::write(p.path().join(".bridle/roles/product-manager.md"), "# PM\n").unwrap();
        }
        let git = |args: &[&str]| {
            let ok = std::process::Command::new("git")
                .arg("-C")
                .arg(p.path())
                .args(["-c", "user.name=t", "-c", "user.email=t@t"])
                .args(args)
                .output()
                .unwrap()
                .status
                .success();
            assert!(ok, "git {args:?}");
        };
        git(&["init", "-q"]);
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "init"]);
        p
    }

    const OLD_CONFIG: &str =
        "[roles.product-manager]\nsystem_prompt = \"workflow/base/roles/product-manager.md\"\n";

    #[test]
    fn rename_migration_moves_config_and_role_file_and_is_idempotent() {
        let p = git_project(OLD_CONFIG, true);
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        let r = rename_product_manager(&ctx).unwrap();
        assert_eq!(r.files.len(), 3);
        let cfg = std::fs::read_to_string(p.path().join(".bridle/config.toml")).unwrap();
        assert_eq!(
            cfg,
            "[roles.project-manager]\nsystem_prompt = \"workflow/base/roles/project-manager.md\"\n"
        );
        assert!(!p.path().join(".bridle/roles/product-manager.md").exists());
        assert!(p.path().join(".bridle/roles/project-manager.md").exists());
        // The changes are now uncommitted, so a second run must find nothing to do (and so
        // not refuse).
        let again = rename_product_manager(&ctx).unwrap();
        assert!(again.files.is_empty());
    }

    #[test]
    fn rename_migration_dry_run_writes_nothing() {
        let p = git_project(OLD_CONFIG, true);
        let ctx = Ctx {
            repo: p.path(),
            dry_run: true,
        };
        assert_eq!(rename_product_manager(&ctx).unwrap().files.len(), 3);
        assert_eq!(
            std::fs::read_to_string(p.path().join(".bridle/config.toml")).unwrap(),
            OLD_CONFIG
        );
        assert!(p.path().join(".bridle/roles/product-manager.md").exists());
    }

    #[test]
    fn rename_migration_refuses_uncommitted_edits() {
        let p = git_project(OLD_CONFIG, false);
        let edited = format!("{OLD_CONFIG}# mine\n");
        std::fs::write(p.path().join(".bridle/config.toml"), &edited).unwrap();
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        let err = rename_product_manager(&ctx).unwrap_err().to_string();
        assert!(err.contains("uncommitted"), "{err}");
        assert_eq!(
            std::fs::read_to_string(p.path().join(".bridle/config.toml")).unwrap(),
            edited
        );
    }

    #[test]
    fn rename_migration_leaves_a_project_without_the_role_alone() {
        let p = git_project("[roles.worker]\nmodel = \"sonnet\"\n", false);
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        assert!(rename_product_manager(&ctx).unwrap().files.is_empty());
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
