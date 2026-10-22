//! Dispatch and implementation for every subcommand except `serve` (see
//! `serve.rs`). See docs/design/cli.md.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use bridle_api::discovery::{self, Env, ProcessEnv};
use bridle_api::{
    AllocPortRequest, BudgetHoldRequest, BudgetOverrideRequest, Client, DoneTicketRequest,
    DropTicketRequest, EditTicketRequest, Event, EventQuery, ExternalUsage, Impact,
    ImpactCheckRequest, InterruptRequest, LandRequest, MaxWorkersRequest, MessageKind,
    MessageQuery, NewTicketRequest, OverlapLevel, ProbeOutcome, ProbeRequest, ProbeResult,
    RemoveQuery, RenewRequest, ResolveConflictRequest, ResumeRequest, SendRequest,
    SetImpactRequest, SetKindRequest, SetPriorityRequest, SetSummaryRequest, SpawnRequest, SpecRef,
    StopRequest, Ticket, TicketKind, TicketPriority, TicketSize, TokenCreateRequest,
    UsageBreakdownQuery, UsageBucket, UsageGroupBy, UsageGroupsQuery, UsageHistoryQuery,
    UsageInstancesQuery, UsagePeriodsQuery, UsageSeriesQuery, UsageShareGroup, Workdir, event_kind,
};
use chrono::{Local, TimeZone, Utc};
use clap::CommandFactory;
use futures::StreamExt;

use crate::cli::{AgentArgs, DaemonArgs, HookArgs, TicketAction, TicketArgs, WorkflowAction};
use crate::cli::{
    AgentsArgs, AnswerArgs, ArchProposeArgs, AskArgs, BudgetAction, BudgetArgs, BudgetHoldArgs,
    ClaimArgs, Cli, Command, CompletionsArgs, ConflictAction, ConflictArgs, CostAction, CostArgs,
    CostAuditArgs, EventsArgs, HandoverAction, HandoverArgs, ImpactAction, ImpactArgs, InboxAction,
    InboxArgs, InboxReadArgs, InboxShowArgs, InterruptArgs, LogsArgs, MessageAction, MessageArgs,
    MessagesArgs, MigrateArgs, PaneAction, PrimeArgs, PrimeRoleArg, ProbeArgs, QueueAction,
    QueueAddTierArgs, QueueArgs, QueueSetArgs, ReadyArgs, ReleaseArgs, RmArgs, RowAction, RowArgs,
    RowCommentArgs, RowDoneArgs, RowDropArgs, RowEditArgs, RowKindArgs, RowListArgs, RowNewArgs,
    RowPlanArgs, RowPriorityArgs, RowReopenArgs, RowSearchArgs, RowShowArgs, RowSkipSettleArgs,
    RowSummaryArgs, RowWatchArgs, RulesAction, RulesArgs, RulesDiffArgs, RulesExplainArgs,
    SendArgs, ShowArgs, SpawnArgs, SpecAction, SpecArgs, SpecExportArgs, SpecFormatArg, StopArgs,
    TicketKindArg, TicketPriorityArg, TicketSizeArg, TokenAction, TokenArgs, UsageArgs,
    UsageBucketArg, UsageByArg, WaitArgs, WakeArgs, WhenArg,
};
use crate::cli::{
    FocusAction, FocusArgs, LandArgs, OrchestratorAction, OrchestratorArgs, PortAction, PortArgs,
};
use crate::error::CliError;
use crate::render;
use crate::serve;

mod agent;
pub(crate) mod daemon;
mod docs;
pub(crate) mod fanout;
mod hook;
mod misc;
mod orchestrator;
mod report;
mod schedule;
mod task;
mod usage;
mod workflow;
pub(crate) use agent::list_sessions;
use agent::*;
use daemon::*;
use hook::*;
pub use misc::read_text;
use misc::*;
use orchestrator::*;
use task::*;
pub(crate) use task::{answer, ask, task_comment};
pub use task::{notify_human_todo, print_task_row, task_priority_arg};
use usage::*;
pub(crate) use workflow::spec_inputs;
use workflow::*;

/// Rewrite a grouped command line (`task claim`, `agent spawn`, ...) into the flat
/// command it forwards to, so the old top-level names and the new groups share one
/// dispatch and can't drift apart.
pub(super) fn normalize(cmd: Command) -> Command {
    use crate::cli::{AgentAction as A, DaemonAction as D, HookAction as H, UsageSub};
    match cmd {
        Command::Daemon(DaemonArgs { action }) => match action {
            D::Serve(a) => Command::Serve(a),
            D::Stop => Command::StopDaemon,
            D::Restart(a) => Command::Restart(a),
            D::Doctor(a) => Command::Doctor(a),
            D::Init(a) => Command::Init(a),
            D::Launchd(a) => Command::Launchd(a),
            D::Systemd(a) => Command::Systemd(a),
            D::Rebuild(a) => Command::Rebuild(a),
            D::List => Command::Daemons,
        },
        Command::Agent(AgentArgs { action }) => match action {
            A::Spawn(a) => Command::Spawn(*a),
            A::List(a) => Command::Agents(a),
            A::Show(a) => Command::Show(a),
            A::Interrupt(a) => Command::Interrupt(a),
            A::Stop(a) => Command::Stop(a),
            A::Resume(a) => Command::Resume(a),
            A::Renew(a) => Command::Renew(a),
            A::Rm(a) => Command::Rm(a),
            A::Logs(a) => Command::Logs(a),
            A::Wake(a) => Command::Wake(a),
        },
        Command::Message(MessageArgs { action }) => match action {
            MessageAction::Send(a) => Command::Send(a),
            MessageAction::Inbox(a) => Command::Inbox(a),
            MessageAction::List(a) => Command::Messages(a),
            action => Command::Message(MessageArgs { action }),
        },
        Command::Hook(HookArgs { action }) => match action {
            H::Statusline => Command::Statusline,
            H::StopCheck => Command::StopCheck,
            H::ArchGuard => Command::ArchGuard,
            H::KillGuard => Command::KillGuard,
            H::LinkCheck => Command::LinkCheck,
        },
        Command::Usage(UsageArgs { sub: Some(sub), .. }) => match sub {
            UsageSub::Cost(a) => Command::Cost(a),
            UsageSub::Budget(a) => Command::Budget(a),
        },
        Command::Ticket(TicketArgs {
            action: TicketAction::Row(row),
        }) => normalize(Command::Task(RowArgs { action: row.into() })),
        Command::Task(RowArgs { action }) => match action {
            RowAction::Claim(a) => Command::Claim(a),
            RowAction::Release(a) => Command::Release(a),
            RowAction::Ready(a) => Command::Ready(a),
            RowAction::Queue(a) => Command::Queue(a),
            RowAction::Land(a) => Command::Land(a),
            RowAction::Conflict(a) => Command::Conflict(a),
            RowAction::Impact(a) => Command::Impact(a),
            RowAction::Ask(a) => Command::Ask(a),
            RowAction::Answer(a) => Command::Answer(a),
            action => Command::Task(RowArgs { action }),
        },
        Command::Orchestrator(OrchestratorArgs { action }) => match action {
            OrchestratorAction::Handover(a) => Command::Handover(a),
            OrchestratorAction::Prime(a) => Command::Prime(a),
            OrchestratorAction::WaitForWake(a) => Command::WaitForWake(a),
            action => Command::Orchestrator(OrchestratorArgs { action }),
        },
        Command::Workflow(action) => match action {
            WorkflowAction::Rules(a) => Command::Rules(a),
            WorkflowAction::Sync => Command::Sync,
            WorkflowAction::Spec(a) => Command::Spec(a),
            WorkflowAction::Goals(a) => Command::Goals(a),
            WorkflowAction::Arch(a) => Command::Arch(a),
            WorkflowAction::Explore(a) => Command::Explore(a),
            WorkflowAction::Trace(a) => Command::Trace(a),
            action => Command::Workflow(action),
        },
        other => other,
    }
}

/// The old top-level spelling of a command that moved into a group, with its new spelling
/// (a67t). Must run before `normalize`, which erases the difference. Not listed: `serve`
/// (launchd and systemd units spell it) and the hook commands, which scripts call.
fn deprecated_spelling(cmd: &Command) -> Option<(&'static str, &'static str)> {
    Some(match cmd {
        Command::StopDaemon => ("stop-daemon", "daemon stop"),
        Command::Restart(_) => ("restart", "daemon restart"),
        Command::Doctor(_) => ("doctor", "daemon doctor"),
        Command::Init(_) => ("init", "daemon init"),
        Command::Launchd(_) => ("launchd", "daemon launchd"),
        Command::Systemd(_) => ("systemd", "daemon systemd"),
        Command::Rebuild(_) => ("rebuild", "daemon rebuild"),
        Command::Daemons => ("daemons", "daemon list"),
        Command::Spawn(_) => ("spawn", "agent spawn"),
        Command::Agents(_) => ("agents", "agent list"),
        Command::Show(_) => ("show", "agent show"),
        Command::Interrupt(_) => ("interrupt", "agent interrupt"),
        Command::Stop(_) => ("stop", "agent stop"),
        Command::Resume(_) => ("resume", "agent resume"),
        Command::Renew(_) => ("renew", "agent renew"),
        Command::Rm(_) => ("rm", "agent rm"),
        Command::Logs(_) => ("logs", "agent logs"),
        Command::Wake(_) => ("wake", "agent wake"),
        Command::Messages(_) => ("messages", "message list"),
        _ => return None,
    })
}

pub async fn run(mut cli: Cli) -> Result<(), CliError> {
    if let Some((old, new)) = deprecated_spelling(&cli.command) {
        eprintln!("warning: `bridle {old}` is deprecated; use `bridle {new}`");
    }
    let command = normalize(std::mem::replace(&mut cli.command, Command::Status));
    let cli = Cli { command, ..cli };
    // The daemon logs its config warnings; every other command prints them once to stderr.
    if !matches!(cli.command, Command::Serve(_)) {
        bridle_api::config_warn::print_to_stderr();
    }
    match &cli.command {
        Command::Serve(args) => serve::run(&cli, args).await,
        Command::Docs(args) => docs::run(args),
        Command::Report(args) => report::report(&cli, args).await,
        Command::Link(args) => crate::link::run(&cli, args).await,
        Command::Dev(args) => crate::dev_site::run(&cli, args).await,
        Command::Up(args) => crate::up::run(&cli, args),
        Command::Gateway(args) => crate::gateway::run(&cli, args).await,
        Command::StopDaemon => stop_daemon(&cli).await,
        Command::Restart(args) => restart(&cli, args).await,
        Command::Doctor(args) => crate::doctor::run(&cli, args),
        Command::Init(args) => {
            crate::init::run(args)?;
            let repo = match &args.repo {
                Some(p) => p.clone(),
                None => std::env::current_dir().unwrap_or_default(),
            };
            if let Some(name) = repo
                .canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            {
                crate::project_setup::print_next(&cli, &repo, &name).await;
            }
            Ok(())
        }
        Command::Launchd(args) => crate::launchd::run(&cli, args),
        Command::Sign(args) => crate::sign::run(args),
        Command::Systemd(args) => crate::systemd::run(&cli, args),
        Command::Rebuild(args) => rebuild(&cli, args.from_origin).await,
        Command::Daemons => daemons(&cli).await,
        Command::Status => status(&cli).await,
        Command::Ci => ci(&cli).await,
        Command::Spawn(args) => spawn(&cli, args).await,
        Command::Agents(args) => agents(&cli, args).await,
        Command::Show(args) => show(&cli, args).await,
        Command::Send(args) => send(&cli, args).await,
        Command::Inbox(args) => inbox(&cli, args).await,
        Command::Message(args) => message(&cli, args).await,
        Command::Recipients => recipients(&cli).await,
        Command::Messages(args) => messages(&cli, args).await,
        Command::Interrupt(args) => interrupt(&cli, args).await,
        Command::Stop(args) => stop(&cli, args).await,
        Command::Resume(args) => resume(&cli, args).await,
        Command::Renew(args) => renew(&cli, args).await,
        Command::Rm(args) => rm(&cli, args).await,
        Command::Logs(args) => logs(&cli, args).await,
        Command::Wake(args) => wake(&cli, args).await,
        Command::Events(args) => events(&cli, args).await,
        Command::Wait(args) => wait(&cli, args).await,
        Command::Usage(args) => usage(&cli, args).await,
        Command::Cost(args) => cost(&cli, args).await,
        Command::Tui => tui(&cli).await,
        Command::Budget(args) => budget(&cli, args).await,
        Command::Token(args) => token(&cli, args).await,
        Command::Schedule(args) => schedule::run(&cli, args).await,
        Command::Task(args) => task(&cli, args).await,
        Command::Impact(args) => impact(&cli, args).await,
        Command::Probe(args) => probe(&cli, args).await,
        Command::Land(args) => land(&cli, args).await,
        Command::Push => push(&cli).await,
        Command::PushBranch(args) => crate::push_branch::run(args).map_err(Into::into),
        Command::Conflict(args) => conflict(&cli, args).await,
        Command::Port(args) => port(&cli, args).await,
        Command::Ask(args) => ask(&cli, args).await,
        Command::Answer(args) => answer(&cli, args).await,
        Command::Claim(args) => claim(&cli, args).await,
        Command::Release(args) => release(&cli, args).await,
        Command::Ready(args) => ready(&cli, args).await,
        Command::Queue(args) => queue(&cli, args).await,
        Command::Statusline => statusline(&cli).await,
        Command::StopCheck => stop_check(&cli).await,
        Command::ArchGuard => arch_guard(&cli).await,
        Command::KillGuard => kill_guard(),
        Command::LinkCheck => link_check(&cli).await,
        Command::Orchestrator(OrchestratorArgs {
            action: OrchestratorAction::NoteSession,
        }) => {
            let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(serde_json::Value::Null);
            crate::orchestrator::note_session(&input);
            Ok(())
        }
        Command::Orchestrator(OrchestratorArgs {
            action: OrchestratorAction::ClearRecovery,
        }) => crate::orchestrator::clear_recovery(),
        Command::Orchestrator(OrchestratorArgs {
            action: OrchestratorAction::Hold { reason },
        }) => crate::orchestrator::hold(&cli, reason.clone()).await,
        Command::Orchestrator(OrchestratorArgs {
            action: OrchestratorAction::Release,
        }) => crate::orchestrator::release(&cli).await,
        Command::Focus(FocusArgs {
            action: Some(FocusAction::Gate),
            ..
        }) => {
            crate::focus::run_gate();
            Ok(())
        }
        Command::Focus(FocusArgs {
            action: Some(FocusAction::Reply),
            ..
        }) => {
            crate::focus::run_reply();
            Ok(())
        }
        Command::Focus(FocusArgs {
            action: None,
            schedule,
        }) => crate::focus::show(*schedule, cli.json).map_err(CliError::Other),
        Command::Review(args) => crate::review::run(&cli, args).await,
        Command::Handover(args) => handover(&cli, args).await,
        Command::WaitForWake(args) => wait_for_wake(&cli, args.timeout).await,
        Command::Mail(args) => match &args.action {
            crate::cli::MailAction::Run => mail_run(&cli).await,
            crate::cli::MailAction::Install(a) => {
                crate::mail_install::install(&cli, a)?;
                if let Some(p) = &cli.project {
                    crate::project_setup::print_next_for(&cli, p, None).await;
                }
                Ok(())
            }
            crate::cli::MailAction::Uninstall => crate::mail_install::uninstall(&cli),
        },
        Command::Prime(args) => prime(&cli, args).await,
        Command::Rules(args) => rules(&cli, args).await,
        Command::Sync => sync(&cli).await,
        Command::Workflow(action) => crate::vendor::run(action),
        Command::Spec(SpecArgs {
            action: SpecAction::Export(args),
        }) => spec_export(&cli, args).await,
        Command::Spec(args) => spec(&cli, args),
        Command::Goals(args) => crate::goals::run(&cli, args).await,
        Command::Ticket(args) => crate::ticket::run(&cli, args).await,
        Command::Arch(args) => arch(&cli, args).await,
        Command::Trace(args) => crate::trace::run(&cli, args),
        Command::Explore(args) => explore(&args.action),
        Command::Pane(args) => pane(&args.action),
        Command::Machine(args) => crate::tools_only::run(&args.action),
        Command::Projects(args) => match &args.action {
            crate::cli::ProjectsAction::TakeOver(a) => crate::take_over::run(&cli, a).await,
            crate::cli::ProjectsAction::Add(a) => {
                crate::project_sync::add(a)?;
                crate::project_setup::print_next_for(&cli, &a.name, None).await;
                Ok(())
            }
            crate::cli::ProjectsAction::Stop(a) => crate::project_stop::run(&cli, a).await,
            crate::cli::ProjectsAction::Start(a) => {
                crate::project_start::run(&cli, a).await?;
                if let (None, Some(p)) = (&a.machine, &cli.project) {
                    crate::project_setup::print_next_for(&cli, p, a.repo.as_deref()).await;
                }
                Ok(())
            }
            crate::cli::ProjectsAction::Preflight(a) => crate::preflight::run(&cli, a),
            crate::cli::ProjectsAction::Drain(a) => crate::project_drain::run(&cli, a).await,
            crate::cli::ProjectsAction::Setup(a) => crate::project_setup::run(&cli, a).await,
            action => crate::project_sync::run(action),
        },
        Command::Advisor(args) => crate::advisor::run(&cli, &args.action).await,
        Command::Session(args) => crate::session::run(&cli, &args.role).await,
        Command::Completions(args) => completions(args),
        Command::Migrate(args) => migrate(&cli, args).await,
        Command::Orchestrator(_) | Command::Daemon(_) | Command::Agent(_) | Command::Hook(_) => {
            unreachable!("normalize forwards the grouped forms")
        }
    }
}

/// Resolve the daemon endpoint and a token, per docs/design/agent-host/daemon.md and
/// principals.md. `resolve_endpoint` failing to find any daemon at all is exactly the
/// "daemon unreachable" case (exit 3). `allow_anonymous_read` mirrors the daemon's own
/// tolerance for token-less GET/HEAD requests (server.rs's `auth_middleware`): pass it
/// for a read-only command so a caller running inside Claude Code with no
/// `$BRIDLE_TOKEN` gets a working request instead of a client-side error ahead of one
/// that would have succeeded anyway.
async fn resolve_endpoint_and_token(
    cli: &Cli,
    allow_anonymous_read: bool,
) -> Result<(String, Option<String>), CliError> {
    let cwd = std::env::current_dir().context("current directory")?;
    let env = ProcessEnv;
    let endpoint =
        discovery::resolve_endpoint(cli.url.as_deref(), cli.project.as_deref(), &cwd, &env)
            .map_err(|e| CliError::Unreachable(e.to_string()))?;
    let token = discovery::resolve_token(
        cli.token.as_deref(),
        endpoint.workspace.as_deref(),
        endpoint.project.as_deref(),
        endpoint.machine.as_deref(),
        &env,
        allow_anonymous_read,
    )?;
    Ok((endpoint.url, token))
}

/// A client for a command that only ever writes (or both reads and writes):
/// keeps today's client-side error when `$CLAUDECODE` is set with no token.
pub async fn client_for(cli: &Cli) -> Result<Client, CliError> {
    let (url, token) = resolve_endpoint_and_token(cli, false).await?;
    Ok(Client::new(url, token).with_advisor(std::env::var("BRIDLE_ADVISOR_NAME").ok()))
}

/// A client for a read-only command: proceeds with no token when `$CLAUDECODE`
/// is set and no token was given, matching the daemon's tolerance for
/// token-less GET/HEAD requests.
pub(crate) async fn client_for_read(cli: &Cli) -> Result<Client, CliError> {
    let (url, token) = resolve_endpoint_and_token(cli, true).await?;
    Ok(Client::new(url, token).with_advisor(std::env::var("BRIDLE_ADVISOR_NAME").ok()))
}

/// `1h23m`-style rendering for a duration in whole seconds, for the `usage`
/// command's busy/wall columns.
fn format_duration_secs(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m{:02}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h{:02}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d{:02}h", secs / 86400, (secs % 86400) / 3600)
    }
}

fn format_age(age: chrono::Duration) -> String {
    let secs = age.num_seconds().max(0);
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// `bridle migrate`: resolve the project(s), apply, then (best effort) tell each project's daemon.
async fn migrate(cli: &Cli, args: &MigrateArgs) -> Result<(), CliError> {
    use crate::migrate::{MIGRATIONS, apply};

    struct Target {
        name: String,
        repo: PathBuf,
        client: Option<Client>,
    }
    let registry = discovery::list_registry();
    let targets: Vec<Target> = if args.all {
        registry
            .iter()
            .map(|d| {
                let token = discovery::resolve_token(
                    None,
                    Some(Path::new(&d.workspace)),
                    Some(&d.project),
                    None,
                    &ProcessEnv,
                    false,
                )
                .ok()
                .flatten();
                Target {
                    name: d.project.clone(),
                    repo: PathBuf::from(&d.repo),
                    client: Some(Client::new(d.url.clone(), token)),
                }
            })
            .collect()
    } else if let Some(name) = &cli.project {
        let d = registry
            .iter()
            .find(|d| &d.project == name)
            .ok_or_else(|| anyhow::anyhow!("no running daemon registered for project {name}"))?;
        vec![Target {
            name: name.clone(),
            repo: PathBuf::from(&d.repo),
            client: client_for(cli).await.ok(),
        }]
    } else {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("running git")?;
        if !out.status.success() {
            return Err(anyhow::anyhow!("not in a git repository").into());
        }
        let repo = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        vec![Target {
            name: repo
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            repo,
            client: client_for(cli).await.ok(),
        }]
    };
    if targets.is_empty() {
        println!("no daemons running: nothing to migrate");
        return Ok(());
    }

    let mut finished = vec![];
    for t in &targets {
        let out = apply(
            &t.repo,
            MIGRATIONS,
            args.dry_run,
            args.only.as_deref(),
            Utc::now(),
        )
        .with_context(|| format!("project {}", t.name))?;
        // One commit per run (br-6gdp), as at start-up. An `--only` run is an opt-in migration
        // whose diff the owner reviews and commits, so it is left alone.
        if !args.dry_run && args.only.is_none() {
            let (repo, done) = (t.repo.clone(), out.done.clone());
            let todo =
                tokio::task::spawn_blocking(move || crate::migrate::commit_leftovers(&repo, &done))
                    .await
                    .map_err(anyhow::Error::from)?;
            if let Some(todo) = todo {
                eprintln!("{}: {todo}", t.name);
            }
        }
        // A daemon being down never fails a migration; the log in the project is the record.
        if !args.dry_run
            && let Some(client) = &t.client
        {
            for rec in &out.done {
                if let Err(e) = client.record_migration(rec).await {
                    tracing::warn!("{}: couldn't record {} as an event: {e}", t.name, rec.id);
                }
            }
        }
        if cli.json {
            render::print_json(&serde_json::json!({
                "project": t.name,
                "dry_run": args.dry_run,
                "applied": out.done,
                "manual_pending": out.manual_pending,
                "failed": out.failed.as_ref().map(|(id, e)| serde_json::json!({"id": id, "error": format!("{e:#}")})),
            }))?;
        } else {
            let verb = if args.dry_run {
                "would apply"
            } else {
                "applied"
            };
            if out.done.is_empty() && out.failed.is_none() && out.manual_pending.is_empty() {
                println!("{}: up to date", t.name);
            }
            for id in &out.manual_pending {
                println!(
                    "{}: {id} is pending-manual: run it with `bridle migrate --only {id}`",
                    t.name
                );
            }
            for r in &out.done {
                println!("{}: {verb} {}: {}", t.name, r.id, r.summary);
                for f in &r.files {
                    println!("    {f}");
                }
            }
        }
        if let Some((id, e)) = out.failed {
            let done = if finished.is_empty() {
                String::new()
            } else {
                format!(" (done before it: {})", finished.join(", "))
            };
            return Err(anyhow::anyhow!("{}: {id} failed: {e:#}{done}", t.name).into());
        }
        finished.push(t.name.clone());
    }
    Ok(())
}

fn completions(args: &CompletionsArgs) -> Result<(), CliError> {
    use clap_complete::generate;
    let mut cmd = Cli::command();
    let mut stdout = std::io::stdout();
    generate(args.shell, &mut cmd, "bridle", &mut stdout);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Command {
        let mut argv = vec!["bridle"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).expect("parses").command
    }

    /// Each old spelling still runs the same command as the new one and warns, naming the new
    /// spelling; the new spelling does not warn (a67t).
    #[test]
    fn old_spellings_warn_and_forward_to_the_group() {
        let pairs: &[(&[&str], &[&str])] = &[
            (&["stop-daemon"], &["daemon", "stop"]),
            (&["restart"], &["daemon", "restart"]),
            (&["doctor"], &["daemon", "doctor"]),
            (&["init"], &["daemon", "init"]),
            (&["launchd", "install"], &["daemon", "launchd", "install"]),
            (&["systemd", "install"], &["daemon", "systemd", "install"]),
            (&["rebuild"], &["daemon", "rebuild"]),
            (&["daemons"], &["daemon", "list"]),
            (&["spawn", "worker"], &["agent", "spawn", "worker"]),
            (&["agents"], &["agent", "list"]),
            (&["show", "w1"], &["agent", "show", "w1"]),
            (&["interrupt", "w1"], &["agent", "interrupt", "w1"]),
            (&["stop", "w1"], &["agent", "stop", "w1"]),
            (&["resume", "w1"], &["agent", "resume", "w1"]),
            (&["renew", "w1"], &["agent", "renew", "w1"]),
            (&["rm", "w1"], &["agent", "rm", "w1"]),
            (&["logs", "w1"], &["agent", "logs", "w1"]),
            (
                &["wake", "orchestrator"],
                &["agent", "wake", "orchestrator"],
            ),
            (&["messages"], &["message", "list"]),
        ];
        for (old, new) in pairs {
            let (_, hint) =
                deprecated_spelling(&parse(old)).unwrap_or_else(|| panic!("{old:?} should warn"));
            assert_eq!(hint, new[..2].join(" "), "{old:?}");
            assert!(deprecated_spelling(&parse(new)).is_none(), "{new:?} warned");
            assert_eq!(
                format!("{:?}", normalize(parse(old))),
                format!("{:?}", normalize(parse(new))),
                "{old:?} vs {new:?}"
            );
        }
    }

    #[test]
    fn message_group_forwards_to_the_flat_commands() {
        assert!(matches!(
            normalize(parse(&["message", "send", "human", "hi"])),
            Command::Send(_)
        ));
        assert!(matches!(
            normalize(parse(&["message", "inbox"])),
            Command::Inbox(_)
        ));
        assert!(matches!(
            normalize(parse(&["message", "show", "m-1"])),
            Command::Message(_)
        ));
    }

    /// Scripts and units spell these; they must stay silent.
    #[test]
    fn hook_and_launcher_spellings_do_not_warn() {
        for args in [
            &["statusline"][..],
            &["stop-check"],
            &["arch-guard"],
            &["kill-guard"],
            &["link-check"],
            &["wait-for-wake"],
            &["serve"],
            &["status"],
            &["inbox"],
            &["send", "human", "hi"],
        ] {
            assert!(deprecated_spelling(&parse(args)).is_none(), "{args:?}");
        }
    }
}
