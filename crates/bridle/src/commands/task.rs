//! Task commands: task, claim, release, ready, queue, dep, land, conflict, impact, ask, answer.

use super::*;

pub(super) async fn ask(cli: &Cli, args: &AskArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client
        .ask_question(&args.task, &args.text, args.to.as_deref())
        .await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("asked on {}", task.id);
    }
    Ok(())
}

pub(super) async fn answer(cli: &Cli, args: &AnswerArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.answer_question(&args.task, &args.text).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("answered on {}", task.id);
    }
    Ok(())
}

pub(super) async fn claim(cli: &Cli, args: &ClaimArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.claim_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("claimed {}", task.id);
    }
    Ok(())
}

pub(super) async fn release(cli: &Cli, args: &ReleaseArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.release_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("released {}", task.id);
    }
    Ok(())
}

pub(super) fn task_kind_arg(k: TaskKindArg) -> TaskKind {
    match k {
        TaskKindArg::Feature => TaskKind::Feature,
        TaskKindArg::Bug => TaskKind::Bug,
        TaskKindArg::Chore => TaskKind::Chore,
        TaskKindArg::Question => TaskKind::Question,
        TaskKindArg::Research => TaskKind::Research,
        TaskKindArg::Explore => TaskKind::Explore,
        TaskKindArg::ArchRevision => TaskKind::ArchRevision,
        TaskKindArg::ReEvaluate => TaskKind::ReEvaluate,
        TaskKindArg::Incident => TaskKind::Incident,
    }
}

pub(super) async fn task(cli: &Cli, args: &TaskArgs) -> Result<(), CliError> {
    match &args.action {
        TaskAction::New(a) => task_new(cli, a).await,
        TaskAction::Show(a) => task_show(cli, a).await,
        TaskAction::Edit(a) => task_edit(cli, a).await,
        TaskAction::List(a) => task_list(cli, a).await,
        TaskAction::Plan(a) => task_plan(cli, a).await,
        TaskAction::Drop(a) => task_drop(cli, a).await,
        TaskAction::Priority(a) => task_priority(cli, a).await,
        TaskAction::Kind(a) => task_kind(cli, a).await,
        TaskAction::Done(a) => task_done(cli, a).await,
        TaskAction::Summary(a) => task_summary(cli, a).await,
        TaskAction::Reopen(a) => task_reopen(cli, a).await,
        TaskAction::SkipSettle(a) => task_skip_settle(cli, a).await,
        TaskAction::Watch(a) => task_watch(cli, a, true).await,
        TaskAction::Unwatch(a) => task_watch(cli, a, false).await,
        TaskAction::Comment(a) => task_comment(cli, a).await,
        TaskAction::Search(a) => task_search(cli, a).await,
        _ => unreachable!("normalize forwards the queue and coordination actions"),
    }
}

/// Whether the project's `.bridle/config.toml` (read from the current
/// directory, like `bridle rules`) defines any components. Best effort: the
/// reminder it gates is only a nudge.
pub(super) fn project_has_components() -> bool {
    std::env::current_dir()
        .ok()
        .and_then(|d| bridle_daemon::config::Config::load(&d).ok())
        .is_some_and(|c| !c.components.is_empty())
}

pub(super) fn size_str(size: Option<TaskSize>) -> &'static str {
    size.map_or("-", TaskSize::as_str)
}

pub(super) fn task_size_arg_to_opt(arg: TaskSizeArg) -> Option<TaskSize> {
    match arg {
        TaskSizeArg::S => Some(TaskSize::S),
        TaskSizeArg::M => Some(TaskSize::M),
        TaskSizeArg::L => Some(TaskSize::L),
        TaskSizeArg::None => Some(TaskSize::None),
    }
}

use bridle_api::settle_clock_text;
use bridle_api::types::sort_by_priority;

pub fn print_task_row(t: &Task) {
    println!(
        "{:<10} {:<9} {:<8} {:<4} {:<8} {}",
        t.id,
        t.kind,
        t.state,
        size_str(t.size),
        t.priority,
        t.title
    );
}

pub(super) async fn task_new(cli: &Cli, args: &TaskNewArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = if args.body.is_some() || args.body_file.is_some() {
        read_text(&args.body, &args.body_file, "body")?
    } else {
        String::new()
    };
    let req = NewTaskRequest {
        ticket: None,
        parent: args.from.clone(),
        for_human: args.for_human,
        title: args.title.clone(),
        kind: task_kind_arg(args.kind),
        body,
        components: args.component.clone(),
        size: args.size.and_then(task_size_arg_to_opt),
        priority: args.priority.map(task_priority_arg),
    };
    let task = client.new_task(&req).await?;
    if args.for_human {
        client
            .send(&SendRequest {
                to: Some("human".to_string()),
                body: format!(
                    "To-do for you ({} priority): {}. Finish it with `bridle task done {}`.",
                    task.priority, task.title, task.id
                ),
                kind: MessageKind::Note,
                when: bridle_api::When::Now,
                reply_to: None,
                task: Some(task.id.clone()),
            })
            .await?;
    }
    if args.component.is_empty() && project_has_components() {
        eprintln!(
            "note: no --component given; this task is repo-wide (see `bridle task edit --component`)"
        );
    }
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_show(cli: &Cli, args: &TaskShowArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let task = client.get_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("id          {}", task.id);
        println!("title       {}", task.title);
        println!("kind        {}", task.kind);
        println!("state       {}", task.state);
        println!("priority    {}", task.priority);
        if let Some(ticket) = &task.ticket {
            println!("ticket      {ticket}");
        }
        if let Some(size) = task.size {
            println!("size        {size}");
        }
        if let Some(branch) = &task.branch {
            println!("branch      {branch}");
        }
        if let Some(commit) = &task.commit {
            println!("commit      {commit}");
        }
        println!("created     {}", task.created_at.to_rfc3339());
        println!("created by  {}", task.created_by);
        if !task.watchers.is_empty() {
            println!("watchers    {}", task.watchers.join(", "));
        }
        println!("updated     {}", task.updated_at.to_rfc3339());
        if let Some(until) = task.settle_until {
            println!("settling    until {}", settle_clock_text(until));
        }
        if !task.components.is_empty() {
            println!("components  {}", task.components.join(", "));
        }
        if !task.body.is_empty() {
            println!();
            println!("{}", task.body);
        }
        if let Some(summary) = &task.summary {
            println!();
            println!("Summary:");
            println!("{summary}");
        }
        if !task.thread.is_empty() {
            println!();
            println!("Thread:");
            for e in &task.thread {
                println!(
                    "  [{}] {} ({}): {}",
                    e.kind,
                    e.from,
                    e.at.to_rfc3339(),
                    e.body
                );
            }
        }
    }
    Ok(())
}

pub(super) async fn task_edit(cli: &Cli, args: &TaskEditArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = if args.body.is_some() || args.body_file.is_some() {
        Some(read_text(&args.body, &args.body_file, "body")?)
    } else {
        None
    };
    let req = EditTaskRequest {
        title: args.title.clone(),
        body,
        components: if args.no_component {
            Some(Vec::new())
        } else if args.component.is_empty() {
            None
        } else {
            Some(args.component.clone())
        },
        size: args.size.and_then(task_size_arg_to_opt),
        ticket: None,
    };
    let task = client.edit_task(&args.task, &req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_list(cli: &Cli, args: &TaskListArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let mut tasks = match &args.claimed_by {
        Some(claimed_by) => client.list_tasks_claimed_by(claimed_by).await?,
        None => client.list_tasks().await?,
    };
    if let Some(component) = &args.component {
        // Both filters at once: the daemon takes one, so intersect by id.
        let scoped = client.list_tasks_component(component).await?;
        tasks.retain(|t| scoped.iter().any(|s| s.id == t.id));
    }
    if let Some(kind) = args.kind {
        let kind = task_kind_arg(kind);
        tasks.retain(|t| t.kind == kind);
    }
    sort_by_priority(&mut tasks);
    if cli.json {
        render::print_json(&tasks)?;
    } else if tasks.is_empty() {
        println!("no tasks");
    } else {
        println!(
            "{:<10} {:<9} {:<8} {:<4} {:<8} TITLE",
            "ID", "KIND", "STATE", "SIZE", "PRI"
        );
        for t in &tasks {
            print_task_row(t);
        }
    }
    Ok(())
}

async fn task_ready(cli: &Cli, id: &str) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.ready_task(id).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_plan(cli: &Cli, args: &TaskPlanArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.plan_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) fn task_priority_arg(arg: TaskPriorityArg) -> TaskPriority {
    match arg {
        TaskPriorityArg::Critical => TaskPriority::Critical,
        TaskPriorityArg::Urgent => TaskPriority::Urgent,
        TaskPriorityArg::High => TaskPriority::High,
        TaskPriorityArg::Normal => TaskPriority::Normal,
        TaskPriorityArg::Low => TaskPriority::Low,
    }
}

pub(super) async fn task_priority(cli: &Cli, args: &TaskPriorityArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let req = SetPriorityRequest {
        priority: task_priority_arg(args.priority),
    };
    let task = client.set_task_priority(&args.task, &req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_kind(cli: &Cli, args: &TaskKindArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let req = SetKindRequest {
        kind: task_kind_arg(args.kind),
    };
    let task = client.set_task_kind(&args.task, &req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_drop(cli: &Cli, args: &TaskDropArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let req = DropTaskRequest {
        reason: args.reason.clone(),
    };
    let task = client.drop_task(&args.task, &req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn land(cli: &Cli, args: &LandArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let r = client
        .land_task(
            &args.task,
            &LandRequest {
                branch: args.branch.clone(),
                check_cmd: args.check_cmd.clone(),
                checked_commit: args.checked_commit.clone(),
            },
        )
        .await?;
    if cli.json {
        render::print_json(&r)?;
    } else {
        println!("landed {} as {}", r.task.id, r.commit);
        for n in &r.notes {
            println!("note: {n}");
        }
    }
    Ok(())
}

pub(super) async fn task_done(cli: &Cli, args: &TaskDoneArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let original_task = client.get_task(&args.task).await.ok();
    let was_human_claimed = original_task
        .as_ref()
        .and_then(|t| t.claimed_by.as_deref())
        .map(|c| c == "human")
        .unwrap_or(false);
    let req = DoneTaskRequest {
        commit: args.commit.clone(),
        branch: args.branch.clone(),
        resolution: args.resolution.clone(),
    };
    let task = match client.done_task(&args.task, &req).await {
        Ok(t) => t,
        Err(e) => {
            // Refused because the work isn't on the integration branch: say whether it
            // would merge cleanly, best effort.
            if let Some(branch) = &args.branch
                && let Ok(r) = client
                    .probe(&ProbeRequest {
                        target: None,
                        branch: Some(branch.clone()),
                    })
                    .await
                && r.outcome != ProbeOutcome::Clean
            {
                eprintln!("warning: {}", probe_line(&r));
            }
            return Err(e.into());
        }
    };
    if task.summary.is_none() && !was_human_claimed {
        eprintln!(
            "warning: {} has no summary; record one with `bridle task summary {} --text ...`",
            task.id, task.id
        );
    }
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn impact(cli: &Cli, args: &ImpactArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let (task, set) = match &args.action {
        ImpactAction::Set(a) => {
            let impact = Impact {
                modify: a.modify.clone(),
                add_under: a.add_under.clone(),
                remove: a.remove.clone(),
                files: a.files.clone(),
            };
            (
                client
                    .set_task_impact(&a.task, &SetImpactRequest { impact })
                    .await?,
                true,
            )
        }
        ImpactAction::Show(a) => (client.get_task(&a.task).await?, false),
        ImpactAction::Check(a) => return impact_check(cli, &client, &a.specs).await,
    };
    if cli.json {
        render::print_json(&task.impact)?;
    } else if task.impact.is_empty() {
        println!("{}: no impact declared", task.id);
    } else {
        if set {
            println!("{}: impact set", task.id);
        }
        let i = &task.impact;
        for (label, ids) in [
            ("modify", &i.modify),
            ("add-under", &i.add_under),
            ("remove", &i.remove),
            ("files", &i.files),
        ] {
            if !ids.is_empty() {
                println!("{label:<10}{}", ids.join(", "));
            }
        }
    }
    Ok(())
}

/// Best effort: id -> (requirement, capability) from the specs directory; empty on any
/// problem, which drops the capability level.
pub(super) fn spec_map(dir: &Path) -> BTreeMap<String, SpecRef> {
    let mut map = BTreeMap::new();
    let mut files = Vec::new();
    if spec_files(dir, &mut files).is_err() {
        return map;
    }
    for f in files {
        let Some(capability) = f.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
            continue;
        };
        let Ok(spec) = bridle_spec::parse_file(&f) else {
            continue;
        };
        for r in &spec.requirements {
            let Some(rid) = &r.id else { continue };
            let sref = SpecRef {
                requirement: rid.clone(),
                capability: capability.clone(),
            };
            map.insert(rid.clone(), sref.clone());
            for sid in r.scenarios.iter().filter_map(|s| s.id.as_ref()) {
                map.insert(sid.clone(), sref.clone());
            }
        }
    }
    map
}

pub(super) async fn impact_check(cli: &Cli, client: &Client, specs: &Path) -> Result<(), CliError> {
    let report = client
        .impact_check(&ImpactCheckRequest {
            spec_map: spec_map(specs),
        })
        .await?;
    if cli.json {
        render::print_json(&report)?;
    } else if report.overlaps.is_empty() && report.probes.is_empty() {
        println!("no overlaps");
    }
    if !cli.json {
        for p in &report.probes {
            let level = format!("{:?}", p.level).to_lowercase();
            println!(
                "{level:<9}{:<12}{}  {}",
                "merge",
                p.tasks.join(" "),
                probe_line(&p.result)
            );
        }
        for o in &report.overlaps {
            let level = format!("{:?}", o.level).to_lowercase();
            println!(
                "{level:<9}{:<12}{}  {} {}",
                o.kind, o.key, o.tasks[0], o.tasks[1]
            );
        }
    }
    if report
        .overlaps
        .iter()
        .any(|o| o.level == OverlapLevel::Conflict)
        || report.probes.iter().any(|p| {
            p.level == OverlapLevel::Conflict && p.result.outcome == ProbeOutcome::Conflict
        })
    {
        return Err(CliError::Other(anyhow::anyhow!("impact conflict")));
    }
    Ok(())
}

pub(super) async fn task_summary(cli: &Cli, args: &TaskSummaryArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let text = read_text(&args.text, &args.file, "summary")?;
    let task = client
        .set_task_summary(&args.task, &SetSummaryRequest { text })
        .await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_reopen(cli: &Cli, args: &TaskReopenArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.reopen_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_skip_settle(cli: &Cli, args: &TaskSkipSettleArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.skip_settle(&args.task, &args.reason).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("skipped the settle period on {}", task.id);
    }
    Ok(())
}

pub(super) async fn task_watch(
    cli: &Cli,
    args: &TaskWatchArgs,
    watch: bool,
) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = if watch {
        client.watch_task(&args.task).await?
    } else {
        client.unwatch_task(&args.task).await?
    };
    if cli.json {
        render::print_json(&task)?;
    } else {
        let verb = if watch {
            "watching"
        } else {
            "no longer watching"
        };
        println!("{verb} {}", task.id);
    }
    Ok(())
}

pub(super) async fn task_comment(cli: &Cli, args: &TaskCommentArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let text = require_body(read_text(&args.text, &args.text_file, "text")?)?;
    if let Some(to) = &args.notify {
        let msgs = client
            .send(&SendRequest {
                to: Some(to.clone()),
                body: text,
                task: Some(args.task.clone()),
                ..Default::default()
            })
            .await?;
        if cli.json {
            render::print_json(&msgs)?;
        } else {
            println!("commented on {}, notified {to}", args.task);
        }
        return Ok(());
    }
    let task = client.note_task(&args.task, &text).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("commented on {}", task.id);
    }
    Ok(())
}

pub(super) async fn task_search(cli: &Cli, args: &TaskSearchArgs) -> Result<(), CliError> {
    if args.words.is_empty() {
        return Err(CliError::from(anyhow::anyhow!(
            "at least one search word is required"
        )));
    }
    let client = client_for_read(cli).await?;
    let word_refs: Vec<&str> = args.words.iter().map(|w| w.as_str()).collect();
    let filtered_tasks = client.search_tasks(&word_refs).await?;
    if cli.json {
        render::print_json(&filtered_tasks)?;
    } else if filtered_tasks.is_empty() {
        println!("no matching tasks");
    } else {
        println!(
            "{:<10} {:<9} {:<8} {:<4} TITLE",
            "ID", "KIND", "STATE", "SIZE"
        );
        for t in &filtered_tasks {
            print_task_row(t);
        }
    }
    Ok(())
}

pub(super) fn edge_kind_arg(k: EdgeKindArg) -> EdgeKind {
    match k {
        EdgeKindArg::Blocks => EdgeKind::Blocks,
        EdgeKindArg::Parent => EdgeKind::Parent,
        EdgeKindArg::DiscoveredFrom => EdgeKind::DiscoveredFrom,
        EdgeKindArg::Related => EdgeKind::Related,
        EdgeKindArg::Supersedes => EdgeKind::Supersedes,
        EdgeKindArg::Duplicates => EdgeKind::Duplicates,
    }
}

/// `--blocked-by <other>` is sugar for `--kind blocks --to <task>` with
/// `from`/`to` swapped (roles-and-lifecycle.md's `bridle dep add tw-7fa2
/// --blocked-by tw-c0f1` reads as "`tw-7fa2` is blocked by `tw-c0f1`", i.e.
/// the edge points from the blocker to the blocked task). clap's
/// `conflicts_with_all` already rules out combining it with `--to`/`--kind`.
pub(super) fn resolve_edge_args(
    args: &DepEdgeArgs,
) -> Result<(String, String, EdgeKind), CliError> {
    match (&args.blocked_by, &args.to) {
        (Some(blocker), None) => Ok((blocker.clone(), args.task.clone(), EdgeKind::Blocks)),
        (None, Some(to)) => Ok((args.task.clone(), to.clone(), edge_kind_arg(args.kind))),
        (None, None) => Err(CliError::from(anyhow::anyhow!(
            "specify --to <task> or --blocked-by <task>"
        ))),
        (Some(_), Some(_)) => unreachable!("clap's conflicts_with_all rules this out"),
    }
}

pub(super) fn print_edge_row(e: &Edge) {
    println!("{:<10} {:<16} {}", e.from, e.kind, e.to);
}

pub(super) async fn conflict(cli: &Cli, args: &ConflictArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    match &args.action {
        ConflictAction::List => {
            let mut list = client.list_conflicts().await?;
            list.sort_by_key(|c| c.state != "open");
            if cli.json {
                render::print_json(&list)?;
            } else if list.is_empty() {
                println!("no conflicts");
            }
            if !cli.json {
                for c in &list {
                    println!(
                        "{:<6}{:<9}{:<10}{}  {} {}{}",
                        c.id,
                        c.state,
                        c.kind,
                        c.key,
                        c.tasks[0],
                        c.tasks[1],
                        c.resolution
                            .as_deref()
                            .map(|r| format!("  ({r})"))
                            .unwrap_or_default()
                    );
                }
            }
        }
        ConflictAction::Resolve(a) => {
            let order = match a.order.as_deref() {
                Some([x, y]) => Some([x.clone(), y.clone()]),
                Some(_) => {
                    return Err(CliError::Other(anyhow::anyhow!("--order takes A,B")));
                }
                None => None,
            };
            let c = client
                .resolve_conflict(
                    &a.id,
                    &ResolveConflictRequest {
                        compatible: a.compatible.clone(),
                        order,
                        merge_into: a.merge_into.clone(),
                    },
                )
                .await?;
            if cli.json {
                render::print_json(&c)?;
            } else {
                println!("{} resolved: {}", c.id, c.resolution.unwrap_or_default());
            }
        }
    }
    Ok(())
}

pub(super) async fn dep(cli: &Cli, args: &DepArgs) -> Result<(), CliError> {
    match &args.action {
        DepAction::Add(a) => dep_add(cli, a).await,
        DepAction::Rm(a) => dep_rm(cli, a).await,
    }
}

pub(super) async fn dep_add(cli: &Cli, args: &DepEdgeArgs) -> Result<(), CliError> {
    let (from, to, kind) = resolve_edge_args(args)?;
    let client = client_for(cli).await?;
    let edge = client.add_edge(&NewEdgeRequest { from, to, kind }).await?;
    if cli.json {
        render::print_json(&edge)?;
    } else {
        print_edge_row(&edge);
    }
    Ok(())
}

pub(super) async fn dep_rm(cli: &Cli, args: &DepEdgeArgs) -> Result<(), CliError> {
    let (from, to, kind) = resolve_edge_args(args)?;
    let client = client_for(cli).await?;
    client
        .remove_edge(&RemoveEdgeQuery { from, to, kind })
        .await?;
    Ok(())
}

pub(super) fn print_task_row_with_project(project: Option<&str>, t: &Task) {
    match project {
        Some(p) => println!(
            "{:<16} {:<10} {:<9} {:<8} {:<4} {}",
            p,
            t.id,
            t.kind,
            t.state,
            size_str(t.size),
            t.title
        ),
        None => print_task_row(t),
    }
}

pub(super) async fn ready(cli: &Cli, args: &ReadyArgs) -> Result<(), CliError> {
    // `--role` has nothing to filter on yet: tasks don't carry a role field
    // (P0-1 gap, docs/design/cli.md). Accepted, not rejected, so a caller
    // scripting ahead of that field landing doesn't need to special-case it.
    let _ = &args.role;

    if let Some(id) = &args.task {
        return task_ready(cli, id).await;
    }

    if !args.all {
        let client = client_for_read(cli).await?;
        let tasks = client.top_tier_ready_tasks().await?;
        if cli.json {
            render::print_json(&tasks)?;
        } else if tasks.is_empty() {
            println!("no ready tasks");
        } else {
            for t in &tasks {
                print_task_row_with_project(None, t);
            }
        }
        return Ok(());
    }

    let daemons = discovery::list_registry();
    let mut rows: Vec<(String, Task)> = Vec::new();
    for info in &daemons {
        let token = discovery::resolve_token(
            cli.token.as_deref(),
            Some(std::path::Path::new(&info.workspace)),
            Some(&info.project),
            None,
            &ProcessEnv,
            true,
        )
        .ok()
        .flatten();
        let client =
            Client::new_with_timeout(info.url.clone(), token, std::time::Duration::from_secs(5));
        if let Ok(tasks) = client.top_tier_ready_tasks().await {
            rows.extend(tasks.into_iter().map(|t| (info.project.clone(), t)));
        }
    }
    if cli.json {
        render::print_json(&rows)?;
    } else if rows.is_empty() {
        println!("no ready tasks");
    } else {
        for (project, t) in &rows {
            print_task_row_with_project(Some(project), t);
        }
    }
    Ok(())
}

pub(super) async fn queue(cli: &Cli, args: &QueueArgs) -> Result<(), CliError> {
    match &args.action {
        None => queue_show(cli).await,
        Some(QueueAction::Set(a)) => queue_set(cli, a).await,
        Some(QueueAction::AddTier(a)) => queue_add_tier(cli, a).await,
    }
}

/// Splits `--tier tw-1,tw-2 --tier tw-3` into the ordered `Vec<Vec<String>>`
/// `set_queue`/`add_queue_tier` want.
pub(super) fn split_tier(s: &str) -> Vec<String> {
    s.split(',')
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

pub(super) async fn queue_set(cli: &Cli, args: &QueueSetArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let tiers: Vec<Vec<String>> = args.tiers.iter().map(|t| split_tier(t)).collect();
    let q = client.set_queue(tiers).await?;
    if cli.json {
        render::print_json(&q)?;
    } else {
        println!("queue set: {} tier(s)", q.tiers.len());
    }
    Ok(())
}

pub(super) async fn queue_add_tier(cli: &Cli, args: &QueueAddTierArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let q = client.add_queue_tier(args.tasks.clone()).await?;
    if cli.json {
        render::print_json(&q)?;
    } else {
        println!("queue set: {} tier(s)", q.tiers.len());
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub(super) struct QueueTaskRow {
    id: String,
    title: String,
    kind: TaskKind,
    state: bridle_api::TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<TaskSize>,
    /// Ready (deps met, no open question) and, since ready already implies
    /// `planned`, therefore unclaimed too (roles-and-lifecycle.md, "the
    /// queue").
    startable: bool,
    /// Still settling (ny9u): when it becomes startable; null once settled.
    settle_until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(serde::Serialize)]
pub(super) struct QueueTierRow {
    rank: usize,
    tasks: Vec<QueueTaskRow>,
}

#[derive(serde::Serialize)]
pub(super) struct QueueView {
    claimed: Vec<Task>,
    tiers: Vec<QueueTierRow>,
}

/// `bridle queue`'s read-only view: claimed tasks with their worker, then
/// the tiers in rank order, each task marked startable or blocked
/// (roles-and-lifecycle.md, "the queue"). Composed client-side from three
/// plain reads (queue, all tasks, ready tasks) rather than a bespoke server
/// endpoint, since none of the three needs PM/human-only write access.
pub(super) async fn queue_show(cli: &Cli) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let q = client.get_queue().await?;
    let all_tasks = client.list_tasks().await?;
    let ready_ids: std::collections::HashSet<String> = client
        .ready_tasks()
        .await?
        .into_iter()
        .map(|t| t.id)
        .collect();
    let by_id: std::collections::HashMap<&str, &Task> =
        all_tasks.iter().map(|t| (t.id.as_str(), t)).collect();

    let claimed: Vec<Task> = all_tasks
        .iter()
        .filter(|t| t.claimed_by.is_some())
        .cloned()
        .collect();
    let tiers: Vec<QueueTierRow> = q
        .tiers
        .iter()
        .enumerate()
        .map(|(i, tier)| QueueTierRow {
            rank: i + 1,
            tasks: tier
                .iter()
                .filter_map(|id| by_id.get(id.as_str()).copied())
                .map(|t| QueueTaskRow {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    kind: t.kind,
                    state: t.state,
                    size: t.size,
                    startable: ready_ids.contains(&t.id),
                    settle_until: t.settle_until,
                })
                .collect(),
        })
        .collect();

    if cli.json {
        render::print_json(&QueueView { claimed, tiers })?;
        return Ok(());
    }

    if claimed.is_empty() {
        println!("no claimed tasks");
    } else {
        println!("Claimed:");
        for t in &claimed {
            println!(
                "  {:<10} {:<9} {:<8} {:<4} {:<16} {}",
                t.id,
                t.kind,
                t.state,
                size_str(t.size),
                t.claimed_by.as_deref().unwrap_or(""),
                t.title
            );
        }
    }
    if tiers.is_empty() {
        println!("no queue set");
        return Ok(());
    }
    for tier in &tiers {
        println!();
        println!("Tier {}:", tier.rank);
        for t in &tier.tasks {
            let mark = match t.settle_until {
                Some(until) => format!("settling until {}", settle_clock_text(until)),
                None if t.startable => "startable".to_string(),
                None => "blocked".to_string(),
            };
            println!(
                "  {:<10} {:<9} {:<8} {:<4} {:<9} {}",
                t.id,
                t.kind,
                t.state,
                size_str(t.size),
                mark,
                t.title
            );
        }
    }
    Ok(())
}
