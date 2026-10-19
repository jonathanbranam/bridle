//! `bridle ticket new` / `resolve` (docs/design/cli.md, ticket 7gk7): mint a ticket file under
//! `docs/tickets/open/` and move it to `resolved/`. The file logic (id mint, frontmatter, find by
//! id) takes a tickets root and does no I/O beyond it, so `check`/`set`/`link` can reuse it.

use std::collections::HashSet;
use std::hash::{BuildHasher, RandomState};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, anyhow, bail};
use bridle_api::{EditTicketRequest, NewTicketRequest, SubmitTicketRequest, TicketKind};
use clap::ValueEnum;

use crate::cli::{
    Cli, LinkKind, TicketAction, TicketAnswerArgs, TicketArgs, TicketAskArgs, TicketCheckArgs,
    TicketCommentArgs, TicketKindArg, TicketLinkArgs, TicketNewArgs, TicketReadyArgs,
    TicketResolveArgs, TicketSetArgs, TicketShowArgs, TicketSubmitArgs,
};
use crate::commands::client_for;
use crate::error::CliError;
use crate::launchd::project_name;

const ID_ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
const ID_LEN: usize = 4;
const MAX_SLUG: usize = 60;

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
        TicketAction::Ready(a) => ready_cmd(cli, &repo, a).await,
        TicketAction::Resolve(a) => resolve_cmd(cli, &repo, a).await,
        TicketAction::Set(a) => set_cmd(cli, &repo, a).await,
        TicketAction::Link(a) => link_cmd(cli, &repo, a).await,
        TicketAction::Check(a) => check_cmd(cli, &repo, a).await,
        TicketAction::Comment(a) => comment_cmd(cli, &repo, a).await,
        TicketAction::Ask(a) => ask_cmd(cli, &repo, a).await,
        TicketAction::Answer(a) => answer_cmd(cli, &repo, a).await,
        TicketAction::Show(a) => show_cmd(cli, &repo, a).await,
        TicketAction::Next(a) => next_cmd(cli, &repo, a).await,
        TicketAction::StageDone(a) => crate::stage_cmds::stage_done(cli, &repo, a).await,
        TicketAction::Approve(a) => crate::stage_cmds::approve(cli, &repo, a).await,
        TicketAction::Changes(a) => crate::stage_cmds::changes(cli, &repo, a).await,
        TicketAction::Restage(a) => crate::stage_cmds::restage(cli, &repo, a).await,
        TicketAction::Submit(_) => unreachable!("handled above"),
        TicketAction::Row(_) => unreachable!("normalized to the task handlers"),
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

/// `--project X` from inside another project's clone would write the file there (br-mggk);
/// refuse. A clone outside any workspace (`cwd_project` unknown) is left to the caller.
fn check_project_matches(requested: Option<&str>, cwd_project: Option<&str>) -> anyhow::Result<()> {
    match (requested, cwd_project) {
        (Some(want), Some(here)) if want != here => bail!(
            "--project {want} but this clone belongs to project {here}; the ticket file would be \
             written here. Run `bridle ticket new` from {want}'s clone"
        ),
        _ => Ok(()),
    }
}

/// A row that already names a ticket with a file keeps it: re-linking to a new file orphans the
/// row once the duplicate is dropped (br-mggk, br-b3ef).
fn check_not_linked(root: &Path, task: &bridle_api::Ticket) -> anyhow::Result<()> {
    let Some(t) = &task.ticket else {
        return Ok(());
    };
    if ["open", "resolved"]
        .iter()
        .any(|d| find_by_id(&root.join(d), t).is_ok())
    {
        bail!(
            "{} already has ticket {t}; not making another (it would re-link the row)",
            task.id
        );
    }
    Ok(())
}

/// A refused mint (the daemon said no) leaves no file behind (br-mggk).
fn discard_unfiled(
    path: &Path,
    rel: &str,
    repos: &[String],
    cause: impl std::fmt::Display,
) -> anyhow::Error {
    let _ = std::fs::remove_file(path);
    anyhow!(
        "ticket {rel} (repos [{}]) not made, its row was not filed: {cause}",
        repos.join(", ")
    )
}

pub(crate) async fn new(cli: &Cli, repo: &Path, args: &TicketNewArgs) -> Result<(), CliError> {
    check_project_matches(
        cli.project.as_deref(),
        crate::project::from_cwd(repo).as_deref(),
    )?;
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
    if let Some(t) = &from {
        check_not_linked(&root, t)?;
    }
    let id = from.as_ref().and_then(|t| reusable_id(&root, &t.id));
    let ask = if args.body.is_some() || args.body_file.is_some() {
        crate::commands::read_text(&args.body, &args.body_file, "body")?
    } else {
        String::new()
    };
    // The caller's own principal, as tasks get `created_by`; left out when no daemon answers,
    // since `ticket new` works without one.
    let created_by = match client_for(cli).await {
        Ok(c) => c.status().await.ok().map(|s| s.principal),
        Err(_) => None,
    };
    let prefix = own_prefix(cli, repo).await?;
    let projects = Projects::from_registry(&prefix);
    let qualify = |v: &[String]| -> anyhow::Result<Vec<String>> {
        v.iter().map(|i| projects.qualify(&root, i)).collect()
    };
    let (blocked_by, related) = (qualify(&args.blocked_by)?, qualify(&args.related)?);
    let stages = if args.version == 2 {
        crate::stages::build(&args.stages, &args.review)?
    } else {
        if !args.stages.is_empty() || !args.review.is_empty() || args.claim {
            return Err(anyhow!("--stages, --review and --claim need --version 2").into());
        }
        Vec::new()
    };
    let fields = Fields {
        title: &title,
        id: id.as_deref(),
        ask: &ask,
        kind,
        repos: &repos,
        blocked_by: &blocked_by,
        related: &related,
        created_by: created_by.as_deref(),
        theme: args.theme.as_deref(),
        prefix: Some(&prefix),
        stages: &stages,
    };
    if let Some(t) = fields.theme {
        check_theme(t)?;
    }
    let today = chrono::Utc::now().date_naive().to_string();
    let path = create(&root, &fields, &today, &task_id_tails(cli).await)?;
    let rel = path
        .strip_prefix(repo)
        .unwrap_or(&path)
        .display()
        .to_string();
    if let Some(task) = &from {
        // The task's `ticket` field names the ticket; the ticket file doesn't list the task.
        let ticket_id = id_of(&path).ok_or_else(|| anyhow!("new ticket has no id"))?;
        let body = if task.ticket.is_some() {
            None
        } else {
            Some(format!("{rel}\n\n{}", task.body))
        };
        let linked = async {
            client_for(cli)
                .await?
                .edit_task(
                    &task.id,
                    &EditTicketRequest {
                        body,
                        ticket: Some(ticket_id),
                        ..Default::default()
                    },
                )
                .await?;
            Ok::<_, CliError>(())
        }
        .await;
        if let Err(e) = linked {
            return Err(discard_unfiled(&path, &rel, &repos, e).into());
        }
    }
    if from.is_none() {
        // Every ticket has a row, `pending` until `ticket ready` (22ab section 2). The daemon
        // gives it the ticket's id from the `ticket` field (br-9e15).
        let ticket_id = id_of(&path).ok_or_else(|| anyhow!("new ticket has no id"))?;
        let priority = args.priority.map(crate::commands::task_priority_arg);
        let task = make_task(
            cli,
            &title,
            kind,
            &ticket_id,
            &rel,
            args.for_human,
            priority,
        )
        .await
        .map_err(|e| discard_unfiled(&path, &rel, &repos, e))?;
        // The file's prefix and the row's come from different places; refuse if they differ.
        if task.id.rsplit_once('-').map(|(p, _)| p) != Some(prefix.as_str()) {
            return Err(anyhow!(
                "{rel} has prefix `{prefix}` but its row is {}; fix the file's id or the row",
                task.id
            )
            .into());
        }
        if args.for_human {
            crate::commands::notify_human_todo(cli, &task).await?;
        }
        if args.claim {
            // A version 2 row stays `pending` while it is prepared, and a row claim needs
            // `planned`, so the filer's claim is the frontmatter line stage-done clears (br-r965).
            let who = created_by
                .as_deref()
                .ok_or_else(|| anyhow!("--claim needs the daemon to say who you are"))?;
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let out = set_line(&text, "claimed_by", who)
                .ok_or_else(|| anyhow!("{rel} has no frontmatter"))?;
            std::fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
        }
    }
    println!("{rel}");
    if let Some(w) = no_ask_warning(&ask) {
        eprintln!("{w}");
    }

    Ok(())
}

/// Warning for a ticket filed with no ask: `ticket ready` refuses it until "## The ask" is written.
fn no_ask_warning(ask: &str) -> Option<String> {
    ask.trim().is_empty().then(|| {
        "warning: no ask written; this ticket cannot be readied until its ask is under \"## The ask\" (use --body or --body-file, or edit the file and commit)".to_string()
    })
}

/// `bridle ticket ready <id>`: mark the ticket's row ready (`pending` -> `open`). Who may ready what is rule
/// `ready-without-asking`; refuses an empty ask or a ticket not committed on the local
/// integration branch, so work never outruns its brief (k7tm).
async fn ready_cmd(cli: &Cli, repo: &Path, args: &TicketReadyArgs) -> Result<(), CliError> {
    let root = tickets_root(repo);
    let id = projects(cli, repo).await?.local(&args.id)?.to_string();
    let path = find_by_id(&root.join("open"), &id)?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    crate::stage_cmds::check_ready_allowed(&text)?;
    let (title, kind) = task_source(&text)?;
    let rel = path.strip_prefix(repo).unwrap_or(&path).to_path_buf();
    let config =
        bridle_daemon::config::Config::load(repo).context("reading .bridle/config.toml")?;
    ensure_committed(repo, &config.branches.integration, &rel)?;
    let client = client_for(cli).await?;
    let rows = client.list_tasks().await?;
    let named = |t: &&bridle_api::Ticket| ticket_of(t).is_some_and(|tk| split_id(&tk).1 == id);
    // A dropped duplicate must not shadow a live row: prefer pending, then any not dropped.
    let found = rows
        .iter()
        .filter(named)
        .find(|t| t.state == bridle_api::TicketState::Pending)
        .or_else(|| {
            rows.iter()
                .filter(named)
                .find(|t| t.state != bridle_api::TicketState::Dropped)
        })
        .or_else(|| rows.iter().find(named));
    let task = match found {
        Some(t) => t.clone(),
        None => {
            // An older ticket file made before every ticket had a row (22ab): file it now,
            // from the file's own title and kind, as `ticket new` does.
            let rel_s = rel.display().to_string();
            make_task(cli, &title, kind, &id, &rel_s, false, None).await?
        }
    };
    let task = client.ready_task(&task.id).await?;
    println!("{}", task.id);
    Ok(())
}

/// The rows that carry a ticket's thread, oldest first: the row with the ticket's own id (its
/// thread is `tickets/<id>.md` on the state branch) or, for a ticket made from an older task,
/// the rows whose `ticket` names it.
async fn thread_rows(
    cli: &Cli,
    repo: &Path,
    id: &str,
) -> Result<Vec<bridle_api::Ticket>, CliError> {
    let tail = projects(cli, repo).await?.local(id)?.to_string();
    let all = crate::commands::client_for_read(cli)
        .await?
        .list_tasks()
        .await?;
    let mut rows: Vec<_> = all
        .iter()
        .filter(|t| split_id(&t.id).1 == tail)
        .cloned()
        .collect();
    if rows.is_empty() {
        rows = all
            .into_iter()
            .filter(|t| ticket_of(t).is_some_and(|tk| split_id(&tk).1 == tail))
            .collect();
    }
    rows.sort_by_key(|t| t.created_at);
    if rows.is_empty() {
        return Err(anyhow!("ticket {id} has no row on this daemon").into());
    }
    Ok(rows)
}

/// The row a new comment, question or answer goes on: the first of [`thread_rows`].
pub(crate) async fn thread_row_id(cli: &Cli, repo: &Path, id: &str) -> Result<String, CliError> {
    Ok(thread_rows(cli, repo, id).await?.remove(0).id)
}

async fn comment_cmd(cli: &Cli, repo: &Path, args: &TicketCommentArgs) -> Result<(), CliError> {
    let task = thread_row_id(cli, repo, &args.id).await?;
    crate::commands::task_comment(
        cli,
        &crate::cli::RowCommentArgs {
            task,
            text: args.text.clone(),
            text_file: args.text_file.clone(),
            notify: args.notify.clone(),
            for_human: args.for_human,
        },
    )
    .await
}

async fn ask_cmd(cli: &Cli, repo: &Path, args: &TicketAskArgs) -> Result<(), CliError> {
    let task = thread_row_id(cli, repo, &args.id).await?;
    crate::commands::ask(
        cli,
        &crate::cli::AskArgs {
            task,
            text: args.text.clone(),
            to: args.to.clone(),
        },
    )
    .await
}

async fn answer_cmd(cli: &Cli, repo: &Path, args: &TicketAnswerArgs) -> Result<(), CliError> {
    let task = thread_row_id(cli, repo, &args.id).await?;
    crate::commands::answer(
        cli,
        &crate::cli::AnswerArgs {
            task,
            text: args.text.clone(),
        },
    )
    .await
}

async fn show_cmd(cli: &Cli, repo: &Path, args: &TicketShowArgs) -> Result<(), CliError> {
    let rows = thread_rows(cli, repo, &args.id).await?;
    if cli.json {
        return Ok(crate::render::print_json(&rows)?);
    }
    let tail = projects(cli, repo).await?.local(&args.id)?.to_string();
    let root = tickets_root(repo);
    if let Ok(path) =
        find_by_id(&root.join("open"), &tail).or_else(|_| find_by_id(&root.join("resolved"), &tail))
    {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        println!("{}", text.trim_end());
        print_stage_states(&text);
    }
    for row in &rows {
        println!();
        println!("Thread ({}, {}):", row.id, row.state);
        for e in &row.thread {
            println!(
                "  [{}] {} ({}): {}",
                e.kind,
                e.from,
                e.at.to_rfc3339(),
                e.body
            );
        }
    }
    Ok(())
}

/// The human's own entry, or one an agent relayed for them (br-ggbx).
fn is_humans(e: &bridle_api::types::ThreadEntry) -> bool {
    e.for_human || e.from.contains("human")
}

async fn next_cmd(
    cli: &Cli,
    repo: &Path,
    args: &crate::cli::TicketNextArgs,
) -> Result<(), CliError> {
    let tail = projects(cli, repo).await?.local(&args.id)?.to_string();
    let path = find_by_id(&tickets_root(repo).join("open"), &tail)?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let front: Vec<&str> = text.lines().skip(1).take_while(|l| *l != "---").collect();
    let stages = crate::stages::parse(&front)
        .map_err(|e| anyhow!("{}: {e}", args.id))?
        .ok_or_else(|| anyhow!("{} is a version 1 ticket: it has no stages", args.id))?;
    let states = crate::stages::states(&stages, &crate::stages::marks(&text));
    // Fetched only for a redo; the stage file's (or the proposal's ticket file's) mtime stands
    // for "last written".
    let rows = if args.comments {
        thread_rows(cli, repo, &args.id).await?
    } else {
        Vec::new()
    };
    let next = crate::stage_next::build(repo, &path, &args.id, &stages, &states, |out| {
        let since = std::fs::metadata(out)
            .and_then(|m| m.modified())
            .ok()
            .map(chrono::DateTime::<chrono::Utc>::from);
        Ok(rows
            .iter()
            .flat_map(|r| &r.thread)
            .filter(|e| is_humans(e) && since.is_none_or(|s| e.at > s))
            .map(|e| crate::stage_next::Comment {
                at: e.at,
                body: e.body.clone(),
            })
            .collect())
    })?;
    if cli.json {
        return Ok(crate::render::print_json(&next)?);
    }
    print!("{}", crate::stage_next::render(&next));
    Ok(())
}

/// Under `ticket show`: each stage of a version 2 ticket and its state.
fn print_stage_states(text: &str) {
    let front: Vec<&str> = text.lines().skip(1).take_while(|l| *l != "---").collect();
    let Ok(Some(stages)) = crate::stages::parse(&front) else {
        return;
    };
    println!();
    println!("Stages:");
    for (name, state) in crate::stages::states(&stages, &crate::stages::marks(text)) {
        println!("  {name}: {state}");
    }
}

/// The title and kind of a ticket's text; refuses an empty "The ask".
fn task_source(text: &str) -> anyhow::Result<(String, TicketKind)> {
    let (pairs, body) = split_front(text).ok_or_else(|| anyhow!("ticket has no frontmatter"))?;
    let get = |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    let title = get("title")
        .map(|t| t.trim_matches(|c| c == '"' || c == '\'').to_string())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| anyhow!("ticket has no title"))?;
    let kind = get("kind")
        .ok_or_else(|| anyhow!("ticket has no kind (bridle ticket set <id> kind <kind>)"))?
        .parse::<TicketKind>()
        .map_err(|_| anyhow!("ticket has an unknown kind"))?;
    // The first non-empty section counts: an older file may have a leading empty heading.
    let has_ask = body
        .split("## The ask")
        .skip(1)
        .any(|rest| !rest.split("\n## ").next().unwrap_or("").trim().is_empty());
    if !has_ask {
        bail!(
            "the ticket's \"The ask\" section is empty: write the ask, commit it, then mark it ready"
        );
    }
    Ok((title, kind))
}

/// Errors unless `rel` (relative to `repo`) is in the tip of the local `branch` (the project's
/// integration branch: `dev` on a dev+release project, where `main` is production).
fn ensure_committed(repo: &Path, branch: &str, rel: &Path) -> anyhow::Result<()> {
    let spec = format!("{branch}:{}", rel.display());
    let in_branch = Command::new("git")
        .current_dir(repo)
        .args(["cat-file", "-e", &spec])
        .output()
        .context("running git")?
        .status
        .success();
    if !in_branch {
        bail!(
            "{} is not committed on {branch}: commit it there first, so work never outruns its brief",
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
        .submit_task(&SubmitTicketRequest {
            title: args.title.clone(),
            kind: kind_of(args.kind),
            body,
        })
        .await?;
    println!("{}", task.id);
    Ok(())
}

fn kind_of(arg: TicketKindArg) -> TicketKind {
    let name = arg.to_possible_value().expect("no skipped kinds");
    name.get_name().parse().expect("CLI kinds are task kinds")
}

/// Creates the task for a ticket and returns its id.
async fn make_task(
    cli: &Cli,
    title: &str,
    kind: TicketKind,
    id: &str,
    rel: &str,
    for_human: bool,
    priority: Option<bridle_api::TicketPriority>,
) -> Result<bridle_api::Ticket, CliError> {
    let client = client_for(cli).await?;
    let task = client
        .new_task(&NewTicketRequest {
            for_human,
            priority,
            title: title.to_string(),
            kind,
            body: rel.to_string(),
            components: Vec::new(),
            size: None,
            ticket: Some(id.to_string()),
            parent: None,
        })
        .await?;
    Ok(task)
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

/// The ticket a task was made from: its `ticket` field, or (a daemon that hasn't migrated it
/// yet) the old `original id:` first body line.
fn ticket_of(t: &bridle_api::Ticket) -> Option<String> {
    t.ticket.clone().or_else(|| {
        let first = t.body.lines().next()?;
        Some(first.strip_prefix(TASK_ORIGIN_PREFIX)?.trim().to_string())
    })
}

/// This project's id prefix: the running daemon's, else the project's own `[tasks] prefix` from
/// its `.bridle/config.toml`. Never guessed from the project name: an older daemon reports no
/// prefix, and a guess wrote `br-` ids into another project (ydyk).
pub(crate) async fn own_prefix(cli: &Cli, repo: &Path) -> Result<String, CliError> {
    let client = client_for(cli).await;
    let daemon = match &client {
        Ok(c) => c.status().await.ok().and_then(|s| s.daemon.task_prefix),
        Err(_) => None,
    };
    let config = bridle_daemon::config::Config::load(repo)
        .map_err(|e| anyhow!("reading {}/.bridle/config.toml: {e}", repo.display()))?
        .task_prefix;
    if daemon.is_none() && config.is_none() {
        // Outside a workspace with no --project, this is the error that names --project (s-f742).
        client.map_err(|e| {
            anyhow!(
                "{e} (project {})",
                project_name(cli, repo).unwrap_or_default()
            )
        })?;
    }
    Ok(pick_prefix(daemon, config, repo)?)
}

/// The daemon's prefix and the config's must agree when both are known; at least one is needed.
fn pick_prefix(
    daemon: Option<String>,
    config: Option<String>,
    repo: &Path,
) -> anyhow::Result<String> {
    match (daemon, config) {
        (Some(d), Some(c)) if d != c => Err(anyhow!(
            "the daemon's task prefix `{d}` differs from `[tasks] prefix = \"{c}\"` in {}/.bridle/config.toml; restart the daemon so they agree",
            repo.display()
        )),
        (Some(p), _) | (None, Some(p)) => Ok(p),
        (None, None) => Err(anyhow!(
            "no task prefix: the daemon does not report one and {}/.bridle/config.toml has no `[tasks] prefix`; start a current daemon or set the prefix",
            repo.display()
        )),
    }
}

pub(crate) async fn projects(cli: &Cli, repo: &Path) -> Result<Projects, CliError> {
    Ok(Projects::from_registry(&own_prefix(cli, repo).await?))
}

async fn resolve_cmd(cli: &Cli, repo: &Path, args: &TicketResolveArgs) -> Result<(), CliError> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = projects(cli, repo).await?.local(&args.id)?.to_string();
    let dest = resolve(&tickets_root(repo), &id, &now)?;
    println!("{}", dest.strip_prefix(repo).unwrap_or(&dest).display());
    Ok(())
}

async fn set_cmd(cli: &Cli, repo: &Path, args: &TicketSetArgs) -> Result<(), CliError> {
    let projects = projects(cli, repo).await?;
    let id = projects.local(&args.id)?;
    let path = set_in(&tickets_root(repo), id, &args.field, &args.value, &projects)?;
    println!("{}", path.strip_prefix(repo).unwrap_or(&path).display());
    Ok(())
}

async fn link_cmd(cli: &Cli, repo: &Path, args: &TicketLinkArgs) -> Result<(), CliError> {
    let projects = projects(cli, repo).await?;
    let id = projects.local(&args.id)?;
    let other = projects.qualify(&tickets_root(repo), &args.other)?;
    if matches!(args.kind, LinkKind::Duplicates | LinkKind::Supersedes) {
        // Rare, and not a field (22ab section 7): an edge between the two rows.
        let own = projects.own.as_deref().unwrap_or_default();
        let from = if own.is_empty() {
            id.to_string()
        } else {
            format!("{own}-{id}")
        };
        let (kind, client) = match args.kind {
            LinkKind::Duplicates => (bridle_api::EdgeKind::Duplicates, client_for(cli).await?),
            _ => (bridle_api::EdgeKind::Supersedes, client_for(cli).await?),
        };
        if args.rm {
            client
                .remove_edge(&bridle_api::RemoveEdgeQuery {
                    from,
                    to: other,
                    kind,
                })
                .await?;
        } else {
            let edge = client
                .add_edge(&bridle_api::NewEdgeRequest {
                    from,
                    to: other,
                    kind,
                })
                .await?;
            println!("{} {} {}", edge.from, edge.kind, edge.to);
        }
        return Ok(());
    }
    let path = link_in(
        &tickets_root(repo),
        id,
        args.kind,
        &other,
        args.rm,
        &projects,
    )?;
    println!("{}", path.strip_prefix(repo).unwrap_or(&path).display());
    Ok(())
}

/// Add (or, with `rm`, remove) `other` in ticket `id`'s `blocked_by`, `related` or `parent`.
/// Edits the file only; the daemon indexes it.
pub fn link_in(
    root: &Path,
    id: &str,
    kind: LinkKind,
    other: &str,
    rm: bool,
    projects: &Projects,
) -> anyhow::Result<PathBuf> {
    let path = find_by_id(&root.join("open"), id)
        .or_else(|_| find_by_id(&root.join("resolved"), id))
        .map_err(|_| anyhow!("no ticket with id {id} in open/ or resolved/"))?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    if let Some(out) = link_text_with(&text, kind, other, rm, projects, root)
        .map_err(|e| anyhow!("{} ({})", e, path.display()))?
    {
        std::fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(path)
}

/// [`link_in`]'s edit adding a link, on the text alone; `None` when nothing changes.
pub fn link_text(
    text: &str,
    kind: LinkKind,
    other: &str,
    projects: &Projects,
    root: &Path,
) -> anyhow::Result<Option<String>> {
    link_text_with(text, kind, other, false, projects, root)
}

fn link_text_with(
    text: &str,
    kind: LinkKind,
    other: &str,
    rm: bool,
    projects: &Projects,
    root: &Path,
) -> anyhow::Result<Option<String>> {
    let (field, is_list) = match kind {
        LinkKind::BlockedBy => ("blocked_by", true),
        LinkKind::Related => ("related", true),
        LinkKind::Parent => ("parent", false),
        _ => bail!("{kind:?} is an edge, not a frontmatter field"),
    };
    let (pairs, _) = split_front(text).ok_or_else(|| anyhow!("no frontmatter"))?;
    let raw = |name: &str| {
        pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.trim_matches('"').to_string())
    };
    let value = if is_list {
        let old = old_name(field).and_then(raw);
        let mut items = raw(field)
            .or(old)
            .and_then(|r| list_items(&r))
            .unwrap_or_default();
        if rm {
            items.retain(|i| i != other);
        } else if !items.iter().any(|i| i == other) {
            items.push(other.to_string());
        }
        items.join(",")
    } else if rm {
        String::new()
    } else {
        other.to_string()
    };
    let out = if field == "parent" && value.is_empty() {
        drop_line(text, "parent")
    } else {
        set_text(text, root, field, &value, projects)?
    };
    Ok((out != text).then_some(out))
}

async fn check_cmd(cli: &Cli, repo: &Path, args: &TicketCheckArgs) -> Result<(), CliError> {
    let root = tickets_root(repo);
    let projects = projects(cli, repo).await?;
    let report = check_in(&root, &repo.join("docs"), &projects);
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
    pub kind: TicketKind,
    pub repos: &'a [String],
    pub blocked_by: &'a [String],
    pub related: &'a [String],
    /// The principal that created it (`external:aide`); optional, older tickets have none.
    pub created_by: Option<&'a str>,
    /// A lasting area, as a slug (`reliability`); optional.
    pub theme: Option<&'a str>,
    /// The project's id prefix: the new ticket's `id:` is then written `<prefix>-<id>` (syqn).
    /// File names stay bare.
    pub prefix: Option<&'a str>,
    /// The stages of a version 2 ticket (`stages::build`); empty writes a version 1 ticket.
    pub stages: &'a [crate::stages::Stage],
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
    let qualified = f.prefix.map_or(id.clone(), |p| format!("{p}-{id}"));
    let text = format!(
        "---\nid: {qualified}\ntitle: {}\nkind: {}\ncreated: {today}\n{}{}{}repos: {}\nchanges: []\nspecs: []\nblocked_by: {}\nrelated: {}\n{}---\n\n{}{}{}",
        yaml_scalar(f.title),
        f.kind,
        f.created_by
            .map(|p| format!("created_by: {}\n", yaml_scalar(p)))
            .unwrap_or_default(),
        f.theme.map(|t| format!("theme: {t}\n")).unwrap_or_default(),
        if f.stages.is_empty() {
            ""
        } else {
            "version: 2\n"
        },
        list(f.repos),
        list(f.blocked_by),
        list(f.related),
        crate::stages::render(f.stages),
        // A body with its own heading is not put under a second, empty one.
        if f.ask.lines().any(|l| l.trim_end() == "## The ask") {
            ""
        } else {
            "## The ask\n\n"
        },
        if f.ask.trim().is_empty() {
            String::new()
        } else {
            format!("{}\n", f.ask.trim_end())
        },
        if f.stages.is_empty() {
            ""
        } else {
            crate::stages::PROPOSAL_TEMPLATE
        },
    );
    let path = open.join(name);
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Move the open ticket `id` to `resolved/`, stamping `resolved:` first. Returns the new path.
pub fn resolve(root: &Path, id: &str, resolved: &str) -> anyhow::Result<PathBuf> {
    let found = find_by_id(&root.join("open"), id)?;
    let dest = root
        .join("resolved")
        .join(found.file_name().expect("a file"));
    if find_by_id(&root.join("resolved"), id).is_ok() {
        bail!("ticket {id} is already in resolved/");
    }
    let text =
        std::fs::read_to_string(&found).with_context(|| format!("reading {}", found.display()))?;
    let stamped = set_resolved(&text, resolved)
        .ok_or_else(|| anyhow!("{} has no frontmatter", found.display()))?;
    std::fs::create_dir_all(root.join("resolved")).context("creating resolved/")?;
    std::fs::write(&dest, stamped).with_context(|| format!("writing {}", dest.display()))?;
    std::fs::remove_file(&found).with_context(|| format!("removing {}", found.display()))?;
    // The stage folder (design.md, attachments) travels with the file.
    let folder = crate::stages::folder_of(&found);
    if folder.is_dir() {
        let to = crate::stages::folder_of(&dest);
        std::fs::rename(&folder, &to)
            .with_context(|| format!("moving {} to {}", folder.display(), to.display()))?;
    }
    Ok(dest)
}

/// The one `*-<id>.md` file in `dir`; errors on none or several.
pub fn find_by_id(dir: &Path, id: &str) -> anyhow::Result<PathBuf> {
    // File names are bare, so a qualified `br-k7tm` finds `*-k7tm.md`.
    let id = split_id(id).1;
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

/// Split `br-k7tm` into (`Some("br")`, `k7tm`); a bare `k7tm` (or anything not of that shape,
/// such as a file stem) is (`None`, itself).
pub fn split_id(id: &str) -> (Option<&str>, &str) {
    match id.split_once('-') {
        Some((p, t))
            if !p.is_empty()
                && p.bytes().all(|b| b.is_ascii_alphanumeric())
                && t.len() == ID_LEN
                && t.bytes().all(|b| b.is_ascii_alphanumeric()) =>
        {
            (Some(p), t)
        }
        _ => (None, id),
    }
}

pub(crate) fn is_bare_id(s: &str) -> bool {
    s.len() == ID_LEN && s.bytes().all(|b| ID_ALPHABET.contains(&b))
}

/// The projects an id can name: this one (`own`, its tickets under the `root` passed to each
/// call) and the other registered ones (prefix, tickets root), for qualifying a bare id.
#[derive(Debug, Default, Clone)]
pub struct Projects {
    pub own: Option<String>,
    pub others: Vec<(String, PathBuf)>,
}

impl Projects {
    /// Other registered projects (from the machine registry), minus this one.
    pub fn from_registry(own: &str) -> Projects {
        let others = bridle_api::discovery::list_registry()
            .into_iter()
            .filter_map(|d| {
                let prefix = d.task_prefix?;
                (prefix != own).then(|| (prefix, tickets_root(Path::new(&d.repo))))
            })
            .collect();
        Projects {
            own: Some(own.to_string()),
            others,
        }
    }

    /// The bare id of a ticket in this project; refuses one qualified with another project's
    /// prefix.
    pub fn local<'a>(&self, id: &'a str) -> anyhow::Result<&'a str> {
        let (p, tail) = split_id(id);
        match (p, &self.own) {
            (Some(p), Some(own)) if p != own => {
                bail!("{id} belongs to project {p}, not {own}; run this in {p}'s checkout")
            }
            _ => Ok(tail),
        }
    }

    /// `item` as a reference written to a ticket: a bare ticket id becomes `<prefix>-<id>`,
    /// found in this project first, else in exactly one other registered project; an id found
    /// in several (or none) is refused with the candidates. Qualified ids and non-ids (file
    /// stems) pass through unchanged. With no `own` prefix known nothing is rewritten.
    fn qualify(&self, root: &Path, item: &str) -> anyhow::Result<String> {
        let Some(own) = &self.own else {
            return Ok(item.to_string());
        };
        if !is_bare_id(item) {
            return Ok(item.to_string());
        }
        if has_ticket(root, item) {
            return Ok(format!("{own}-{item}"));
        }
        let hits: Vec<String> = self
            .others
            .iter()
            .filter(|(_, r)| has_ticket(r, item))
            .map(|(p, _)| format!("{p}-{item}"))
            .collect();
        match hits.len() {
            1 => Ok(hits.into_iter().next().expect("one hit")),
            0 => bail!("no ticket {item} in {own} or any registered project"),
            _ => bail!("{item} is ambiguous: {}", hits.join(", ")),
        }
    }
}

fn has_ticket(root: &Path, tail: &str) -> bool {
    ["open", "resolved"]
        .iter()
        .any(|d| find_by_id(&root.join(d), tail).is_ok())
}

const LIST_FIELDS: [&str; 5] = ["repos", "changes", "specs", "blocked_by", "related"];
const REQUIRED: [&str; 8] = [
    "id",
    "title",
    "created",
    "repos",
    "changes",
    "specs",
    "blocked_by",
    "related",
];
/// Old frontmatter names and the standard names that replace them (syqn). Readers accept both
/// until the rename migration has run everywhere; writers use the new ones only.
const RENAMED: [(&str, &str); 5] = [
    ("needs", "blocked_by"),
    ("see", "related"),
    ("opened", "created"),
    ("filed_by", "created_by"),
    ("closed", "resolved"),
];

/// The standard name for `field`, mapping an old name to its new one.
fn canon(field: &str) -> &str {
    RENAMED
        .iter()
        .find(|(old, _)| *old == field)
        .map_or(field, |(_, new)| new)
}

/// The old name `field` replaced, if any.
fn old_name(field: &str) -> Option<&'static str> {
    RENAMED
        .iter()
        .find(|(_, new)| *new == field)
        .map(|(o, _)| *o)
}

/// A theme slug: lowercase letters, digits and hyphens, not starting or ending with a hyphen.
fn check_theme(t: &str) -> anyhow::Result<()> {
    let ok = !t.is_empty()
        && !t.starts_with('-')
        && !t.ends_with('-')
        && t.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !ok {
        bail!("theme {t:?} is not a slug (lowercase letters, digits and hyphens)");
    }
    Ok(())
}

/// A ticket id as `parent` may hold it: letters and digits, with an optional `<prefix>-`.
/// A parent in another project is only checked for form; `set` and `check` look it up through
/// the registry when its project is registered here.
fn check_parent(v: &str) -> anyhow::Result<()> {
    let word = |w: &str| !w.is_empty() && w.bytes().all(|b| b.is_ascii_alphanumeric());
    let ok = match v.split_once('-') {
        Some((prefix, id)) => word(prefix) && word(id),
        None => word(v),
    };
    if !ok {
        bail!("parent {v:?} is not a ticket id (letters and digits, optionally <prefix>-<id>)");
    }
    Ok(())
}

/// Set frontmatter `field` of ticket `id` (open or resolved) to `value`; returns the path.
#[cfg(test)]
pub fn set(root: &Path, id: &str, field: &str, value: &str) -> anyhow::Result<PathBuf> {
    set_in(root, id, field, value, &Projects::default())
}

/// [`set`], writing `blocked_by`, `related` and `parent` references in the qualified form.
pub fn set_in(
    root: &Path,
    id: &str,
    field: &str,
    value: &str,
    projects: &Projects,
) -> anyhow::Result<PathBuf> {
    let field = canon(field);
    if matches!(field, "id" | "created" | "resolved") {
        bail!("{field} is not editable (resolve stamps resolved; id and created never change)");
    }
    let is_list = LIST_FIELDS.contains(&field);
    if !is_list && !matches!(field, "title" | "kind" | "theme" | "parent") {
        bail!(
            "unknown field {field}; one of title, kind, theme, parent, repos, changes, specs, blocked_by, related"
        );
    }
    if field == "kind" {
        value.parse::<TicketKind>().map_err(|e| anyhow!(e))?;
    }
    if field == "theme" {
        check_theme(value)?;
    }
    if field == "parent" {
        check_parent(value)?;
    }
    let path = find_by_id(&root.join("open"), id)
        .or_else(|_| find_by_id(&root.join("resolved"), id))
        .map_err(|_| anyhow!("no ticket with id {id} in open/ or resolved/"))?;
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let out = set_text(&text, root, field, value, projects)
        .map_err(|e| anyhow!("{} ({})", e, path.display()))?;
    std::fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// `text` with frontmatter `field` set to `value` (the transform behind [`set_in`]).
fn set_text(
    text: &str,
    root: &Path,
    field: &str,
    value: &str,
    projects: &Projects,
) -> anyhow::Result<String> {
    let is_list = LIST_FIELDS.contains(&field);
    let refs = matches!(field, "blocked_by" | "related");
    let rendered = if is_list {
        let mut items: Vec<String> = value
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        if refs {
            items = items
                .iter()
                .map(|i| projects.qualify(root, i))
                .collect::<anyhow::Result<_>>()?;
        }
        list(&items)
    } else if field == "parent" {
        yaml_scalar(&projects.qualify(root, value)?)
    } else {
        yaml_scalar(value)
    };
    // A ticket still in the old form keeps one copy of the field: drop the old-named line so the
    // new one is the only one (the new name wins on read anyway).
    let text = match old_name(field) {
        Some(old) => drop_line(text, old),
        None => text.to_string(),
    };
    let out = set_line(&text, field, &rendered).ok_or_else(|| anyhow!("no frontmatter"))?;
    Ok(out)
}

/// `text` with frontmatter `key: value` replaced, or added last before the closing `---`
/// (and before `resolved:`/`closed:`, which stay last); `None` without a frontmatter block.
pub(crate) fn set_line(text: &str, key: &str, value: &str) -> Option<String> {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let end = front_end(&lines)?;
    let prefix = format!("{key}:");
    let new = format!("{key}: {value}");
    match (1..end).find(|&i| lines[i].starts_with(&prefix)) {
        Some(i) => lines[i] = new,
        None => {
            let at = if end > 1
                && (lines[end - 1].starts_with("resolved:")
                    || lines[end - 1].starts_with("closed:"))
            {
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

/// `text` without the frontmatter line for `key`.
pub(crate) fn drop_line(text: &str, key: &str) -> String {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some(end) = front_end(&lines) else {
        return text.to_string();
    };
    let prefix = format!("{key}:");
    let mut out: Vec<&str> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if !(i > 0 && i < end && l.starts_with(&prefix)) {
            out.push(l);
        }
    }
    let mut s = out.join("\n");
    if text.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Index of the closing `---` of a leading frontmatter block.
pub(crate) fn front_end(lines: &[String]) -> Option<usize> {
    if lines.first().map(String::as_str) != Some("---") {
        return None;
    }
    lines.iter().skip(1).position(|l| l == "---").map(|p| p + 1)
}

/// A bare id, including old ones with a `0` or `1` that new ids no longer use.
fn loose_id(s: &str) -> bool {
    s.len() == ID_LEN
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// `text` with its frontmatter in the standard form (syqn): old field names renamed (the new
/// name wins when both are there), a bare `id:` and bare ticket ids in `blocked_by`, `related`
/// and `parent` qualified with `prefix`. A bare reference is qualified only when `root` holds a
/// ticket with that id (a ticket of another project is left for its owner to qualify). `None`
/// when there is no frontmatter or nothing changes, so a rerun is a no-op.
pub fn standardize_front(text: &str, prefix: &str, root: &Path) -> Option<String> {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let end = front_end(&lines)?;
    let keys: Vec<String> = lines[1..end]
        .iter()
        .filter_map(|l| l.split_once(':').map(|(k, _)| k.trim().to_string()))
        .collect();
    let qualify = |item: &str| {
        if loose_id(item) && has_ticket(root, item) {
            format!("{prefix}-{item}")
        } else {
            item.to_string()
        }
    };
    let mut changed = false;
    let mut out: Vec<String> = vec![lines[0].clone()];
    for line in lines.drain(1..end).collect::<Vec<_>>() {
        let Some((key, value)) = line.split_once(':') else {
            out.push(line);
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        let new_key = canon(key);
        if new_key != key {
            changed = true;
            if keys.iter().any(|k| k == new_key) {
                continue;
            }
        }
        let new_value = match new_key {
            "id" if loose_id(value.trim_matches('"')) => {
                format!("{prefix}-{}", value.trim_matches('"'))
            }
            "parent" => qualify(value.trim_matches('"')),
            "blocked_by" | "related" => match list_items(value) {
                Some(items) => {
                    let q: Vec<String> = items.iter().map(|i| qualify(i)).collect();
                    if q == items {
                        value.to_string()
                    } else {
                        format!("[{}]", q.join(", "))
                    }
                }
                None => value.to_string(),
            },
            _ => value.to_string(),
        };
        if new_key == key && new_value == value {
            out.push(line);
        } else {
            changed = true;
            out.push(format!("{new_key}: {new_value}"));
        }
    }
    if !changed {
        return None;
    }
    out.extend(lines.drain(1..));
    let mut s = out.join("\n");
    if text.ends_with('\n') {
        s.push('\n');
    }
    Some(s)
}

/// `text` with `kind: bug` changed to `kind: fix` when the ticket lists a task (bpku); `None`
/// for any other ticket.
pub fn bug_to_fix(text: &str) -> Option<String> {
    let (pairs, _) = split_front(text)?;
    let get = |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    if get("kind") != Some("bug") || list_items(get("tasks")?).is_none_or(|t| t.is_empty()) {
        return None;
    }
    set_line(text, "kind", "fix")
}

/// `text` with `kind:` set to `pick(id)` when it has frontmatter but no (or an empty) kind
/// (migration 0006); `None` for any other ticket. `id` is the ticket's `id:` value, or empty.
pub fn backfill_kind(text: &str, pick: impl Fn(&str) -> String) -> Option<String> {
    let (pairs, _) = split_front(text)?;
    let get = |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    if get("kind").is_some_and(|k| !k.is_empty()) {
        return None;
    }
    set_line(text, "kind", &pick(get("id").unwrap_or("")))
}

/// The frontmatter as (key, raw value) pairs and the body after it.
fn split_front(text: &str) -> Option<(Vec<(String, String)>, String)> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let end = front_end(&lines)?;
    let pairs = lines[1..end]
        .iter()
        // Indented lines belong to a block (`stages:`), whose keys must not read as fields.
        .filter(|l| !l.starts_with([' ', '\t']))
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
/// point (by file stem, anywhere beneath it).
#[cfg(test)]
pub fn check(root: &Path, docs: &Path) -> Report {
    check_in(root, docs, &Projects::default())
}

/// [`check`], knowing this project's prefix (a ticket's `id:` must carry it, if qualified) and
/// the other registered projects (a reference qualified with one of theirs is looked up there).
pub fn check_in(root: &Path, docs: &Path, projects: &Projects) -> Report {
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
        let mut note = |error: bool, msg: String| {
            let line = format!("{at}: {msg}");
            if error {
                problems.push(line);
            } else {
                warnings.push(line);
            }
        };
        let file_id = id_of(path);
        if file_id
            .as_deref()
            .is_some_and(bridle_api::types::is_reserved_id)
        {
            note(true, "id is reserved (placeholder); mint another".into());
        }
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
        // The new name wins over the old one when both are present.
        let raw = |k: &str| {
            front
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.as_str())
        };
        let get = |k: &str| raw(k).or_else(|| old_name(k).and_then(raw));
        for (old, new) in RENAMED {
            if raw(old).is_some() && raw(new).is_some() {
                note(
                    false,
                    format!("has both {old} and {new}; {new} wins, delete {old}"),
                );
            }
        }
        for k in REQUIRED {
            if get(k).is_none_or(str::is_empty) && !(LIST_FIELDS.contains(&k) && get(k).is_some()) {
                note(true, format!("missing {k}"));
            }
        }
        match get("kind") {
            None | Some("") => note(true, "missing kind".into()),
            Some(k) if k.parse::<TicketKind>().is_err() => note(true, format!("unknown kind {k}")),
            Some(_) => {}
        }
        match (resolved, get("resolved")) {
            (true, None | Some("")) => {
                note(true, "missing resolved (required under resolved/)".into())
            }
            (false, Some(_)) => note(true, "resolved is set but the ticket is under open/".into()),
            _ => {}
        }
        if let Some(t) = get("theme")
            && let Err(e) = check_theme(t)
        {
            note(true, e.to_string());
        }
        if let Some(p) = get("parent") {
            if let Err(e) = check_parent(p) {
                note(true, e.to_string());
            } else if let (Some(pre), tail) = split_id(p)
                && let Some((_, r)) = projects.others.iter().find(|(o, _)| o == pre)
                && !has_ticket(r, tail)
            {
                note(true, format!("parent {p} is not a ticket in project {pre}"));
            }
        }
        if let Some(id) = get("id") {
            // The file name carries the bare id; the `id:` may be qualified (br-k7tm) or not.
            let (prefix, tail) = split_id(id);
            if file_id.as_deref().is_some_and(|f| f != tail) {
                note(true, format!("id {id} doesn't match the file name"));
            }
            if let (Some(p), Some(own)) = (prefix, &projects.own)
                && p != own
            {
                note(
                    true,
                    format!("id {id} has prefix {p}, but this project's is {own}"),
                );
            }
            if let Some(other) = seen.insert(tail.to_string(), path.clone()) {
                note(true, format!("id {id} is also used by {}", shown(&other)));
            }
        }
        check_stages(&text, get("version"), &mut |m| note(true, m));
        for k in LIST_FIELDS {
            let Some(raw) = get(k) else { continue };
            let Some(items) = list_items(raw) else {
                note(true, format!("{k} is not a [a, b] list"));
                continue;
            };
            if k == "blocked_by" || k == "related" {
                for it in items {
                    let (prefix, tail) = split_id(&it);
                    let foreign = match (prefix, &projects.own) {
                        (Some(p), Some(own)) if p != own => Some(p),
                        _ => None,
                    };
                    let found = match foreign {
                        // Another project: looked up if registered here, else only well-formed.
                        Some(p) => projects
                            .others
                            .iter()
                            .find(|(o, _)| o == p)
                            .is_none_or(|(_, r)| has_ticket(r, tail)),
                        None => ids.contains(tail) || stems.contains(&it),
                    };
                    if !found {
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

/// Version 2 needs a well-formed `stages:`; version 1 is not looked at (its `stages`, if any,
/// is just another key).
fn check_stages(text: &str, version: Option<&str>, err: &mut dyn FnMut(String)) {
    match version {
        None | Some("1") => return,
        Some("2") => {}
        Some(v) => return err(format!("unknown version {v}")),
    }
    let front: Vec<&str> = text.lines().skip(1).take_while(|l| *l != "---").collect();
    match crate::stages::parse(&front) {
        Ok(None) => err("version 2 needs a stages: block".into()),
        Ok(Some(s)) => {
            if let Err(e) = crate::stages::validate(&s) {
                err(e);
            }
        }
        Err(e) => err(e),
    }
}

/// The ticket id a task gives its ticket: its id tail, when that is in the ticket alphabet (task
/// ids minted before br-9e15 are hex and may hold `0` or `1`) and no ticket has it yet.
fn reusable_id(root: &Path, task_id: &str) -> Option<String> {
    let tail = task_id.rsplit_once('-')?.1;
    let ok = tail.len() == ID_LEN && tail.bytes().all(|b| ID_ALPHABET.contains(&b));
    (ok && !bridle_api::types::is_reserved_id(tail) && !existing_ids(root).contains(tail))
        .then(|| tail.to_string())
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
        if !taken.contains(&id) && !bridle_api::types::is_reserved_id(&id) {
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

/// `text` with `resolved: <ts>` as the last frontmatter line (replacing any existing one, in
/// either name);
/// `None` when there is no frontmatter block.
fn set_resolved(text: &str, resolved: &str) -> Option<String> {
    let mut lines: Vec<&str> = text.lines().collect();
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    lines.retain({
        let mut i = 0;
        move |l| {
            i += 1;
            !(i > 1 && i <= end && (l.starts_with("resolved:") || l.starts_with("closed:")))
        }
    });
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    let stamp = format!("resolved: {resolved}");
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
            kind: TicketKind::Feature,
            repos,
            blocked_by: &[],
            related: &[],
            created_by: None,
            theme: None,
            prefix: None,
            stages: &[],
        }
    }

    #[test]
    fn pick_prefix_uses_the_config_when_the_daemon_reports_none() {
        let r = Path::new("/p");
        assert_eq!(pick_prefix(None, Some("ui".into()), r).unwrap(), "ui");
        assert_eq!(pick_prefix(Some("ui".into()), None, r).unwrap(), "ui");
        assert_eq!(
            pick_prefix(Some("ui".into()), Some("ui".into()), r).unwrap(),
            "ui"
        );
    }

    #[test]
    fn pick_prefix_refuses_with_no_source_or_a_mismatch() {
        let r = Path::new("/p");
        assert!(
            pick_prefix(None, None, r)
                .unwrap_err()
                .to_string()
                .contains("no task prefix")
        );
        let e = pick_prefix(Some("br".into()), Some("ui".into()), r).unwrap_err();
        assert!(e.to_string().contains("differs"));
    }

    fn row_with_ticket(ticket: Option<&str>) -> bridle_api::Ticket {
        serde_json::from_value(serde_json::json!({
            "id": "br-abcd", "title": "T", "kind": "feature", "state": "open", "body": "x",
            "thread": [], "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z", "ticket": ticket,
        }))
        .unwrap()
    }

    #[test]
    fn project_flag_for_another_clone_is_refused() {
        assert!(check_project_matches(Some("a"), Some("b")).is_err());
        assert!(check_project_matches(Some("a"), Some("a")).is_ok());
        assert!(check_project_matches(None, Some("b")).is_ok());
        assert!(check_project_matches(Some("a"), None).is_ok());
    }

    #[test]
    fn refused_mint_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t-abcd.md");
        std::fs::write(&path, "x").unwrap();
        let e = discard_unfiled(&path, "t-abcd.md", &["x".to_string()], "forbidden");
        assert!(!path.exists());
        assert!(e.to_string().contains("forbidden"));
    }

    #[test]
    fn from_task_refuses_a_row_that_already_has_a_ticket_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("open")).unwrap();
        std::fs::write(dir.path().join("open/old-thing-wxyz.md"), "x").unwrap();
        assert!(check_not_linked(dir.path(), &row_with_ticket(Some("wxyz"))).is_err());
        assert!(check_not_linked(dir.path(), &row_with_ticket(Some("zzzz"))).is_ok());
        assert!(check_not_linked(dir.path(), &row_with_ticket(None)).is_ok());
    }

    #[test]
    fn ticket_of_reads_the_field_then_the_old_body_line() {
        let mut t: bridle_api::Ticket = serde_json::from_value(serde_json::json!({
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
                "---\nid: {id}\ntitle: \"Fix the: thing (now)\"\nkind: feature\ncreated: 2026-09-30\nrepos: [proj]\nchanges: []\nspecs: []\nblocked_by: []\nrelated: []\n---\n\n## The ask\n\n"
            )
        );
    }

    #[test]
    fn new_records_created_by_and_check_accepts_it_or_its_absence() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("docs/tickets");
        let repos = vec!["proj".to_string()];
        let mut f = fields("Who filed this", &repos);
        f.created_by = Some("external:aide");
        let p = create(&root, &f, "2026-10-07", &HashSet::new()).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(
            text.contains("created: 2026-10-07\ncreated_by: external:aide\nrepos: [proj]\n"),
            "{text}"
        );
        // An old ticket with no `created_by` still passes alongside it.
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
    fn resolve_moves_and_stamps_resolved() {
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
            text.contains("related: []\nresolved: 2026-10-01T12:00:00Z\n---\n"),
            "{text}"
        );
        assert!(resolve(dir.path(), &id, "x").is_err());
    }

    #[test]
    fn a_version_1_ticket_is_written_as_before() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let p = create(
            dir.path(),
            &fields("Thing", &repos),
            "2026-09-30",
            &HashSet::new(),
        )
        .unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(
            !text.contains("version") && !text.contains("stages") && !text.contains("Proposal")
        );
        assert!(
            text.ends_with("related: []\n---\n\n## The ask\n\n"),
            "{text}"
        );
        let r = check(dir.path(), dir.path());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
    }

    #[test]
    fn a_version_2_ticket_checks_resolves_with_its_folder_and_rejects_bad_stages() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let stages = crate::stages::build(
            &["design".into(), "specs".into(), "tasks".into()],
            &["proposal".into(), "specs".into()],
        )
        .unwrap();
        let f = Fields {
            stages: &stages,
            ..fields("Thing", &repos)
        };
        let p = create(dir.path(), &f, "2026-10-17", &HashSet::new()).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains("version: 2\nrepos:"), "{text}");
        assert!(
            text.contains("  tasks: {requires: [design, specs]}\n---\n"),
            "{text}"
        );
        let (front, body) = split_front(&text).unwrap();
        assert!(
            !front.iter().any(|(k, _)| k == "design"),
            "indented keys are not fields"
        );
        assert!(crate::stages::find_proposal(&body).is_some());
        let r = check(dir.path(), dir.path());
        assert!(r.errors.is_empty(), "{:?}", r.errors);

        let folder = crate::stages::folder_of(&p);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("design.md"), "d").unwrap();
        let dest = resolve(dir.path(), &id_of(&p).unwrap(), "2026-10-18T00:00:00Z").unwrap();
        assert!(!folder.exists());
        assert!(crate::stages::folder_of(&dest).join("design.md").is_file());

        let bad = text
            .replace(
                "  design: {requires: [proposal]}",
                "  design: {requires: [tasks]}",
            )
            .replace("resolved", "");
        std::fs::write(
            &dest,
            bad.replace("---\n\n## The ask", "resolved: x\n---\n\n## The ask"),
        )
        .unwrap();
        let r = check(dir.path(), dir.path());
        assert!(
            r.errors.iter().any(|e| e.contains("cycle")),
            "{:?}",
            r.errors
        );
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
    fn set_resolved_replaces_an_existing_stamp_in_either_name() {
        for old in ["resolved", "closed"] {
            let t = set_resolved(&format!("---\nid: a\n{old}: old\n---\nbody\n"), "new").unwrap();
            assert_eq!(t, "---\nid: a\nresolved: new\n---\nbody\n");
        }
        assert!(set_resolved("no frontmatter", "x").is_none());
    }

    const FRONT: &str =
        "opened: 2026-09-30\nrepos: [p]\nchanges: []\nspecs: []\nneeds: []\nsee: []\n";

    fn write(root: &Path, dir: &str, name: &str, id: &str, extra: &str, body: &str) {
        let d = root.join(dir);
        std::fs::create_dir_all(&d).unwrap();
        let kind = if extra.contains("kind:") {
            ""
        } else {
            "kind: chore\n"
        };
        let text = format!("---\nid: {id}\ntitle: T\n{FRONT}{kind}{extra}---\n{body}\n");
        std::fs::write(d.join(name), text).unwrap();
    }

    fn report(root: &Path) -> Report {
        check(&root.join("docs/tickets"), &root.join("docs"))
    }

    fn problems(root: &Path) -> String {
        report(root).errors.join("\n")
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
    fn a_reserved_id_is_reported_and_never_reused() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        write(&t, "open", "a-thing-xxxx.md", "xxxx", "", "");
        assert!(problems(dir.path()).contains("reserved"));
        assert_eq!(reusable_id(&t, "br-xxxx"), None);
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
            "resolved is set but the ticket is under open/",
            "missing resolved (required under resolved/)",
            "id eeee is also used by",
            "blocked_by names nope, which is not a ticket",
            "link [[nowhere]] points at no file",
        ] {
            assert!(p.contains(want), "missing {want:?} in:\n{p}");
        }
        assert!(!p.contains("in-code") && !p.contains("inline-code"), "{p}");
        assert!(!p.contains("related names"), "{p}");
    }

    #[test]
    fn check_warns_on_missing_kind_and_tasks_and_errors_on_unknown_kind() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        // No kind: the helpers would add one.
        std::fs::create_dir_all(t.join("open")).unwrap();
        std::fs::write(
            t.join("open/a-aaaa.md"),
            format!("---\nid: aaaa\ntitle: T\n{FRONT}---\n\n"),
        )
        .unwrap();
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
        let r = report(dir.path());
        let e = r.errors.join("\n");
        assert_eq!(r.errors.len(), 2, "{e}");
        assert!(e.contains("b-bbbb.md: unknown kind nonsense"), "{e}");
        assert!(e.contains("a-aaaa.md: missing kind"), "{e}");
        assert!(!e.contains("c-cccc"), "{e}");
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
        assert!(set(dir.path(), &id, "tasks", "br-1111").is_err());
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
        set(dir.path(), &id, "related", "").unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains("title: \"New: title\"\n"), "{text}");
        assert!(text.contains("repos: [a, b]\n"), "{text}");
        assert!(
            text.contains("blocked_by: [xxxx]\nrelated: []\n---\n"),
            "{text}"
        );
        assert!(text.ends_with("## The ask\n\n"));
        // A resolved ticket is editable too, and `resolved:` stays last.
        let dest = resolve(dir.path(), &id, "2026-10-01T00:00:00Z").unwrap();
        set(dir.path(), &id, "specs", "s1").unwrap();
        let text = std::fs::read_to_string(dest).unwrap();
        assert!(text.contains("specs: [s1]\n"), "{text}");
        assert!(
            text.contains("resolved: 2026-10-01T00:00:00Z\n---\n"),
            "{text}"
        );
        for f in ["id", "created", "opened", "resolved", "closed"] {
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
        assert_eq!((title.as_str(), kind), ("Thing", TicketKind::Feature));
    }

    #[test]
    fn no_ask_warns_the_filer_and_an_ask_does_not() {
        assert!(no_ask_warning("").unwrap().contains("cannot be readied"));
        assert!(no_ask_warning(" \n").is_some());
        assert!(no_ask_warning("Do it.").is_none());
    }

    #[test]
    fn a_body_with_its_own_ask_heading_gets_exactly_one() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let mut f = fields("Thing", &repos);
        f.ask = "## The ask\n\nDo the thing.\n";
        let p = create(dir.path(), &f, "2026-10-03", &HashSet::new()).unwrap();
        let text = std::fs::read_to_string(p).unwrap();
        assert_eq!(text.matches("## The ask").count(), 1, "{text}");
        assert!(task_source(&text).is_ok());
    }

    #[test]
    fn ready_reads_the_first_non_empty_ask() {
        let head = "---\nid: x\ntitle: T\nkind: feature\n---\n\n";
        let ok = format!("{head}## The ask\n\n## The ask\n\nDo it.\n");
        assert!(task_source(&ok).is_ok());
        let empty = format!("{head}## The ask\n\n## The ask\n\n");
        assert!(task_source(&empty).is_err());
    }

    #[test]
    fn ticket_ready_refuses_an_empty_ask() {
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
    fn ticket_ready_refuses_an_uncommitted_ticket() {
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
        assert!(ensure_committed(dir.path(), "main", Path::new("a.md")).is_ok());
        let err = ensure_committed(dir.path(), "main", Path::new("b.md")).unwrap_err();
        assert!(err.to_string().contains("not committed on main"), "{err}");
        // A dev integration branch: a file only on main is refused, one on dev is accepted.
        git(&["checkout", "-q", "-b", "dev"]);
        std::fs::write(dir.path().join("c.md"), "z").unwrap();
        git(&["add", "c.md"]);
        git(&["commit", "-q", "-m", "c"]);
        assert!(ensure_committed(dir.path(), "dev", Path::new("c.md")).is_ok());
        let err = ensure_committed(dir.path(), "main", Path::new("c.md")).unwrap_err();
        assert!(err.to_string().contains("not committed on main"), "{err}");
    }
    const NEW_FRONT: &str =
        "created: 2026-09-30\nrepos: [p]\nchanges: []\nspecs: []\nblocked_by: []\nrelated: []\n";

    fn write_raw(root: &Path, dir: &str, name: &str, front: &str) {
        let d = root.join(dir);
        std::fs::create_dir_all(&d).unwrap();
        let kind = if front.contains("kind:") {
            ""
        } else {
            "kind: chore\n"
        };
        std::fs::write(d.join(name), format!("---\n{front}{kind}---\nbody\n")).unwrap();
    }

    #[test]
    fn check_reads_old_and_new_names_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        write(&t, "open", "old-aaaa.md", "aaaa", "", "");
        write_raw(
            &t,
            "open",
            "new-bbbb.md",
            &format!("id: bbbb\ntitle: T\n{NEW_FRONT}theme: human-ui\nparent: aaaa\n"),
        );
        write_raw(
            &t,
            "resolved",
            "done-cccc.md",
            &format!("id: cccc\ntitle: T\n{NEW_FRONT}parent: br-aaaa\nresolved: 2026-10-01\n"),
        );
        let r = report(dir.path());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert!(
            r.warnings.iter().all(|w| !w.contains("both")),
            "{:?}",
            r.warnings
        );
    }

    #[test]
    fn check_warns_when_both_names_are_present_and_the_new_one_wins() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        // `needs` names a missing ticket, but `blocked_by` (empty) wins, so no error.
        write_raw(
            &t,
            "open",
            "both-aaaa.md",
            &format!("id: aaaa\ntitle: T\n{NEW_FRONT}needs: [nope]\n"),
        );
        let r = report(dir.path());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert!(
            r.warnings
                .iter()
                .any(|w| w.contains("has both needs and blocked_by")),
            "{:?}",
            r.warnings
        );
    }

    #[test]
    fn check_validates_theme_and_parent() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        for (name, id, extra) in [
            ("a-aaaa.md", "aaaa", "theme: Human_UI\n"),
            ("b-bbbb.md", "bbbb", "parent: br--x\n"),
            (
                "c-cccc.md",
                "cccc",
                "theme: agents-and-cli\nparent: br-aaaa\n",
            ),
        ] {
            write_raw(
                &t,
                "open",
                name,
                &format!("id: {id}\ntitle: T\n{NEW_FRONT}{extra}"),
            );
        }
        let p = problems(dir.path());
        assert!(p.contains("a-aaaa.md: theme"), "{p}");
        assert!(p.contains("b-bbbb.md: parent"), "{p}");
        assert!(!p.contains("c-cccc"), "{p}");
    }

    #[test]
    fn set_maps_old_names_and_validates_theme_and_parent() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        write(&t, "open", "old-aaaa.md", "aaaa", "", "");
        let root = &t;
        set(root, "aaaa", "see", "bbbb").unwrap();
        set(root, "aaaa", "theme", "reliability").unwrap();
        set(root, "aaaa", "parent", "br-k7tm").unwrap();
        let text = std::fs::read_to_string(t.join("open/old-aaaa.md")).unwrap();
        assert!(text.contains("related: [bbbb]\n"), "{text}");
        assert!(!text.contains("see:"), "{text}");
        assert!(text.contains("theme: reliability\n") && text.contains("parent: br-k7tm\n"));
        for bad in ["Reliability", "a_b", "-a", ""] {
            assert!(set(root, "aaaa", "theme", bad).is_err(), "{bad}");
        }
        assert!(set(root, "aaaa", "parent", "a b").is_err());
    }

    #[test]
    fn new_writes_theme_and_the_new_names_only() {
        let dir = tempfile::tempdir().unwrap();
        let repos = vec!["p".to_string()];
        let mut f = fields("Thing", &repos);
        f.theme = Some("human-ui");
        let p = create(dir.path(), &f, "2026-10-10", &HashSet::new()).unwrap();
        let text = std::fs::read_to_string(p).unwrap();
        assert!(
            text.contains("created: 2026-10-10\ntheme: human-ui\nrepos: [p]\n"),
            "{text}"
        );
        for old in ["opened:", "needs:", "see:", "filed_by:", "closed:"] {
            assert!(!text.contains(old), "{old} in {text}");
        }
    }

    fn projects(own: &str, others: &[(&str, &Path)]) -> Projects {
        Projects {
            own: Some(own.to_string()),
            others: others
                .iter()
                .map(|(p, r)| (p.to_string(), r.to_path_buf()))
                .collect(),
        }
    }

    #[test]
    fn link_adds_and_removes_frontmatter_links() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("docs/tickets");
        let repos = vec!["p".to_string()];
        let mut f = fields("Linked", &repos);
        f.prefix = Some("br");
        let p = create(&root, &f, "2026-10-10", &HashSet::new()).unwrap();
        let id = id_of(&p).unwrap();
        let ctx = projects("br", &[]);
        let go = |kind, other: &str, rm| link_in(&root, &id, kind, other, rm, &ctx).unwrap();
        go(LinkKind::BlockedBy, "br-aaaa", false);
        go(LinkKind::BlockedBy, "br-bbbb", false);
        go(LinkKind::BlockedBy, "br-aaaa", false);
        go(LinkKind::Parent, "br-cccc", false);
        go(LinkKind::Related, "tw-dddd", false);
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains("blocked_by: [br-aaaa, br-bbbb]"), "{text}");
        assert!(text.contains("parent: br-cccc"), "{text}");
        assert!(text.contains("related: [tw-dddd]"), "{text}");
        go(LinkKind::BlockedBy, "br-aaaa", true);
        go(LinkKind::Parent, "br-cccc", true);
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains("blocked_by: [br-bbbb]"), "{text}");
        assert!(!text.contains("parent:"), "{text}");
    }

    #[test]
    fn split_id_reads_qualified_and_bare() {
        assert_eq!(split_id("br-k7tm"), (Some("br"), "k7tm"));
        assert_eq!(split_id("k7tm"), (None, "k7tm"));
        assert_eq!(split_id("a-thing-k7tm"), (None, "a-thing-k7tm"));
    }

    #[test]
    fn new_with_a_prefix_writes_a_qualified_id_and_a_bare_file_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("docs/tickets");
        let repos = vec!["p".to_string()];
        let mut f = fields("Qualified", &repos);
        f.prefix = Some("br");
        let p = create(&root, &f, "2026-10-10", &HashSet::new()).unwrap();
        let id = id_of(&p).unwrap();
        assert!(p.to_string_lossy().ends_with(&format!("qualified-{id}.md")));
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.starts_with(&format!("---\nid: br-{id}\n")), "{text}");
        // File-name tail matches the id without its prefix; both forms find it.
        let ctx = projects("br", &[]);
        assert_eq!(problems_in(dir.path(), &ctx), "");
        assert!(find_by_id(&root.join("open"), &format!("br-{id}")).is_ok());
        assert!(find_by_id(&root.join("open"), &id).is_ok());
    }

    fn problems_in(root: &Path, ctx: &Projects) -> String {
        check_in(&root.join("docs/tickets"), &root.join("docs"), ctx)
            .errors
            .join("\n")
    }

    #[test]
    fn check_reads_old_bare_and_new_qualified_ids_and_refs() {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("docs/tickets");
        write(&t, "open", "old-one-aaaa.md", "aaaa", "", "");
        write(&t, "open", "new-one-bbbb.md", "br-bbbb", "", "");
        write(
            &t,
            "open",
            "refs-cccc.md",
            "br-cccc",
            "blocked_by: [br-aaaa, bbbb]\nrelated: [other-9999]\n",
            "",
        );
        let ctx = projects("br", &[]);
        // `other-9999` is a foreign-looking ref with no registered project: well-formed only.
        assert_eq!(problems_in(dir.path(), &ctx), "");
        // A wrong prefix on the id and a mismatched tail are both caught.
        write(&t, "open", "bad-dddd.md", "zz-dddd", "", "");
        write(&t, "open", "bad2-eeee.md", "br-ffff", "", "");
        let p = problems_in(dir.path(), &ctx);
        assert!(p.contains("has prefix zz"), "{p}");
        assert!(p.contains("id br-ffff doesn't match the file name"), "{p}");
    }

    #[test]
    fn set_expands_bare_refs_and_refuses_ambiguous_or_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            dir.path().join("a/docs/tickets"),
            dir.path().join("b/docs/tickets"),
            dir.path().join("c/docs/tickets"),
        );
        write(&a, "open", "mine-aaaa.md", "br-aaaa", "", "");
        write(&a, "open", "src-eeee.md", "br-eeee", "", "");
        write(&b, "open", "theirs-bbbb.md", "tw-bbbb", "", "");
        write(&b, "open", "both-cccc.md", "tw-cccc", "", "");
        write(&c, "open", "both2-cccc.md", "hx-cccc", "", "");
        let ctx = projects("br", &[("tw", &b), ("hx", &c)]);
        // Bare in this project -> qualified with its prefix; qualified stays.
        set_in(&a, "eeee", "blocked_by", "aaaa, tw-bbbb", &ctx).unwrap();
        // Found in exactly one other project -> that project's prefix.
        set_in(&a, "br-eeee", "related", "bbbb", &ctx).unwrap();
        let text = std::fs::read_to_string(a.join("open/src-eeee.md")).unwrap();
        assert!(text.contains("blocked_by: [br-aaaa, tw-bbbb]"), "{text}");
        assert!(text.contains("related: [tw-bbbb]"), "{text}");
        // Ambiguous across projects: refused with the candidates.
        let e = set_in(&a, "eeee", "related", "cccc", &ctx)
            .unwrap_err()
            .to_string();
        assert!(e.contains("tw-cccc") && e.contains("hx-cccc"), "{e}");
        // Unknown everywhere: refused.
        assert!(set_in(&a, "eeee", "related", "9999", &ctx).is_err());
        // Parent takes another project's qualified id as is, and expands a bare one.
        set_in(&a, "eeee", "parent", "tw-bbbb", &ctx).unwrap();
        set_in(&a, "eeee", "parent", "aaaa", &ctx).unwrap();
        let text = std::fs::read_to_string(a.join("open/src-eeee.md")).unwrap();
        assert!(text.contains("parent: br-aaaa"), "{text}");
        // check looks a registered project's reference up there.
        set_in(&a, "eeee", "parent", "tw-zzzz", &Projects::default()).unwrap();
        let p = check_in(&a, &dir.path().join("a/docs"), &ctx)
            .errors
            .join("\n");
        assert!(
            p.contains("parent tw-zzzz is not a ticket in project tw"),
            "{p}"
        );
    }

    #[test]
    fn local_refuses_another_projects_id() {
        let ctx = projects("br", &[]);
        assert_eq!(ctx.local("br-k7tm").unwrap(), "k7tm");
        assert_eq!(ctx.local("k7tm").unwrap(), "k7tm");
        let e = ctx.local("tw-k7tm").unwrap_err().to_string();
        assert!(e.contains("belongs to project tw"), "{e}");
    }
}

#[cfg(test)]
mod relayed_comment_tests {
    use super::is_humans;
    use bridle_api::types::{ThreadEntry, ThreadEntryKind};

    fn entry(from: &str, for_human: bool) -> ThreadEntry {
        ThreadEntry {
            kind: ThreadEntryKind::Note,
            from: from.to_string(),
            body: "x".to_string(),
            at: chrono::Utc::now(),
            for_human,
        }
    }

    #[test]
    fn next_comments_counts_the_humans_and_what_an_agent_relayed() {
        assert!(is_humans(&entry("human", false)));
        assert!(is_humans(&entry("external:advisor/tj7q", true)));
        assert!(!is_humans(&entry("external:advisor/tj7q", false)));
    }
}
