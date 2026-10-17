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
    Migration {
        id: "0002-standard-ticket-fields",
        description: "Rename ticket frontmatter fields to the standard names and qualify ticket ids with the project prefix (ticket syqn).",
        // Rewrites many files of the human's project: they review it first, on a trial branch.
        manual_only: true,
        run: standard_ticket_fields,
    },
    Migration {
        id: "0003-bug-tickets-that-are-fixes",
        description: "Make `kind: bug` tickets that have a task `kind: fix`; a bug with none stays a report (ticket bpku).",
        // Rewrites many files of the human's project: they review it first, on a trial branch.
        manual_only: true,
        run: bug_tickets_that_are_fixes,
    },
    Migration {
        id: "0004-ticket-links-in-frontmatter",
        description: "Carry the daemon's blocks, parent and related edges between tickets into their frontmatter and drop the `tasks:` list (ticket 3v75).",
        // Rewrites many files of the human's project: they review it first, on a trial branch.
        manual_only: true,
        run: ticket_links_in_frontmatter,
    },
    Migration {
        id: "0005-a-ticket-for-every-active-task",
        description: "Give every active task without a ticket a stub ticket, and every extra task of a ticket a child ticket, so each piece of work has one (ticket 72t9).",
        // Writes ticket files into the human's project and edits the daemon's rows: they review it first, on a trial branch.
        manual_only: true,
        run: a_ticket_for_every_active_task,
    },
    Migration {
        id: "0006-backfill-ticket-kind",
        description: "Give every ticket without a `kind:` the kind of its task row, else `chore` (ticket v3dk).",
        // Automatic (the human approved it, br-e7e2); the protected-branch guard still keeps it
        // off an existing project's main, and it skips files with uncommitted changes.
        manual_only: false,
        run: backfill_ticket_kind,
    },
    Migration {
        id: "0007-gitignore-agent-site",
        description: "Add `.agent-site/` (throwaway dev sites) to .gitignore (ticket tgxy).",
        // Automatic: one line, appended only if absent; the guard and the uncommitted-changes
        // refusal still apply.
        manual_only: false,
        run: gitignore_agent_site,
    },
];

const AGENT_SITE_LINE: &str = ".agent-site/";

/// Appends `.agent-site/` to `.gitignore` unless a line already says so, so a rerun finds nothing.
fn gitignore_agent_site(ctx: &Ctx) -> anyhow::Result<Report> {
    let project = project_of(ctx.repo);
    let path = ctx.repo.join(".gitignore");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    let has = text.lines().any(|l| {
        matches!(
            l.trim(),
            ".agent-site" | ".agent-site/" | "/.agent-site" | "/.agent-site/"
        )
    });
    if has {
        return Ok(Report {
            files: vec![],
            summary: ".gitignore already ignores .agent-site/".into(),
        });
    }
    let files = vec![".gitignore".to_string()];
    refuse_on_protected_branch(ctx.repo, &project)?;
    refuse_if_uncommitted(ctx.repo, &files)?;
    if !ctx.dry_run {
        let mut new = text;
        if !new.is_empty() && !new.ends_with('\n') {
            new.push('\n');
        }
        new.push_str(AGENT_SITE_LINE);
        new.push('\n');
        std::fs::write(&path, new).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(Report {
        files,
        summary: format!("added {AGENT_SITE_LINE} to .gitignore"),
    })
}

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

/// Rewrites every ticket under `docs/tickets/` to the standard frontmatter (syqn): `needs`,
/// `see`, `opened`, `filed_by`, `closed` become `blocked_by`, `related`, `created`,
/// `created_by`, `resolved`, and bare ids become `<prefix>-<id>`. Tickets already in the new
/// form are left alone, so a rerun finds nothing. The old forms stay readable until every
/// project has run this.
fn standard_ticket_fields(ctx: &Ctx) -> anyhow::Result<Report> {
    let project = project_of(ctx.repo);
    let prefix = match bridle_daemon::config::Config::load(ctx.repo) {
        Ok(c) => c.task_prefix,
        Err(_) => None,
    }
    .unwrap_or_else(|| bridle_daemon::config::default_task_prefix(&project));
    let root = crate::ticket::tickets_root(ctx.repo);
    let mut paths = vec![];
    md_files(&root, &mut paths);
    paths.sort();
    let mut edits = vec![];
    for p in paths {
        let text =
            std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
        if let Some(new) = crate::ticket::standardize_front(&text, &prefix, &root) {
            edits.push((p, new));
        }
    }
    let files: Vec<String> = edits
        .iter()
        .map(|(p, _)| {
            p.strip_prefix(ctx.repo)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    if files.is_empty() {
        return Ok(Report {
            files,
            summary: "every ticket already has the standard fields".into(),
        });
    }
    refuse_on_protected_branch(ctx.repo, &project)?;
    refuse_if_uncommitted(ctx.repo, &files)?;
    if !ctx.dry_run {
        for (p, new) in &edits {
            std::fs::write(p, new).with_context(|| format!("writing {}", p.display()))?;
        }
    }
    Ok(Report {
        summary: format!("standardized {} tickets (prefix {prefix})", files.len()),
        files,
    })
}

/// `bug` is a report and `fix` the change that fixes it (bpku): a `bug` ticket with a task was
/// worked as a fix, so it becomes one. Tickets already `fix`, and bugs nobody worked, are left
/// alone, so a rerun finds nothing. The daemon's rows are not touched: an open row changes with
/// `bridle task kind`, an integrated one stays as it was.
fn bug_tickets_that_are_fixes(ctx: &Ctx) -> anyhow::Result<Report> {
    let project = project_of(ctx.repo);
    let root = crate::ticket::tickets_root(ctx.repo);
    let mut paths = vec![];
    md_files(&root, &mut paths);
    paths.sort();
    let mut edits = vec![];
    for p in paths {
        let text =
            std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
        if let Some(new) = crate::ticket::bug_to_fix(&text) {
            edits.push((p, new));
        }
    }
    let files: Vec<String> = edits
        .iter()
        .map(|(p, _)| {
            p.strip_prefix(ctx.repo)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    if files.is_empty() {
        return Ok(Report {
            files,
            summary: "no bug ticket with a task".into(),
        });
    }
    refuse_on_protected_branch(ctx.repo, &project)?;
    refuse_if_uncommitted(ctx.repo, &files)?;
    if !ctx.dry_run {
        for (p, new) in &edits {
            std::fs::write(p, new).with_context(|| format!("writing {}", p.display()))?;
        }
    }
    Ok(Report {
        summary: format!("{} bug tickets became fix", files.len()),
        files,
    })
}

/// Links live only in the frontmatter (3v75): each `blocks`, `parent` and `related` edge between
/// two tickets of the project, read from the state branch (`edges.toml`, and each task's
/// `ticket` field), is written into the ticket that owns it (`blocked_by`, `parent`, `related`),
/// and the `tasks:` list goes. Edges that don't join two tickets here, and `duplicates`,
/// `supersedes` and `discovered-from`, stay edges. A rerun finds nothing to add.
fn ticket_links_in_frontmatter(ctx: &Ctx) -> anyhow::Result<Report> {
    let git_show = |path: &str| -> Option<String> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(ctx.repo)
            .args(["show", &format!("bridle/state:{path}")])
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let edges = git_show("edges.toml").unwrap_or_default();
    let mut rows = std::collections::HashMap::new();
    let listing = std::process::Command::new("git")
        .arg("-C")
        .arg(ctx.repo)
        .args(["ls-tree", "--name-only", "bridle/state", "tasks/"])
        .output();
    if let Ok(l) = listing {
        for path in String::from_utf8_lossy(&l.stdout).lines() {
            if let Some(text) = git_show(path) {
                let front = text.split("+++").nth(1).unwrap_or_default();
                let get = |key: &str| {
                    front
                        .lines()
                        .find_map(|l| l.strip_prefix(&format!("{key} = \"")))
                        .and_then(|v| v.strip_suffix('"'))
                        .map(str::to_string)
                };
                if let (Some(id), Some(ticket)) = (get("id"), get("ticket")) {
                    rows.insert(id, ticket);
                }
            }
        }
    }
    carry_links(ctx, &edges, &rows)
}

/// A task row as the state branch holds it: what the 0005 migration needs.
#[derive(Debug)]
struct Row {
    id: String,
    state: String,
    ticket: Option<String>,
    created_at: String,
    kind: Option<String>,
}

/// The rows of `tasks/*.md` on the `bridle/state` branch, read with git (the daemon owns the
/// worktree, so nothing is read from it).
fn state_rows(repo: &Path) -> Vec<Row> {
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
    };
    // `tasks/` is still read for one release (95t9); `tracking/` is where touched rows move.
    let listing = ["tracking/", "tasks/"]
        .iter()
        .filter_map(|d| git(&["ls-tree", "--name-only", "bridle/state", d]))
        .collect::<String>();
    listing
        .lines()
        .filter_map(|path| {
            let text = git(&["show", &format!("bridle/state:{path}")])?;
            let front: toml::Table = text.split("+++\n").nth(1)?.parse().ok()?;
            let get = |k: &str| front.get(k).and_then(|v| v.as_str()).map(str::to_string);
            Some(Row {
                id: get("id")?,
                state: get("state")?,
                ticket: get("ticket"),
                created_at: get("created_at").unwrap_or_default(),
                kind: get("kind"),
            })
        })
        .collect()
}

/// The kind a ticket without one gets: that of its own row (same id), else of a row that names it
/// in `ticket`, else `chore` (the human edits it with `ticket set`). Only a kind the CLI knows
/// counts. Rows come from the state branch, so a stopped daemon changes nothing.
fn kind_for(rows: &[Row], id: &str) -> String {
    let bare = |s: &str| s.rsplit('-').next().unwrap_or(s).to_string();
    let want = bare(id);
    fn known(r: &Row) -> Option<&str> {
        r.kind
            .as_deref()
            .filter(|k| k.parse::<bridle_api::TicketKind>().is_ok())
    }
    let own = rows.iter().filter(|r| bare(&r.id) == want).find_map(known);
    let named = || {
        rows.iter()
            .filter(|r| r.ticket.as_deref().map(bare).as_deref() == Some(want.as_str()))
            .find_map(known)
    };
    own.or_else(named).unwrap_or("chore").to_string()
}

/// Adds the missing `kind:` to every ticket (0006), so `ticket check` can require it. Tickets
/// that have one are left alone, so a rerun finds nothing. Spikes under `docs/spikes` are not
/// ticket-checked and are left for now.
fn backfill_ticket_kind(ctx: &Ctx) -> anyhow::Result<Report> {
    let project = project_of(ctx.repo);
    let rows = state_rows(ctx.repo);
    let root = crate::ticket::tickets_root(ctx.repo);
    let mut paths = vec![];
    md_files(&root, &mut paths);
    paths.sort();
    let mut edits = vec![];
    for p in paths {
        let text =
            std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
        if let Some(new) = crate::ticket::backfill_kind(&text, |id| kind_for(&rows, id)) {
            edits.push((p, new));
        }
    }
    let files: Vec<String> = edits
        .iter()
        .map(|(p, _)| {
            p.strip_prefix(ctx.repo)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    if files.is_empty() {
        return Ok(Report {
            files,
            summary: "every ticket already has a kind".into(),
        });
    }
    refuse_on_protected_branch(ctx.repo, &project)?;
    refuse_if_uncommitted(ctx.repo, &files)?;
    if !ctx.dry_run {
        for (p, new) in &edits {
            std::fs::write(p, new).with_context(|| format!("writing {}", p.display()))?;
        }
    }
    Ok(Report {
        summary: format!("gave {} tickets a kind", files.len()),
        files,
    })
}

fn a_ticket_for_every_active_task(ctx: &Ctx) -> anyhow::Result<Report> {
    let project = project_of(ctx.repo);
    let mut rows = state_rows(ctx.repo);
    rows.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    let plan = plan_tickets(ctx.repo, &rows);
    let summary = |n: usize, what: &str| format!("{n} {what}");
    if plan.is_empty() {
        return Ok(Report {
            files: vec![],
            summary: "every active task already has a ticket".into(),
        });
    }
    refuse_on_protected_branch(ctx.repo, &project)?;
    if !ctx.dry_run {
        let exe = std::env::current_exe().context("finding the bridle binary")?;
        for (task, parent) in &plan {
            // `ticket new --from-task` writes the file and links the row to it through the
            // project's daemon, as it does by hand.
            let out = std::process::Command::new(&exe)
                .current_dir(ctx.repo)
                .args(["ticket", "new", "--from-task", task])
                .output()
                .context("running bridle ticket new")?;
            if !out.status.success() {
                bail!(
                    "bridle ticket new --from-task {task}: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            if let Some(parent) = parent {
                let rel = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let child = crate::ticket::id_of(Path::new(&rel))
                    .ok_or_else(|| anyhow::anyhow!("no ticket id in {rel:?}"))?;
                let out = std::process::Command::new(&exe)
                    .current_dir(ctx.repo)
                    .args(["ticket", "link", &child, "parent", parent])
                    .output()
                    .context("running bridle ticket link")?;
                if !out.status.success() {
                    bail!(
                        "bridle ticket link {child} parent {parent}: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    );
                }
            }
        }
    }
    let children = plan.iter().filter(|(_, p)| p.is_some()).count();
    Ok(Report {
        files: plan
            .iter()
            .map(|(t, _)| format!("ticket for task {t}"))
            .collect(),
        summary: format!(
            "{}, {}",
            summary(plan.len() - children, "stub tickets"),
            summary(children, "child tickets")
        ),
    })
}

/// The active tasks that need a ticket made: (task id, `Some(parent ticket)` for an extra task
/// of a ticket that has another). A task whose `ticket` names a file that exists, and that is
/// its ticket's first task, is fine; finished tasks stay read-only history. `rows` is oldest
/// first.
fn plan_tickets(repo: &Path, rows: &[Row]) -> Vec<(String, Option<String>)> {
    use crate::ticket::split_id;
    let root = crate::ticket::tickets_root(repo);
    let mut paths = vec![];
    md_files(&root, &mut paths);
    // Ticket tail -> qualified id.
    let known: std::collections::HashMap<String, String> = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .filter_map(|t| {
            t.lines()
                .find_map(|l| l.strip_prefix("id: "))
                .map(|i| i.trim().trim_matches('"').to_string())
        })
        .map(|id| (split_id(&id).1.to_string(), id))
        .collect();
    let ticket_of = |r: &Row| {
        r.ticket
            .as_deref()
            .and_then(|t| known.get(split_id(t).1))
            .cloned()
    };
    // The first task of a ticket is the one with its id, else the oldest: it keeps the ticket.
    let mut first: std::collections::HashMap<String, &str> = Default::default();
    for r in rows {
        if let Some(t) = ticket_of(r) {
            let slot = first.entry(t.clone()).or_insert(&r.id);
            if split_id(&r.id).1 == split_id(&t).1 {
                *slot = &r.id;
            }
        }
    }
    let active = |r: &&Row| !matches!(r.state.as_str(), "integrated" | "dropped");
    rows.iter()
        .filter(active)
        .filter_map(|r| match ticket_of(r) {
            None => Some((r.id.clone(), None)),
            Some(t) if first.get(&t) != Some(&r.id.as_str()) => Some((r.id.clone(), Some(t))),
            Some(_) => None,
        })
        .collect()
}

/// [`ticket_links_in_frontmatter`] given the state branch's `edges.toml` text and the rows'
/// `ticket` fields (row id -> ticket id).
fn carry_links(
    ctx: &Ctx,
    edges_toml: &str,
    row_ticket: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Report> {
    use crate::cli::LinkKind;
    use crate::ticket::{Projects, find_by_id, split_id};
    let root = crate::ticket::tickets_root(ctx.repo);
    // The project's tickets: bare id -> qualified id.
    let mut paths = vec![];
    md_files(&root, &mut paths);
    paths.sort();
    let mut known = std::collections::HashMap::new();
    for p in &paths {
        if let Ok(text) = std::fs::read_to_string(p)
            && let Some(id) = text
                .lines()
                .find_map(|l| l.strip_prefix("id: "))
                .map(|i| i.trim().trim_matches('"').to_string())
        {
            known.insert(split_id(&id).1.to_string(), id);
        }
    }
    let ticket_for = |row: &str| -> Option<String> {
        let t = row_ticket.get(row).map(String::as_str).unwrap_or(row);
        known.get(split_id(t).1).cloned()
    };
    let own = known
        .values()
        .find_map(|q| split_id(q).0.map(str::to_string));
    let projects = Projects {
        own,
        others: vec![],
    };
    // (ticket tail, link kind, other ticket)
    let mut adds: Vec<(String, LinkKind, String)> = vec![];
    for block in edges_toml.split("[[edge]]").skip(1) {
        let get = |key: &str| {
            block
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{key} = \"")))
                .and_then(|v| v.strip_suffix('"'))
        };
        let (Some(from), Some(to), Some(kind)) = (get("from"), get("to"), get("kind")) else {
            continue;
        };
        let (Some(a), Some(b)) = (ticket_for(from), ticket_for(to)) else {
            continue;
        };
        let (a_tail, b_tail) = (split_id(&a).1.to_string(), split_id(&b).1.to_string());
        match kind {
            "blocks" => adds.push((b_tail, LinkKind::BlockedBy, a)),
            "parent" => adds.push((b_tail, LinkKind::Parent, a)),
            "related" => adds.push((a_tail, LinkKind::Related, b)),
            _ => {}
        }
    }
    // What would change, per file, applied to a copy of the text so a dry run writes nothing.
    let mut changed: std::collections::BTreeMap<PathBuf, String> = Default::default();
    for (tail, kind, other) in &adds {
        let Ok(path) = find_by_id(&root.join("open"), tail)
            .or_else(|_| find_by_id(&root.join("resolved"), tail))
        else {
            continue;
        };
        let text = match changed.get(&path) {
            Some(t) => t.clone(),
            None => std::fs::read_to_string(&path)?,
        };
        if *kind == LinkKind::Parent && text.lines().any(|l| l.starts_with("parent:")) {
            continue;
        }
        if let Some(new) = crate::ticket::link_text(&text, *kind, other, &projects, &root)? {
            changed.insert(path, new);
        }
    }
    for p in &paths {
        let text = match changed.get(p) {
            Some(t) => t.clone(),
            None => std::fs::read_to_string(p)?,
        };
        let dropped = crate::ticket::drop_line(&text, "tasks");
        if dropped != text {
            changed.insert(p.clone(), dropped);
        }
    }
    let files: Vec<String> = changed
        .keys()
        .map(|p| {
            p.strip_prefix(ctx.repo)
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    if files.is_empty() {
        return Ok(Report {
            files,
            summary: "no ticket links to carry".into(),
        });
    }
    refuse_on_protected_branch(ctx.repo, &project_of(ctx.repo))?;
    refuse_if_uncommitted(ctx.repo, &files)?;
    if !ctx.dry_run {
        for (p, new) in &changed {
            std::fs::write(p, new).with_context(|| format!("writing {}", p.display()))?;
        }
    }
    Ok(Report {
        summary: format!("{} tickets updated", files.len()),
        files,
    })
}

fn md_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.filter_map(Result::ok) {
        let p = e.path();
        if p.is_dir() {
            md_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "md") {
            out.push(p);
        }
    }
}

/// The project's name, as the CLI resolves it: the running workspace's, else the directory name.
fn project_of(repo: &Path) -> String {
    crate::project::from_cwd(repo)
        .or_else(|| repo.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

/// The first protected-branch guard (rule existing-projects): a migration that rewrites a
/// project's files refuses while `main`, `dev` or the configured `[branches] release` is checked
/// out, unless the project declares that branch its `[branches] integration` (and it is not also
/// the release branch). Migrations call it only once they have something to change. Bridle's own repo is not an
/// existing project in that sense, and a repo without git has no branch to protect.
fn refuse_on_protected_branch(repo: &Path, project: &str) -> anyhow::Result<()> {
    if project == "bridle" {
        return Ok(());
    }
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["branch", "--show-current"])
        .output();
    let Ok(out) = out else { return Ok(()) };
    if !out.status.success() {
        return Ok(());
    }
    let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let branches = bridle_daemon::config::Config::load(repo)
        .ok()
        .map(|c| c.branches);
    let release = branches.as_ref().and_then(|b| b.release.clone());
    let own_integration = branches
        .as_ref()
        .is_some_and(|b| b.integration_set && b.integration == branch);
    let is_release = release.as_deref() == Some(branch.as_str());
    if is_release || (!own_integration && (branch == "main" || branch == "dev")) {
        return Err(Refused(format!(
            "{branch} is a protected branch of {project}; check out the project's trial branch (e.g. bridle-adopt) and rerun"
        ))
        .into());
    }
    Ok(())
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

    fn git(p: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(p)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    const OLD_TICKET: &str = "---\nid: abk2\ntitle: \"Old\"\nopened: 2026-01-01\nrepos: [x]\nchanges: []\nspecs: []\nneeds: [cdk4, zzzz]\nsee: []\nparent: cdk4\n---\nbody\n";
    fn other_ticket(pre: &str) -> String {
        format!(
            "---\nid: {pre}-cdk4\ntitle: \"New\"\ncreated: 2026-01-01\nrepos: [x]\nchanges: []\nspecs: []\nblocked_by: []\nrelated: []\n---\n"
        )
    }

    /// A git project on branch `branch` holding two tickets, one old-form and one new-form.
    fn ticket_project(branch: &str) -> tempfile::TempDir {
        let p = git_project("", false);
        let open = p.path().join("docs/tickets/open");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::write(open.join("old-abk2.md"), OLD_TICKET).unwrap();
        let pre = bridle_daemon::config::default_task_prefix(&project_of(p.path()));
        std::fs::write(open.join("new-cdk4.md"), other_ticket(&pre)).unwrap();
        git(p.path(), &["checkout", "-q", "-B", branch]);
        git(p.path(), &["add", "."]);
        git(p.path(), &["commit", "-q", "-m", "tickets"]);
        p
    }

    #[test]
    fn ticket_migration_standardizes_old_tickets_and_skips_new_ones_and_rerun_is_a_noop() {
        let p = ticket_project("bridle-adopt");
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        let r = standard_ticket_fields(&ctx).unwrap();
        assert_eq!(r.files, vec!["docs/tickets/open/old-abk2.md".to_string()]);
        let pre = bridle_daemon::config::default_task_prefix(&project_of(p.path()));
        let got = std::fs::read_to_string(p.path().join("docs/tickets/open/old-abk2.md")).unwrap();
        assert_eq!(
            got,
            format!(
                "---\nid: {pre}-abk2\ntitle: \"Old\"\ncreated: 2026-01-01\nrepos: [x]\nchanges: []\nspecs: []\nblocked_by: [{pre}-cdk4, zzzz]\nrelated: []\nparent: {pre}-cdk4\n---\nbody\n"
            )
        );
        // Committed, the same tree has nothing left to do.
        git(p.path(), &["commit", "-qam", "migrated"]);
        assert!(standard_ticket_fields(&ctx).unwrap().files.is_empty());
    }

    #[test]
    fn ticket_migration_refuses_uncommitted_tickets_and_dry_run_writes_nothing() {
        let p = ticket_project("bridle-adopt");
        let path = p.path().join("docs/tickets/open/old-abk2.md");
        let dry = Ctx {
            repo: p.path(),
            dry_run: true,
        };
        assert_eq!(standard_ticket_fields(&dry).unwrap().files.len(), 1);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), OLD_TICKET);
        std::fs::write(&path, format!("{OLD_TICKET}more\n")).unwrap();
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        assert!(is_refusal(&standard_ticket_fields(&ctx).unwrap_err()));
    }

    #[test]
    fn bug_tickets_with_a_task_become_fix_and_rerun_is_a_noop() {
        let p = ticket_project("bridle-adopt");
        let open = p.path().join("docs/tickets/open");
        let ticket = |kind: &str, tasks: &str| {
            format!("---\nid: br-aaaa\ntitle: t\nkind: {kind}\ntasks: {tasks}\n---\nbody\n")
        };
        std::fs::write(open.join("worked-aaaa.md"), ticket("bug", "[br-aaaa]")).unwrap();
        std::fs::write(open.join("report-bbbb.md"), ticket("bug", "[]")).unwrap();
        std::fs::write(open.join("feat-cccc.md"), ticket("feature", "[br-cccc]")).unwrap();
        git(p.path(), &["add", "."]);
        git(p.path(), &["commit", "-q", "-m", "more"]);
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        let r = bug_tickets_that_are_fixes(&ctx).unwrap();
        assert_eq!(
            r.files,
            vec!["docs/tickets/open/worked-aaaa.md".to_string()]
        );
        let got = std::fs::read_to_string(open.join("worked-aaaa.md")).unwrap();
        assert!(got.contains("\nkind: fix\n"), "{got}");
        git(p.path(), &["commit", "-qam", "migrated"]);
        assert!(bug_tickets_that_are_fixes(&ctx).unwrap().files.is_empty());
    }

    #[test]
    fn edges_between_tickets_move_into_frontmatter_and_tasks_goes() {
        let p = ticket_project("bridle-adopt");
        let open = p.path().join("docs/tickets/open");
        let ticket = |id: &str| {
            format!(
                "---\nid: br-{id}\ntitle: t\nkind: feature\nblocked_by: []\nrelated: []\ntasks: [br-{id}]\n---\nbody\n"
            )
        };
        for id in ["aaaa", "bbbb", "cccc"] {
            std::fs::write(open.join(format!("t-{id}.md")), ticket(id)).unwrap();
        }
        git(p.path(), &["add", "."]);
        git(p.path(), &["commit", "-q", "-m", "more"]);
        let edge = |from: &str, to: &str, kind: &str| {
            format!(
                "[[edge]]\nfrom = \"{from}\"\nto = \"{to}\"\nkind = \"{kind}\"\ncreated_at = \"x\"\n\n"
            )
        };
        let edges = [
            edge("br-aaaa", "br-bbbb", "blocks"),
            edge("br-cccc", "br-bbbb", "parent"),
            edge("br-aaaa", "br-oldhex", "related"), // a row made from ticket cccc
            edge("br-aaaa", "br-zzzz", "blocks"),    // no such ticket: stays an edge
            edge("br-aaaa", "br-bbbb", "duplicates"),
        ]
        .concat();
        let rows = [("br-oldhex".to_string(), "cccc".to_string())].into();
        let ctx = Ctx {
            repo: p.path(),
            dry_run: true,
        };
        let r = carry_links(&ctx, &edges, &rows).unwrap();
        assert_eq!(r.files.len(), 3, "{:?}", r.files);
        assert!(
            !std::fs::read_to_string(open.join("t-bbbb.md"))
                .unwrap()
                .contains("parent")
        );
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        carry_links(&ctx, &edges, &rows).unwrap();
        let b = std::fs::read_to_string(open.join("t-bbbb.md")).unwrap();
        assert!(b.contains("blocked_by: [br-aaaa]"), "{b}");
        assert!(b.contains("parent: br-cccc"), "{b}");
        assert!(!b.contains("tasks:"), "{b}");
        let a = std::fs::read_to_string(open.join("t-aaaa.md")).unwrap();
        assert!(a.contains("related: [br-cccc]"), "{a}");
        git(p.path(), &["commit", "-qam", "migrated"]);
        assert!(carry_links(&ctx, &edges, &rows).unwrap().files.is_empty());
    }

    #[test]
    fn protected_branches_are_refused_for_existing_projects_but_not_bridle() {
        let p = ticket_project("main");
        let ctx = Ctx {
            repo: p.path(),
            dry_run: false,
        };
        assert!(is_refusal(&standard_ticket_fields(&ctx).unwrap_err()));
        assert!(refuse_on_protected_branch(p.path(), "other").is_err());
        // Bridle's own repo on main passes.
        assert!(refuse_on_protected_branch(p.path(), "bridle").is_ok());
        git(p.path(), &["checkout", "-q", "-b", "bridle-adopt"]);
        assert!(refuse_on_protected_branch(p.path(), "other").is_ok());
        // A configured release branch is protected too.
        std::fs::write(
            p.path().join(".bridle/config.toml"),
            "[branches]\nintegration = \"bridle-adopt\"\nrelease = \"bridle-adopt\"\n",
        )
        .unwrap();
        assert!(refuse_on_protected_branch(p.path(), "other").is_err());
    }

    #[test]
    fn a_declared_integration_branch_is_not_protected_and_nothing_to_change_never_refuses() {
        let p = ticket_project("main");
        let ctx = Ctx {
            repo: p.path(),
            dry_run: true,
        };
        // Nothing to change: no refusal on main, with or without a declared integration.
        std::fs::remove_dir_all(p.path().join("docs/tickets")).unwrap();
        for m in MIGRATIONS.iter().filter(|m| !m.manual_only) {
            if m.id.starts_with("0007") {
                continue; // adds a line when .gitignore lacks it
            }
            let r = (m.run)(&ctx);
            assert!(r.is_ok() || !is_refusal(&r.unwrap_err()), "{}", m.id);
        }
        // main declared as integration: the guard passes.
        std::fs::write(
            p.path().join(".bridle/config.toml"),
            "[branches]\nintegration = \"main\"\n",
        )
        .unwrap();
        assert!(refuse_on_protected_branch(p.path(), "other").is_ok());
        // ...unless it is also the release branch.
        std::fs::write(
            p.path().join(".bridle/config.toml"),
            "[branches]\nintegration = \"main\"\nrelease = \"main\"\n",
        )
        .unwrap();
        assert!(refuse_on_protected_branch(p.path(), "other").is_err());
    }

    #[test]
    fn shipped_ids_are_ordered_and_unique() {
        let ids: Vec<_> = MIGRATIONS.iter().map(|m| m.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn plan_tickets_names_ticketless_and_extra_active_tasks_only() {
        let d = tempfile::tempdir().unwrap();
        let open = d.path().join("docs/tickets/open");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::write(open.join("a-thing-aaaa.md"), "---\nid: br-aaaa\n---\n").unwrap();
        let row = |id: &str, state: &str, ticket: Option<&str>, at: &str| Row {
            id: id.into(),
            state: state.into(),
            ticket: ticket.map(str::to_string),
            created_at: at.into(),
            kind: None,
        };
        let rows = vec![
            // The ticket's own task keeps it, though an older-id extra exists.
            row("br-aaaa", "open", Some("br-aaaa"), "2026-10-02"),
            row("br-bbbb", "planned", Some("aaaa"), "2026-10-01"),
            // Active with no ticket, and one naming a ticket with no file.
            row("br-cccc", "pending", None, "2026-10-03"),
            row("br-dddd", "claimed", Some("zzzz"), "2026-10-04"),
            // Finished work stays history.
            row("br-eeee", "integrated", None, "2026-10-05"),
            row("br-ffff", "dropped", Some("aaaa"), "2026-10-06"),
        ];
        assert_eq!(
            plan_tickets(d.path(), &rows),
            vec![
                ("br-bbbb".to_string(), Some("br-aaaa".to_string())),
                ("br-cccc".to_string(), None),
                ("br-dddd".to_string(), None),
            ]
        );
    }

    #[test]
    fn backfill_kind_uses_the_row_else_chore_and_reruns_clean() {
        let d = tempfile::tempdir().unwrap();
        let open = d.path().join("docs/tickets/open");
        std::fs::create_dir_all(&open).unwrap();
        let t = |id: &str, extra: &str| format!("---\nid: br-{id}\ntitle: x\n{extra}---\nbody\n");
        std::fs::write(open.join("a-aaaa.md"), t("aaaa", "")).unwrap();
        std::fs::write(open.join("b-bbbb.md"), t("bbbb", "")).unwrap();
        std::fs::write(open.join("c-cccc.md"), t("cccc", "kind: feature\n")).unwrap();
        std::fs::write(open.join("d-dddd.md"), t("dddd", "kind:\n")).unwrap();
        let ctx = |dry_run| Ctx {
            repo: d.path(),
            dry_run,
        };
        // No state branch here (daemon down, nothing to read): everything falls back to chore.
        let r = backfill_ticket_kind(&ctx(true)).unwrap();
        assert_eq!(r.files.len(), 3, "{r:?}");
        assert!(
            !std::fs::read_to_string(open.join("a-aaaa.md"))
                .unwrap()
                .contains("kind")
        );
        backfill_ticket_kind(&ctx(false)).unwrap();
        let read = |n: &str| std::fs::read_to_string(open.join(n)).unwrap();
        assert!(read("a-aaaa.md").contains("kind: chore\n"));
        assert!(read("d-dddd.md").contains("kind: chore\n"));
        assert!(read("c-cccc.md").contains("kind: feature\n"));
        assert!(backfill_ticket_kind(&ctx(false)).unwrap().files.is_empty());
    }

    #[test]
    fn kind_for_prefers_own_row_then_naming_row() {
        let row = |id: &str, ticket: Option<&str>, kind: &str| Row {
            id: id.into(),
            state: "open".into(),
            ticket: ticket.map(str::to_string),
            created_at: String::new(),
            kind: Some(kind.into()),
        };
        let rows = vec![
            row("br-aaaa", Some("br-aaaa"), "fix"),
            row("br-bbbb", Some("aaaa"), "docs"),
            row("br-cccc", Some("dddd"), "feature"),
            row("br-eeee", None, "nonsense"),
        ];
        assert_eq!(kind_for(&rows, "br-aaaa"), "fix");
        assert_eq!(kind_for(&rows, "br-dddd"), "feature");
        assert_eq!(kind_for(&rows, "br-eeee"), "chore");
        assert_eq!(kind_for(&rows, "br-zzzz"), "chore");
    }

    #[test]
    fn gitignore_agent_site_appends_once() {
        let d = tempfile::tempdir().unwrap();
        let gi = d.path().join(".gitignore");
        std::fs::write(&gi, "/target").unwrap();
        let ctx = |dry_run| Ctx {
            repo: d.path(),
            dry_run,
        };
        let r = gitignore_agent_site(&ctx(true)).unwrap();
        assert_eq!(r.files, [".gitignore"]);
        assert_eq!(std::fs::read_to_string(&gi).unwrap(), "/target");
        gitignore_agent_site(&ctx(false)).unwrap();
        assert_eq!(
            std::fs::read_to_string(&gi).unwrap(),
            "/target\n.agent-site/\n"
        );
        assert!(gitignore_agent_site(&ctx(false)).unwrap().files.is_empty());
    }
}
