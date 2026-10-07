//! `bridle ticket new` / `resolve` (docs/design/cli.md, ticket 7gk7): mint a ticket file under
//! `docs/tickets/open/` and move it to `resolved/`. The file logic (id mint, frontmatter, find by
//! id) takes a tickets root and does no I/O beyond it, so `check`/`set`/`link` can reuse it.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, RandomState};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow, bail};
use bridle_api::{EditTaskRequest, NewTaskRequest, SubmitTaskRequest, TaskKind};
use clap::ValueEnum;

use crate::cli::{
    Cli, TaskKindArg, TicketAction, TicketArgs, TicketCheckArgs, TicketNewArgs, TicketResolveArgs,
    TicketSetArgs, TicketSubmitArgs, TicketTaskArgs,
};
use crate::commands::client_for;
use crate::error::CliError;
use crate::launchd::project_name;

const ID_ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
const ID_LEN: usize = 4;
const MAX_SLUG: usize = 60;

/// Whether a ticket with no `kind:` or a one-sided/missing ticket<->task link fails
/// `ticket check` (true) or only warns (false). Off until the backfill migration
/// (ticket v3dk, slice B) has run in every project; it flips this to true.
const MISSING_KIND_OR_LINK_IS_ERROR: bool = false;

/// First line of a task body made from a ticket before the `ticket` field (vk3y); still read
/// from a daemon that hasn't migrated it yet.
const TASK_ORIGIN_PREFIX: &str = "original id: ";

pub async fn run(cli: &Cli, args: &TicketArgs) -> Result<(), CliError> {
    // Submitting talks to a daemon only, so it works outside a git checkout of the project.
    if let TicketAction::Submit(a) = &args.action {
        return submit(cli, a).await;
    }
    let repo = repo_root()?;
    match &args.action {
        TicketAction::New(a) => new(cli, &repo, a).await,
        TicketAction::Task(a) => task_cmd(cli, &repo, a).await,
        TicketAction::Resolve(a) => resolve_cmd(&repo, a),
        TicketAction::Set(a) => set_cmd(&repo, a),
        TicketAction::Check(a) => check_cmd(cli, &repo, a).await,
        TicketAction::Submit(_) => unreachable!("handled above"),
    }
}

pub fn repo_root() -> anyhow::Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!("not in a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

pub fn tickets_root(repo: &Path) -> PathBuf {
    repo.join("docs/tickets")
}

async fn new(cli: &Cli, repo: &Path, args: &TicketNewArgs) -> Result<(), CliError> {
    let repos = if args.repos.is_empty() {
        vec![project_name(cli, repo)?]
    } else {
        args.repos.clone()
    };
    let from = match &args.from_task {
        Some(id) => Some(client_for(cli).await?.get_task(id).await?),
        None => None,
    };
    let title = args
        .title
        .clone()
        .or_else(|| from.as_ref().map(|t| t.title.clone()))
        .ok_or_else(|| anyhow!("a title is required"))?;
    let kind = match (args.kind, &from) {
        (Some(k), _) => kind_of(k),
        (None, Some(t)) => t.kind,
        (None, None) => return Err(anyhow!("a kind is required").into()),
    };
    let root = tickets_root(repo);
    let id = from.as_ref().and_then(|t| reusable_id(&root, &t.id));
    let ask = if args.body.is_some() || args.body_file.is_some() {
        crate::commands::read_text(&args.body, &args.body_file, "body")?
    } else {
        String::new()
    };
    // The caller's own principal, as tasks get `created_by`; left out when no daemon answers,
    // since `ticket new` works without one.
    let filed_by = match client_for(cli).await {
        Ok(c) => c.status().await.ok().map(|s| s.principal),
        Err(_) => None,
    };
    let fields = Fields {
        title: &title,
        id: id.as_deref(),
        ask: &ask,
        kind,
        repos: &repos,
        needs: &args.needs,
        see: &args.see,
        filed_by: filed_by.as_deref(),
    };
    let today = chrono::Utc::now().date_naive().to_string();
    let path = create(&root, &fields, &today, &task_id_tails(cli).await)?;
    let rel = path
        .strip_prefix(repo)
        .unwrap_or(&path)
        .display()
        .to_string();
    if let Some(task) = &from {
        // Link both ways: the ticket lists the task, and the task's `ticket` field names the
        // ticket (what `ticket check` reads); the ticket's id is the task's tail unless that
        // couldn't be reused.
        let ticket_id = id_of(&path).ok_or_else(|| anyhow!("new ticket has no id"))?;
        set(&root, &ticket_id, "tasks", &task.id)?;
        let body = if task.ticket.is_some() {
            None
        } else {
            Some(format!("{rel}\n\n{}", task.body))
        };
        client_for(cli)
            .await?
            .edit_task(
                &task.id,
                &EditTaskRequest {
                    body,
                    ticket: Some(ticket_id),
                    ..Default::default()
                },
            )
            .await?;
    }
    println!("{rel}");

    Ok(())
}

/// `bridle ticket task <id>`: file the task for a ticket that is committed and has an ask. The
/// task is filed only now so it can never reach an agent before its brief is in git (k7tm).
async fn task_cmd(cli: &Cli, repo: &Path, args: &TicketTaskArgs) -> Result<(), CliError> {
    let root = tickets_root(repo);
    let path = find_by_id(&root.join("open"), &args.id)?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let (title, kind) = task_source(&text)?;
    let rel = path.strip_prefix(repo).unwrap_or(&path).to_path_buf();
    ensure_committed(repo, &rel)?;
    let rel = rel.display().to_string();
    // The daemon gives the task the ticket's id from the `ticket` field (br-9e15).
    let task_id = make_task(cli, &title, kind, &args.id, &rel).await?;
    set(&root, &args.id, "tasks", &task_id)?;
    println!("{task_id}");
    Ok(())
}

/// The title and kind of a ticket's text; refuses an empty "The ask".
fn task_source(text: &str) -> anyhow::Result<(String, TaskKind)> {
    let (pairs, body) = split_front(text).ok_or_else(|| anyhow!("ticket has no frontmatter"))?;
    let get = |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    let title = get("title")
        .map(|t| t.trim_matches(|c| c == '"' || c == '\'').to_string())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| anyhow!("ticket has no title"))?;
    let kind = get("kind")
        .ok_or_else(|| anyhow!("ticket has no kind (bridle ticket set <id> kind <kind>)"))?
        .parse::<TaskKind>()
        .map_err(|_| anyhow!("ticket has an unknown kind"))?;
    let ask = body
        .split_once("## The ask")
        .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(""))
        .unwrap_or("");
    if ask.trim().is_empty() {
        bail!(
            "the ticket's \"The ask\" section is empty: write the ask, commit it, then file the task"
        );
    }
    Ok((title, kind))
}

/// Errors unless `rel` (relative to `repo`) is in the tip of the local `main`.
fn ensure_committed(repo: &Path, rel: &Path) -> anyhow::Result<()> {
    let spec = format!("main:{}", rel.display());
    let in_main = Command::new("git")
        .current_dir(repo)
        .args(["cat-file", "-e", &spec])
        .output()
        .context("running git")?
        .status
        .success();
    if !in_main {
        bail!(
            "{} is not committed on main: commit it there first, so the task never outruns its brief",
            rel.display()
        );
    }
    Ok(())
}

async fn submit(cli: &Cli, args: &TicketSubmitArgs) -> Result<(), CliError> {
    let body = if args.body.is_some() || args.body_file.is_some() {
        crate::commands::read_text(&args.body, &args.body_file, "body")?
    } else {
        String::new()
    };
    let client = client_for(cli).await?;
    let task = client
        .submit_task(&SubmitTaskRequest {
            title: args.title.clone(),
            kind: kind_of(args.kind),
            body,
        })
        .await?;
    println!("{}", task.id);
    Ok(())
}

fn kind_of(arg: TaskKindArg) -> TaskKind {
    let name = arg.to_possible_value().expect("no skipped kinds");
    name.get_name().parse().expect("CLI kinds are task kinds")
}

/// Creates the task for a ticket and returns its id.
async fn make_task(
    cli: &Cli,
    title: &str,
    kind: TaskKind,
    id: &str,
    rel: &str,
) -> Result<String, CliError> {
    let client = client_for(cli).await?;
    let task = client
        .new_task(&NewTaskRequest {
            for_human: false,
            priority: None,
            title: title.to_string(),
            kind,
            body: rel.to_string(),
            components: Vec::new(),
            size: None,
            ticket: Some(id.to_string()),
        })
        .await?;
    Ok(task.id)
}

/// The id tails of every task (`br-k7tm` -> `k7tm`). Empty when the daemon isn't reachable
/// (no tasks can be filed then either).
async fn task_id_tails(cli: &Cli) -> HashSet<String> {
    let Ok(client) = client_for(cli).await else {
        return HashSet::new();
    };
    let Ok(tasks) = client.list_tasks().await else {
        return HashSet::new();
    };
    tasks
        .into_iter()
        .filter_map(|t| Some(t.id.rsplit_once('-')?.1.to_string()))
        .collect()
}

/// Task id -> the ticket id its body names, for every task made from a ticket. `None` when
/// the daemon isn't reachable: the task side of the link can't be checked then.
async fn task_links(cli: &Cli) -> Option<HashMap<String, String>> {
    let client = client_for(cli).await.ok()?;
    let tasks = client.list_tasks().await.ok()?;
    Some(
        tasks
            .into_iter()
            .filter_map(|t| Some((t.id.clone(), ticket_of(&t)?)))
            .collect(),
    )
}

/// The ticket a task was made from: its `ticket` field, or (a daemon that hasn't migrated it
/// yet) the old `original id:` first body line.
fn ticket_of(t: &bridle_api::Task) -> Option<String> {
    t.ticket.clone().or_else(|| {
        let first = t.body.lines().next()?;
        Some(first.strip_prefix(TASK_ORIGIN_PREFIX)?.trim().to_string())
    })
}

fn resolve_cmd(repo: &Path, args: &TicketResolveArgs) -> Result<(), CliError> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let dest = resolve(&tickets_root(repo), &args.id, &now)?;
    println!("{}", dest.strip_prefix(repo).unwrap_or(&dest).display());
    Ok(())
}

fn set_cmd(repo: &Path, args: &TicketSetArgs) -> Result<(), CliError> {
    let path = set(&tickets_root(repo), &args.id, &args.field, &args.value)?;
    println!("{}", path.strip_prefix(repo).unwrap_or(&path).display());
    Ok(())
}

async fn check_cmd(cli: &Cli, repo: &Path, args: &TicketCheckArgs) -> Result<(), CliError> {
    let root = tickets_root(repo);
    let links = task_links(cli).await;
    let report = check(&root, &repo.join("docs"), links.as_ref());
    for w in &report.warnings {
        eprintln!("warning: {w}");
    }
    if report.errors.is_empty() {
        if !args.quiet && report.warnings.is_empty() {
            println!("tickets ok");
        }
        return Ok(());
    }
    for p in &report.errors {
        eprintln!("{p}");
    }
    Err(anyhow!("ticket check: {} problem(s)", report.errors.len()).into())
}

pub struct Fields<'a> {
    /// Use this id instead of minting one (see `reusable_id`).
    pub id: Option<&'a str>,
    pub title: &'a str,
    /// Text of "The ask" (may be empty).
    pub ask: &'a str,
    pub kind: TaskKind,
    pub repos: &'a [String],
    pub needs: &'a [String],
    pub see: &'a [String],
    /// The principal that filed it (`external:aide`); optional, older tickets have none.
    pub filed_by: Option<&'a str>,
}

/// Write a new ticket into `<root>/open/`, creating the folders; returns its path.
/// `task_ids` are the id tails of existing tasks (`br-k7tm` -> `k7tm`): tickets and tasks share
/// one id space, so a new ticket never takes one.
pub fn create(
    root: &Path,
    f: &Fields,
    today: &str,
    task_ids: &HashSet<String>,
) -> anyhow::Result<PathBuf> {
    let (open, resolved) = (root.join("open"), root.join("resolved"));
    for d in [&open, &resolved] {
        std::fs::create_dir_all(d).with_context(|| format!("creating {}", d.display()))?;
    }
    let id = match f.id {
        Some(id) => id.to_string(),
        None => {
            let mut taken = existing_ids(root);
            taken.extend(task_ids.iter().cloned());
            mint_id(&taken)
        }
    };
    let slug = slugify(f.title);
    let name = if slug.is_empty() {
        format!("ticket-{id}.md")
    } else {
        format!("{slug}-{id}.md")
    };
    let text = format!(
        "---\nid: {id}\ntitle: {}\nkind: {}\nopened: {today}\n{}repos: {}\nchanges: []\nspecs: []\nneeds: {}\nsee: {}\ntasks: []\n---\n\n## The ask\n\n{}",
        yaml_scalar(f.title),
        f.kind,
        f.filed_by
            .map(|p| format!("filed_by: {}\n", yaml_scalar(p)))
            .unwrap_or_default(),
        list(f.repos),
        list(f.needs),
        list(f.see),
        if f.ask.trim().is_empty() {
            String::new()
        } else {
            format!("{}\n", f.ask.trim_end())
        },
    );
    let path = open.join(name);
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Move the open ticket `id` to `resolved/`, stamping `closed:` first. Returns the new path.
pub fn resolve(root: &Path, id: &str, closed: &str) -> anyhow::Result<PathBuf> {
    let found = find_by_id(&root.join("open"), id)?;
    let dest = root
        .join("resolved")
        .join(found.file_name().expect("a file"));
    if find_by_id(&root.join("resolved"), id).is_ok() {
        bail!("ticket {id} is already in resolved/");
    }
    let text =
        std::fs::read_to_string(&found).with_context(|| format!("reading {}", found.display()))?;
    let stamped = set_closed(&text, closed)
        .ok_or_else(|| anyhow!("{} has no frontmatter", found.display()))?;
    std::fs::create_dir_all(root.join("resolved")).context("creating resolved/")?;
    std::fs::write(&dest, stamped).with_context(|| format!("writing {}", dest.display()))?;
    std::fs::remove_file(&found).with_context(|| format!("removing {}", found.display()))?;
    Ok(dest)
}

/// The one `*-<id>.md` file in `dir`; errors on none or several.
pub fn find_by_id(dir: &Path, id: &str) -> anyhow::Result<PathBuf> {
    let suffix = format!("-{id}.md");
    let mut hits: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(&suffix))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    hits.sort();
    match hits.len() {
        0 => bail!("no ticket with id {id} in {}", dir.display()),
        1 => Ok(hits.remove(0)),
        _ => bail!(
            "ticket id {id} is ambiguous in {}: {}",
            dir.display(),
            hits.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// The id a ticket file name ends with (`<slug>-<id>.md`).
pub fn id_of(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let id = stem.rsplit('-').next()?;
    (id.len() == ID_LEN && id.bytes().all(|b| ID_ALPHABET.contains(&b))).then(|| id.to_string())
}

const LIST_FIELDS: [&str; 6] = ["repos", "changes", "specs", "needs", "see", "tasks"];
const REQUIRED: [&str; 8] = [
    "id", "title", "opened", "repos", "changes", "specs", "needs", "see",
];

/// Set frontmatter `field` of ticket `id` (open or resolved) to `value`; returns the path.
pub fn set(root: &Path, id: &str, field: &str, value: &str) -> anyhow::Result<PathBuf> {
    if matches!(field, "id" | "opened" | "closed") {
        bail!("{field} is not editable (resolve stamps closed; id and opened never change)");
    }
    let is_list = LIST_FIELDS.contains(&field);
    if !is_list && field != "title" && field != "kind" {
        bail!(
            "unknown field {field}; one of title, kind, repos, changes, specs, needs, see, tasks"
        );
    }
    if field == "kind" {
        value.parse::<TaskKind>().map_err(|e| anyhow!(e))?;
    }
    let path = find_by_id(&root.join("open"), id)
        .or_else(|_| find_by_id(&root.join("resolved"), id))
        .map_err(|_| anyhow!("no ticket with id {id} in open/ or resolved/"))?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let rendered = if is_list {
        let items: Vec<String> = value
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        list(&items)
    } else {
        yaml_scalar(value)
    };
    let out = set_line(&text, field, &rendered)
        .ok_or_else(|| anyhow!("{} has no frontmatter", path.display()))?;
    std::fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// `text` with frontmatter `key: value` replaced, or added last before the closing `---`
/// (and before `closed:`, which stays last); `None` without a frontmatter block.
fn set_line(text: &str, key: &str, value: &str) -> Option<String> {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let end = front_end(&lines)?;
    let prefix = format!("{key}:");
    let new = format!("{key}: {value}");
    match (1..end).find(|&i| lines[i].starts_with(&prefix)) {
        Some(i) => lines[i] = new,
        None => {
            let at = if end > 1 && lines[end - 1].starts_with("closed:") {
                end - 1
            } else {
                end
            };
            lines.insert(at, new);
        }
    }
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// Index of the closing `---` of a leading frontmatter block.
fn front_end(lines: &[String]) -> Option<usize> {
    if lines.first().map(String::as_str) != Some("---") {
        return None;
    }
    lines.iter().skip(1).position(|l| l == "---").map(|p| p + 1)
}

/// The frontmatter as (key, raw value) pairs and the body after it.
fn split_front(text: &str) -> Option<(Vec<(String, String)>, String)> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let end = front_end(&lines)?;
    let pairs = lines[1..end]
        .iter()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    Some((pairs, lines[end + 1..].join("\n")))
}

fn list_items(raw: &str) -> Option<Vec<String>> {
    let inner = raw.strip_prefix('[')?.strip_suffix(']')?;
    Some(
        inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

/// `[[stem]]` / `[[stem|text]]` targets in `body`, outside fenced code blocks and inline code.
fn wiki_links(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut fenced = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        if fenced {
            continue;
        }
        // Odd-numbered pieces between backticks are inline code.
        let line: String = line.split('`').step_by(2).collect::<Vec<_>>().join(" ");
        let mut rest = line.as_str();
        while let Some(i) = rest.find("[[") {
            rest = &rest[i + 2..];
            let Some(j) = rest.find("]]") else { break };
            let target = rest[..j].split('|').next().unwrap_or("");
            let target = target.split('#').next().unwrap_or("").trim();
            if !target.is_empty() {
                out.push(target.to_string());
            }
            rest = &rest[j + 2..];
        }
    }
    out
}

fn md_stems(dir: &Path, out: &mut HashSet<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.filter_map(Result::ok) {
        let p = e.path();
        if p.is_dir() {
            md_stems(&p, out);
        } else if p.extension().is_some_and(|x| x == "md")
            && let Some(s) = p.file_stem()
        {
            out.insert(s.to_string_lossy().into_owned());
        }
    }
}

/// A link target is a file stem anywhere under `docs`, or a path (with or without `.md`)
/// from the repo root or from `docs`.
fn link_exists(link: &str, docs: &Path, stems: &HashSet<String>) -> bool {
    if stems.contains(link) {
        return true;
    }
    let with_md = if link.ends_with(".md") {
        link.to_string()
    } else {
        format!("{link}.md")
    };
    link.contains('/')
        && [docs.parent().unwrap_or(docs), docs]
            .iter()
            .any(|base| base.join(&with_md).is_file())
}

#[derive(Debug, Default)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Every problem in the tickets under `root/{open,resolved}`; `docs` is where `[[links]]` may
/// point (by file stem, anywhere beneath it). `task_links` maps task id -> the ticket id it was
/// made from; `None` skips the task side of the two-way link.
pub fn check(root: &Path, docs: &Path, task_links: Option<&HashMap<String, String>>) -> Report {
    let mut problems = Vec::new();
    let mut warnings = Vec::new();
    let mut files: Vec<(bool, PathBuf)> = Vec::new();
    for (resolved, dir) in [(false, "open"), (true, "resolved")] {
        let Ok(rd) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        let mut found: Vec<PathBuf> = rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "md"))
            .collect();
        found.sort();
        files.extend(found.into_iter().map(|p| (resolved, p)));
    }
    let shown = |p: &Path| {
        p.strip_prefix(root.parent().and_then(Path::parent).unwrap_or(root))
            .unwrap_or(p)
            .display()
            .to_string()
    };
    let stems: HashSet<String> = files
        .iter()
        .filter_map(|(_, p)| Some(p.file_stem()?.to_string_lossy().into_owned()))
        .collect();
    let ids: HashSet<String> = files.iter().filter_map(|(_, p)| id_of(p)).collect();
    let mut doc_stems = HashSet::new();
    md_stems(docs, &mut doc_stems);
    let mut seen: std::collections::HashMap<String, PathBuf> = Default::default();

    for (resolved, path) in &files {
        let at = shown(path);
        // `soft` gaps are ones the backfill migration will fill: warnings until
        // MISSING_KIND_OR_LINK_IS_ERROR.
        let mut note = |error: bool, msg: String| {
            let line = format!("{at}: {msg}");
            if error {
                problems.push(line);
            } else {
                warnings.push(line);
            }
        };
        let file_id = id_of(path);
        if file_id.is_none() {
            note(true, "file name doesn't end in -<4-character id>.md".into());
        }
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                note(true, format!("unreadable: {e}"));
                continue;
            }
        };
        let Some((front, body)) = split_front(&text) else {
            note(true, "no frontmatter".into());
            continue;
        };
        let get = |k: &str| {
            front
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.as_str())
        };
        for k in REQUIRED {
            if get(k).is_none_or(str::is_empty) && !(LIST_FIELDS.contains(&k) && get(k).is_some()) {
                note(true, format!("missing {k}"));
            }
        }
        match get("kind") {
            None | Some("") => note(MISSING_KIND_OR_LINK_IS_ERROR, "missing kind".into()),
            Some(k) if k.parse::<TaskKind>().is_err() => note(true, format!("unknown kind {k}")),
            Some(_) => {}
        }
        let tasks = get("tasks").and_then(list_items);
        if get("tasks").is_none() {
            note(
                MISSING_KIND_OR_LINK_IS_ERROR,
                "missing tasks (the tasks made from this ticket)".into(),
            );
        }
        if let (Some(links), Some(me)) = (task_links, get("id")) {
            for t in tasks.iter().flatten() {
                match links.get(t) {
                    None => note(
                        true,
                        format!(
                            "tasks names {t}, which is not a task made from a ticket \
                             (or its task has no `ticket` field)"
                        ),
                    ),
                    Some(o) if o != me => {
                        note(true, format!("tasks names {t}, which names ticket {o}"))
                    }
                    Some(_) => {}
                }
            }
            let mut mine: Vec<&String> = links
                .iter()
                .filter(|(_, o)| *o == me)
                .map(|(t, _)| t)
                .collect();
            mine.sort();
            for t in mine {
                if !tasks.iter().flatten().any(|x| x == t) {
                    let msg = format!("task {t} names this ticket, but tasks doesn't list it");
                    if tasks.is_some() {
                        note(true, msg)
                    } else {
                        note(MISSING_KIND_OR_LINK_IS_ERROR, msg)
                    }
                }
            }
        }
        match (resolved, get("closed")) {
            (true, None | Some("")) => {
                note(true, "missing closed (required under resolved/)".into())
            }
            (false, Some(_)) => note(true, "closed is set but the ticket is under open/".into()),
            _ => {}
        }
        if let Some(id) = get("id") {
            if file_id.as_deref().is_some_and(|f| f != id) {
                note(true, format!("id {id} doesn't match the file name"));
            }
            if let Some(other) = seen.insert(id.to_string(), path.clone()) {
                note(true, format!("id {id} is also used by {}", shown(&other)));
            }
        }
        for k in LIST_FIELDS {
            let Some(raw) = get(k) else { continue };
            let Some(items) = list_items(raw) else {
                note(true, format!("{k} is not a [a, b] list"));
                continue;
            };
            if k == "needs" || k == "see" {
                for it in items {
                    if !ids.contains(&it) && !stems.contains(&it) {
                        note(true, format!("{k} names {it}, which is not a ticket"));
                    }
                }
            }
        }
        for link in wiki_links(&body) {
            if !link_exists(&link, docs, &doc_stems) {
                note(true, format!("link [[{link}]] points at no file"));
            }
        }
    }
    Report {
        errors: problems,
        warnings,
    }
}

/// The ticket id a task gives its ticket: its id tail, when that is in the ticket alphabet (task
/// ids minted before br-9e15 are hex and may hold `0` or `1`) and no ticket has it yet.
fn reusable_id(root: &Path, task_id: &str) -> Option<String> {
    let tail = task_id.rsplit_once('-')?.1;
    let ok = tail.len() == ID_LEN && tail.bytes().all(|b| ID_ALPHABET.contains(&b));
    (ok && !existing_ids(root).contains(tail)).then(|| tail.to_string())
}

fn existing_ids(root: &Path) -> HashSet<String> {
    ["open", "resolved"]
        .iter()
        .filter_map(|d| std::fs::read_dir(root.join(d)).ok())
        .flat_map(|rd| rd.filter_map(Result::ok))
        .filter_map(|e| id_of(&e.path()))
        .collect()
}

fn mint_id(taken: &HashSet<String>) -> String {
    let state = RandomState::new();
    for n in 0u64.. {
        let mut h = state.hash_one(n);
        let id: String = (0..ID_LEN)
            .map(|_| {
                let c = ID_ALPHABET[(h % ID_ALPHABET.len() as u64) as usize] as char;
                h /= ID_ALPHABET.len() as u64;
                c
            })
            .collect();
        if !taken.contains(&id) {
            return id;
        }
    }
    unreachable!("the id space is finite but far larger than any repo's tickets")
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    if slug.len() > MAX_SLUG {
        slug.truncate(MAX_SLUG);
    }
    slug.trim_end_matches('-').to_string()
}

fn list(items: &[String]) -> String {
    format!("[{}]", items.join(", "))
}

/// A frontmatter scalar, double-quoted only when plain YAML would misread it.
fn yaml_scalar(s: &str) -> String {
    let risky = s.contains(": ")
        || s.contains(" #")
        || s.ends_with(':')
        || s.starts_with(|c: char| "-?:,[]{}#&*!|>'\"%@`".contains(c) || c.is_whitespace());
    if risky {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// `text` with `closed: <ts>` as the last frontmatter line (replacing any existing one);
/// `None` when there is no frontmatter block.
fn set_closed(text: &str, closed: &str) -> Option<String> {
    let mut lines: Vec<&str> = text.lines().collect();
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    lines.retain({
        let mut i = 0;
        move |l| {
            i += 1;
            !(i > 1 && i <= end && l.starts_with("closed:"))
        }
    });
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    let stamp = format!("closed: {closed}");
    lines.insert(end, &stamp);
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields<'a>(title: &'a str, repos: &'a [String]) -> Fields<'a> {
        Fields {
            id: None,
            title,
            ask: "",
            kind: TaskKind::Feature,
            repos,
            needs: &[],
            see: &[],
            filed_by: None,
        }
    }

    #[test]
    fn ticket_of_reads_the_field_then_the_old_body_line() {
        let mut t: bridle_api::Task = serde_json::from_value(serde_json::json!({
            "id": "br-abcd", "title": "T", "kind": "feature", "state": "open", "body": "x",
            "thread": [], "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }))
        .unwrap();
        assert_eq!(ticket_of(&t), None);
        t.body = "original id: old1\nrest".to_string();
        assert_eq!(ticket_of(&t).as_deref(), Some("old1"));
        t.ticket = Some("abcd".to_string());
        assert_eq!(ticket_of(&t).as_deref(), Some("abcd"));
    }

    #[test]
    fn new_writes_named_file_with_frontmatter_and_makes_folders() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["proj".to_string()];
        let p = create(
            dir.path(),
            &fields("Fix the: thing (now)", &repos),
            "2026-09-30",
            &HashSet::new(),
        )
        .unwrap();
        assert!(dir.path().join("resolved").is_dir());
        let id = id_of(&p).unwrap();
        assert_eq!(p.parent().unwrap(), dir.path().join("open"));
        assert_eq!(
            p.file_name().unwrap().to_string_lossy(),
            format!("fix-the-thing-now-{id}.md")
        );
        let text = std::fs::read_to_string(&p).unwrap();
        assert_eq!(
            text,
            format!(
                "---\nid: {id}\ntitle: \"Fix the: thing (now)\"\nkind: feature\nopened: 2026-09-30\nrepos: [proj]\nchanges: []\nspecs: []\nneeds: []\nsee: []\ntasks: []\n---\n\n## The ask\n\n"
            )
        );
    }

    #[test]
    fn new_records_filed_by_and_check_accepts_it_or_its_absence() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("docs/tickets");
        let repos = vec!["proj".to_string()];
        let mut f = fields("Who filed this", &repos);
        f.filed_by = Some("external:aide");
        let p = create(&root, &f, "2026-10-07", &HashSet::new()).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(
            text.contains("opened: 2026-10-07\nfiled_by: external:aide\nrepos: [proj]\n"),
            "{text}"
        );
        // An old ticket with no `filed_by` still passes alongside it.
        write(&root, "open", "old-thing-aaaa.md", "aaaa", "", "");
        assert_eq!(problems(dir.path()), "");
    }

    #[test]
    fn ids_are_unique_across_open_and_resolved() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let mut seen = HashSet::new();
        for i in 0..200 {
            let p = create(
                dir.path(),
                &fields(&format!("t{i}"), &repos),
                "2026-09-30",
                &HashSet::new(),
            )
            .unwrap();
            let id = id_of(&p).unwrap();
            assert!(seen.insert(id.clone()));
            if i % 2 == 0 {
                resolve(dir.path(), &id, "2026-09-30T00:00:00Z").unwrap();
            }
        }
        // A taken id is never minted again.
        let taken: HashSet<String> = ID_ALPHABET
            .iter()
            .map(|c| format!("{0}{0}{0}{0}", *c as char))
            .collect();
        let id = mint_id(&taken);
        assert!(!taken.contains(&id));
    }

    #[test]
    fn a_new_ticket_never_takes_a_task_id() {
        let repos = vec!["p".to_string()];
        // Every id but one is a task's.
        let mut tasks: HashSet<String> = ID_ALPHABET
            .iter()
            .map(|c| format!("{0}{0}{0}{0}", *c as char))
            .collect();
        tasks.remove("aaaa");
        for _ in 0..50 {
            let d = tempfile::tempdir().unwrap();
            let p = create(d.path(), &fields("t", &repos), "2026-09-30", &tasks).unwrap();
            assert!(!tasks.contains(&id_of(&p).unwrap()));
        }
    }

    #[test]
    fn a_task_id_is_reused_unless_hex_or_taken() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path();
        assert_eq!(reusable_id(root, "br-k7tm").as_deref(), Some("k7tm"));
        // Pre-br-9e15 hex ids with 0 or 1 can't be ticket ids.
        assert_eq!(reusable_id(root, "br-a1b2"), None);
        assert_eq!(reusable_id(root, "br-e70f"), None);
        let repos = vec!["p".to_string()];
        let f = Fields {
            id: Some("k7tm"),
            ..fields("t", &repos)
        };
        let p = create(root, &f, "2026-10-03", &HashSet::from(["k7tm".to_string()])).unwrap();
        assert_eq!(id_of(&p).as_deref(), Some("k7tm"));
        assert_eq!(reusable_id(root, "br-k7tm"), None);
    }

    #[test]
    fn resolve_moves_and_stamps_closed() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let p = create(
            dir.path(),
            &fields("Thing", &repos),
            "2026-09-30",
            &HashSet::new(),
        )
        .unwrap();
        let id = id_of(&p).unwrap();
        let dest = resolve(dir.path(), &id, "2026-10-01T12:00:00Z").unwrap();
        assert!(!p.exists());
        assert_eq!(dest.parent().unwrap(), dir.path().join("resolved"));
        let text = std::fs::read_to_string(&dest).unwrap();
        assert!(
            text.contains("tasks: []\nclosed: 2026-10-01T12:00:00Z\n---\n"),
            "{text}"
        );
        assert!(resolve(dir.path(), &id, "x").is_err());
    }

    #[test]
    fn resolve_refuses_unknown_and_ambiguous_ids() {
        let dir = tempfile::tempdir().unwrap();
        let open = dir.path().join("open");
        std::fs::create_dir_all(&open).unwrap();
        let e = resolve(dir.path(), "zzzz", "x").unwrap_err().to_string();
        assert!(e.contains("no ticket"), "{e}");
        for n in ["a-abcd.md", "b-abcd.md"] {
            std::fs::write(open.join(n), "---\nid: abcd\n---\n").unwrap();
        }
        let e = resolve(dir.path(), "abcd", "x").unwrap_err().to_string();
        assert!(e.contains("ambiguous"), "{e}");
    }

    #[test]
    fn set_closed_replaces_an_existing_stamp() {
        let t = set_closed("---\nid: a\nclosed: old\n---\nbody\n", "new").unwrap();
        assert_eq!(t, "---\nid: a\nclosed: new\n---\nbody\n");
        assert!(set_closed("no frontmatter", "x").is_none());
    }

    const FRONT: &str =
        "opened: 2026-09-30\nrepos: [p]\nchanges: []\nspecs: []\nneeds: []\nsee: []\n";

    fn write(root: &Path, dir: &str, name: &str, id: &str, extra: &str, body: &str) {
        let d = root.join(dir);
        std::fs::create_dir_all(&d).unwrap();
        let text = format!("---\nid: {id}\ntitle: T\n{FRONT}{extra}---\n{body}\n");
        std::fs::write(d.join(name), text).unwrap();
    }

    fn report(root: &Path, links: Option<&HashMap<String, String>>) -> Report {
        check(&root.join("docs/tickets"), &root.join("docs"), links)
    }

    fn problems(root: &Path) -> String {
        report(root, None).errors.join("\n")
    }

    #[test]
    fn check_passes_a_clean_tree() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        write(
            &t,
            "open",
            "a-thing-aaaa.md",
            "aaaa",
            "",
            "see [[b-thing-bbbb|b]]",
        );
        write(
            &t,
            "resolved",
            "b-thing-bbbb.md",
            "bbbb",
            "closed: 2026-10-01T00:00:00Z\n",
            "",
        );
        assert_eq!(problems(dir.path()), "");
    }

    #[test]
    fn check_reports_each_kind_of_problem() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        std::fs::create_dir_all(t.join("open")).unwrap();
        std::fs::write(t.join("open/no-front-cccc.md"), "hi\n").unwrap();
        std::fs::write(t.join("open/short.md"), "---\nid: zzzz\n---\n").unwrap();
        write(&t, "open", "mismatch-aaaa.md", "dddd", "", "");
        write(&t, "open", "dup-eeee.md", "eeee", "closed: x\n", "");
        write(&t, "resolved", "dup-eeee.md", "eeee", "", "");
        write(
            &t,
            "open",
            "links-ffff.md",
            "ffff",
            "",
            "[[nowhere|x]] and [[dup-eeee]] `[[inline-code]]`\n```\n[[in-code]]\n```",
        );
        let f = std::fs::read_to_string(t.join("open/links-ffff.md")).unwrap();
        std::fs::write(
            t.join("open/links-ffff.md"),
            f.replace("needs: []", "needs: [nope]")
                .replace("see: []", "see: [eeee, dup-eeee]"),
        )
        .unwrap();
        let p = problems(dir.path());
        for want in [
            "no-front-cccc.md: no frontmatter",
            "short.md: file name doesn't end in -<4-character id>.md",
            "short.md: missing title",
            "mismatch-aaaa.md: id dddd doesn't match the file name",
            "closed is set but the ticket is under open/",
            "missing closed (required under resolved/)",
            "id eeee is also used by",
            "needs names nope, which is not a ticket",
            "link [[nowhere]] points at no file",
        ] {
            assert!(p.contains(want), "missing {want:?} in:\n{p}");
        }
        assert!(!p.contains("in-code") && !p.contains("inline-code"), "{p}");
        assert!(!p.contains("see names"), "{p}");
    }

    #[test]
    fn check_warns_on_missing_kind_and_tasks_and_errors_on_unknown_kind() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        write(&t, "open", "a-aaaa.md", "aaaa", "", "");
        write(
            &t,
            "open",
            "b-bbbb.md",
            "bbbb",
            "kind: nonsense\ntasks: []\n",
            "",
        );
        write(
            &t,
            "open",
            "c-cccc.md",
            "cccc",
            "kind: bug\ntasks: []\n",
            "",
        );
        let r = report(dir.path(), None);
        assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
        assert!(r.errors[0].contains("b-bbbb.md: unknown kind nonsense"));
        let w = r.warnings.join("\n");
        assert!(w.contains("a-aaaa.md: missing kind"), "{w}");
        assert!(w.contains("a-aaaa.md: missing tasks"), "{w}");
        assert!(!w.contains("c-cccc"), "{w}");
    }

    #[test]
    fn check_flags_a_link_present_on_one_side_only() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        let ok = "kind: bug\n";
        write(
            &t,
            "open",
            "a-aaaa.md",
            "aaaa",
            &format!("{ok}tasks: [br-1111, br-2222]\n"),
            "",
        );
        write(
            &t,
            "open",
            "b-bbbb.md",
            "bbbb",
            &format!("{ok}tasks: []\n"),
            "",
        );
        write(&t, "open", "c-cccc.md", "cccc", ok, "");
        let links: HashMap<String, String> = [
            ("br-1111", "aaaa"), // both sides
            ("br-2222", "bbbb"), // aaaa lists a task naming another ticket
            ("br-3333", "bbbb"), // bbbb doesn't list it
            ("br-4444", "cccc"), // cccc has no tasks field yet: a warning
        ]
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .into();
        let r = report(dir.path(), Some(&links));
        let e = r.errors.join("\n");
        assert!(
            e.contains("a-aaaa.md: tasks names br-2222, which names ticket bbbb"),
            "{e}"
        );
        assert!(
            e.contains("b-bbbb.md: task br-3333 names this ticket"),
            "{e}"
        );
        assert_eq!(r.errors.len(), 3, "{e}"); // br-2222 is wrong on both sides
        assert!(
            r.warnings
                .iter()
                .any(|w| w.contains("c-cccc.md: task br-4444"))
        );
        let unknown = HashMap::new();
        let e = report(dir.path(), Some(&unknown)).errors.join("\n");
        assert!(
            e.contains("tasks names br-1111, which is not a task"),
            "{e}"
        );
    }

    #[test]
    fn set_kind_edits_and_validates() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let p = create(
            dir.path(),
            &fields("Thing", &repos),
            "2026-09-30",
            &HashSet::new(),
        )
        .unwrap();
        let id = id_of(&p).unwrap();
        set(dir.path(), &id, "kind", "question").unwrap();
        assert!(
            std::fs::read_to_string(&p)
                .unwrap()
                .contains("kind: question\n")
        );
        let e = set(dir.path(), &id, "kind", "nope")
            .unwrap_err()
            .to_string();
        assert!(e.contains("unknown task kind"), "{e}");
        set(dir.path(), &id, "tasks", "br-1111, br-2222").unwrap();
        assert!(
            std::fs::read_to_string(&p)
                .unwrap()
                .contains("tasks: [br-1111, br-2222]\n")
        );
    }

    #[test]
    fn set_edits_fields_and_refuses_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let p = create(
            dir.path(),
            &fields("Thing", &repos),
            "2026-09-30",
            &HashSet::new(),
        )
        .unwrap();
        let id = id_of(&p).unwrap();
        set(dir.path(), &id, "repos", "a, b").unwrap();
        set(dir.path(), &id, "title", "New: title").unwrap();
        set(dir.path(), &id, "needs", "xxxx").unwrap();
        set(dir.path(), &id, "see", "").unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains("title: \"New: title\"\n"), "{text}");
        assert!(text.contains("repos: [a, b]\n"), "{text}");
        assert!(
            text.contains("needs: [xxxx]\nsee: []\ntasks: []\n---\n"),
            "{text}"
        );
        assert!(text.ends_with("## The ask\n\n"));
        // A resolved ticket is editable too, and `closed:` stays last.
        let dest = resolve(dir.path(), &id, "2026-10-01T00:00:00Z").unwrap();
        set(dir.path(), &id, "specs", "s1").unwrap();
        let text = std::fs::read_to_string(dest).unwrap();
        assert!(text.contains("specs: [s1]\n"), "{text}");
        assert!(
            text.contains("closed: 2026-10-01T00:00:00Z\n---\n"),
            "{text}"
        );
        for f in ["id", "opened", "closed"] {
            let e = set(dir.path(), &id, f, "x").unwrap_err().to_string();
            assert!(e.contains("not editable"), "{e}");
        }
        let e = set(dir.path(), &id, "status", "x").unwrap_err().to_string();
        assert!(e.contains("unknown field"), "{e}");
        let e = set(dir.path(), "zzzz", "title", "x")
            .unwrap_err()
            .to_string();
        assert!(e.contains("no ticket"), "{e}");
    }

    #[test]
    fn new_with_a_body_fills_the_ask() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let mut f = fields("Thing", &repos);
        f.ask = "Do the thing.\n";
        let p = create(dir.path(), &f, "2026-10-03", &HashSet::new()).unwrap();
        let text = std::fs::read_to_string(p).unwrap();
        assert!(text.ends_with("## The ask\n\nDo the thing.\n"), "{text}");
        let (title, kind) = task_source(&text).unwrap();
        assert_eq!((title.as_str(), kind), ("Thing", TaskKind::Feature));
    }

    #[test]
    fn ticket_task_refuses_an_empty_ask() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let p = create(
            dir.path(),
            &fields("Thing", &repos),
            "2026-10-03",
            &HashSet::new(),
        )
        .unwrap();
        let err = task_source(&std::fs::read_to_string(p).unwrap()).unwrap_err();
        assert!(err.to_string().contains("empty"), "{err}");
    }

    #[test]
    fn ticket_task_refuses_an_uncommitted_ticket() {
        let dir = tempfile::tempdir().unwrap();
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .current_dir(dir.path())
                .args(["-c", "user.name=t", "-c", "user.email=t@t"])
                .args(args)
                .output()
                .unwrap();
            assert!(out.status.success(), "{args:?}");
        };
        git(&["init", "-q", "-b", "main"]);
        std::fs::write(dir.path().join("a.md"), "x").unwrap();
        git(&["add", "a.md"]);
        git(&["commit", "-q", "-m", "a"]);
        std::fs::write(dir.path().join("b.md"), "y").unwrap();
        assert!(ensure_committed(dir.path(), Path::new("a.md")).is_ok());
        let err = ensure_committed(dir.path(), Path::new("b.md")).unwrap_err();
        assert!(err.to_string().contains("not committed"), "{err}");
    }
}
