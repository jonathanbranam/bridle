//! Ticket commands: task, claim, release, ready, queue, dep, land, conflict, impact, ask, answer.

use super::*;

pub(crate) async fn ask(cli: &Cli, args: &AskArgs) -> Result<(), CliError> {
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

pub(crate) async fn answer(cli: &Cli, args: &AnswerArgs) -> Result<(), CliError> {
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

pub(super) fn task_kind_arg(k: TicketKindArg) -> TicketKind {
    match k {
        TicketKindArg::Feature => TicketKind::Feature,
        TicketKindArg::Bug => TicketKind::Bug,
        TicketKindArg::Fix => TicketKind::Fix,
        TicketKindArg::Epic => TicketKind::Epic,
        TicketKindArg::Docs => TicketKind::Docs,
        TicketKindArg::Chore => TicketKind::Chore,
        TicketKindArg::Question => TicketKind::Question,
        TicketKindArg::Research => TicketKind::Research,
        TicketKindArg::Explore => TicketKind::Explore,
        TicketKindArg::ArchRevision => TicketKind::ArchRevision,
        TicketKindArg::ReEvaluate => TicketKind::ReEvaluate,
        TicketKindArg::Incident => TicketKind::Incident,
    }
}

pub(super) async fn task(cli: &Cli, args: &RowArgs) -> Result<(), CliError> {
    match &args.action {
        RowAction::New(a) => task_new(cli, a).await,
        RowAction::Show(a) => task_show(cli, a).await,
        RowAction::Edit(a) => task_edit(cli, a).await,
        RowAction::List(a) => task_list(cli, a).await,
        RowAction::Plan(a) => task_plan(cli, a).await,
        RowAction::Drop(a) => task_drop(cli, a).await,
        RowAction::Priority(a) => task_priority(cli, a).await,
        RowAction::Kind(a) => task_kind(cli, a).await,
        RowAction::Done(a) => task_done(cli, a).await,
        RowAction::Summary(a) => task_summary(cli, a).await,
        RowAction::Reopen(a) => task_reopen(cli, a).await,
        RowAction::Hold(a) => {
            let req = HoldTicketRequest {
                reason: a.reason.clone(),
                until: a.until.clone(),
            };
            let client = client_for(cli).await?;
            print_changed(cli, client.hold_task(&a.task, &req).await?)
        }
        RowAction::Unhold(a) => {
            let client = client_for(cli).await?;
            print_changed(cli, client.unhold_task(&a.task).await?)
        }
        RowAction::Built(a) => {
            let req = BuiltTicketRequest {
                branch: a.branch.clone(),
                commit: a.commit.clone(),
            };
            let client = client_for(cli).await?;
            print_changed(cli, client.built_task(&a.task, &req).await?)
        }
        RowAction::SendBack(a) => {
            let req = SendBackTicketRequest {
                reason: a.reason.clone(),
            };
            let client = client_for(cli).await?;
            print_changed(cli, client.send_back_task(&a.task, &req).await?)
        }
        RowAction::SkipSettle(a) => task_skip_settle(cli, a).await,
        RowAction::Watch(a) => task_watch(cli, a, true).await,
        RowAction::Unwatch(a) => task_watch(cli, a, false).await,
        RowAction::Comment(a) => task_comment(cli, a).await,
        RowAction::Search(a) => task_search(cli, a).await,
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

pub(super) fn size_str(size: Option<TicketSize>) -> &'static str {
    size.map_or("-", TicketSize::as_str)
}

pub(super) fn task_size_arg_to_opt(arg: TicketSizeArg) -> Option<TicketSize> {
    match arg {
        TicketSizeArg::S => Some(TicketSize::S),
        TicketSizeArg::M => Some(TicketSize::M),
        TicketSizeArg::L => Some(TicketSize::L),
        TicketSizeArg::None => Some(TicketSize::None),
    }
}

use bridle_api::settle_clock_text;
use bridle_api::types::{
    BuiltTicketRequest, HoldTicketRequest, RemoteWatchRequest, SendBackTicketRequest,
    sort_by_priority,
};

pub fn print_task_row(t: &Ticket) {
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

/// The one inbox note a human to-do sends (coordination.md, "Human to-dos").
pub async fn notify_human_todo(cli: &Cli, task: &Ticket) -> Result<(), CliError> {
    client_for(cli)
        .await?
        .send(&SendRequest {
            to: Some("human".to_string()),
            body: format!(
                "To-do for you ({} priority): {}. Finish it with `bridle ticket done {}`.",
                task.priority, task.title, task.id
            ),
            kind: MessageKind::Note,
            when: bridle_api::When::Now,
            reply_to: None,
            task: Some(task.id.clone()),
        })
        .await?;
    Ok(())
}

pub(super) async fn task_new(cli: &Cli, args: &RowNewArgs) -> Result<(), CliError> {
    if args.for_human && args.from.is_none() {
        // The hidden alias makes a ticket too, so no new ticketless to-do appears (cy4q).
        let repo = crate::ticket::repo_root()?;
        let ticket_args = crate::cli::TicketNewArgs {
            title: Some(args.title.clone()),
            from_task: None,
            repos: Vec::new(),
            blocked_by: Vec::new(),
            related: Vec::new(),
            theme: None,
            kind: Some(args.kind),
            body: args.body.clone(),
            body_file: args.body_file.clone(),
            for_human: true,
            priority: args.priority,
            version: 1,
            stages: Vec::new(),
            review: Vec::new(),
            claim: false,
        };
        return crate::ticket::new(cli, &repo, &ticket_args).await;
    }
    let client = client_for(cli).await?;
    let body = if args.body.is_some() || args.body_file.is_some() {
        read_text(&args.body, &args.body_file, "body")?
    } else {
        String::new()
    };
    let req = NewTicketRequest {
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
        notify_human_todo(cli, &task).await?;
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

pub(super) async fn task_show(cli: &Cli, args: &RowShowArgs) -> Result<(), CliError> {
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

pub(super) async fn task_edit(cli: &Cli, args: &RowEditArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = if args.body.is_some() || args.body_file.is_some() {
        Some(read_text(&args.body, &args.body_file, "body")?)
    } else {
        None
    };
    let req = EditTicketRequest {
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

pub(super) async fn task_list(cli: &Cli, args: &RowListArgs) -> Result<(), CliError> {
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
    // A ticket's row is readied through the ticket, which checks the ask and the commit (k7tm).
    if let Ok(t) = client.get_task(id).await
        && let Some(ticket) = &t.ticket
    {
        return Err(anyhow::anyhow!(
            "task {id} belongs to ticket {ticket}: use `bridle ticket ready {ticket}`"
        )
        .into());
    }
    let task = client.ready_task(id).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_plan(cli: &Cli, args: &RowPlanArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.plan_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub fn task_priority_arg(arg: TicketPriorityArg) -> TicketPriority {
    match arg {
        TicketPriorityArg::Critical => TicketPriority::Critical,
        TicketPriorityArg::Urgent => TicketPriority::Urgent,
        TicketPriorityArg::High => TicketPriority::High,
        TicketPriorityArg::Normal => TicketPriority::Normal,
        TicketPriorityArg::Low => TicketPriority::Low,
    }
}

pub(super) async fn task_priority(cli: &Cli, args: &RowPriorityArgs) -> Result<(), CliError> {
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

pub(super) async fn task_kind(cli: &Cli, args: &RowKindArgs) -> Result<(), CliError> {
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

pub(super) async fn task_drop(cli: &Cli, args: &RowDropArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let req = DropTicketRequest {
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

pub(super) async fn push(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let r = client.push().await?;
    if cli.json {
        render::print_json(&r)?;
    } else {
        println!("pushed {} to origin", r.branch);
    }
    Ok(())
}

pub(super) async fn task_done(cli: &Cli, args: &RowDoneArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let original_task = client.get_task(&args.task).await.ok();
    let was_human_claimed = original_task
        .as_ref()
        .and_then(|t| t.claimed_by.as_deref())
        .map(|c| c == "human")
        .unwrap_or(false);
    let req = DoneTicketRequest {
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

pub(super) async fn task_summary(cli: &Cli, args: &RowSummaryArgs) -> Result<(), CliError> {
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

fn print_changed(cli: &Cli, task: Ticket) -> Result<(), CliError> {
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_reopen(cli: &Cli, args: &RowReopenArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.reopen_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

pub(super) async fn task_skip_settle(cli: &Cli, args: &RowSkipSettleArgs) -> Result<(), CliError> {
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
    args: &RowWatchArgs,
    watch: bool,
) -> Result<(), CliError> {
    // A task on another project's daemon (no direct token for it): our own daemon registers the
    // watch there through its peer token, and the updates come back as messages.
    let remote = super::misc::own_daemon_for_other_project(cli).await?;
    let client = match &remote {
        Some((own, _)) => own.clone(),
        None => client_for(cli).await?,
    };
    let task = if let Some((_, project)) = remote {
        client
            .remote_watch(&RemoteWatchRequest {
                project,
                task: args.task.clone(),
                watch,
            })
            .await?
    } else if watch {
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

pub(crate) async fn task_comment(cli: &Cli, args: &RowCommentArgs) -> Result<(), CliError> {
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
    let task = client
        .note_task_for(&args.task, &text, args.for_human)
        .await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("commented on {}", task.id);
    }
    Ok(())
}

pub(super) async fn task_search(cli: &Cli, args: &RowSearchArgs) -> Result<(), CliError> {
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

pub(super) fn print_task_row_with_project(project: Option<&str>, t: &Ticket) {
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
    let mut rows: Vec<(String, Ticket)> = Vec::new();
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
pub(super) struct QueueRow {
    id: String,
    title: String,
    kind: TicketKind,
    state: bridle_api::TicketState,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<TicketSize>,
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
    tasks: Vec<QueueRow>,
}

#[derive(serde::Serialize)]
pub(super) struct QueueView {
    claimed: Vec<Ticket>,
    tiers: Vec<QueueTierRow>,
}

/// The label after a queued task: `held` and `built` say so (never startable), an unmet
/// dependency is `blocked`.
fn queue_mark(t: &QueueRow) -> String {
    use bridle_api::TicketState;
    match (t.state, t.settle_until) {
        (TicketState::Held, _) => "held".to_string(),
        (TicketState::Built, _) => "built".to_string(),
        (_, Some(until)) => format!("settling until {}", settle_clock_text(until)),
        _ if t.startable => "startable".to_string(),
        _ => "blocked".to_string(),
    }
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
    let by_id: std::collections::HashMap<&str, &Ticket> =
        all_tasks.iter().map(|t| (t.id.as_str(), t)).collect();

    let claimed: Vec<Ticket> = all_tasks
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
                .map(|t| QueueRow {
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
            let mark = queue_mark(t);
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

#[cfg(test)]
mod queue_mark_tests {
    use super::*;
    use bridle_api::TicketState;

    fn row(state: TicketState, startable: bool) -> QueueRow {
        QueueRow {
            id: "t-1".into(),
            title: "x".into(),
            kind: TicketKind::Chore,
            state,
            size: None,
            startable,
            settle_until: None,
        }
    }

    #[test]
    fn held_and_built_are_labelled_and_never_startable() {
        assert_eq!(queue_mark(&row(TicketState::Held, true)), "held");
        assert_eq!(queue_mark(&row(TicketState::Built, true)), "built");
        assert_eq!(queue_mark(&row(TicketState::Planned, true)), "startable");
        assert_eq!(queue_mark(&row(TicketState::Planned, false)), "blocked");
    }
}
