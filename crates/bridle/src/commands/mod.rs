//! Dispatch and implementation for every subcommand except `serve` (see
//! `serve.rs`). See docs/design/cli.md.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use bridle_api::discovery::{self, Env, ProcessEnv};
use bridle_api::{
    AllocPortRequest, BudgetHoldRequest, BudgetOverrideRequest, Client, DoneTaskRequest,
    DropTaskRequest, Edge, EdgeKind, EditTaskRequest, Event, EventQuery, Impact,
    ImpactCheckRequest, InterruptRequest, LandRequest, MaxWorkersRequest, MessageKind,
    MessageQuery, NewEdgeRequest, NewTaskRequest, OverlapLevel, ProbeOutcome, ProbeRequest,
    ProbeResult, RemoveEdgeQuery, RemoveQuery, RenewRequest, ResolveConflictRequest, ResumeRequest,
    SendRequest, SetImpactRequest, SetPriorityRequest, SetSummaryRequest, SpawnRequest, SpecRef,
    StopRequest, Task, TaskKind, TaskPriority, TaskSize, TokenCreateRequest, UsageBreakdownQuery,
    UsageGroupBy, Workdir, event_kind,
};
use chrono::{Local, TimeZone, Utc};
use futures::StreamExt;

use crate::cli::{
    AgentsArgs, AnswerArgs, ArchProposeArgs, AskArgs, BudgetAction, BudgetArgs, BudgetHoldArgs,
    ClaimArgs, Cli, Command, ConflictAction, ConflictArgs, CostAction, CostArgs, CostAuditArgs,
    DepAction, DepArgs, DepEdgeArgs, EdgeKindArg, EventsArgs, HandoverAction, HandoverArgs,
    ImpactAction, ImpactArgs, InboxAction, InboxArgs, InboxReadArgs, InboxShowArgs, InterruptArgs,
    LogsArgs, PaneAction, PrimeArgs, PrimeRoleArg, ProbeArgs, QueueAction, QueueAddTierArgs,
    QueueArgs, QueueSetArgs, ReadyArgs, ReleaseArgs, RmArgs, RulesAction, RulesArgs, RulesDiffArgs,
    RulesExplainArgs, SendArgs, ShowArgs, SpawnArgs, SpecAction, SpecArgs, SpecExportArgs,
    SpecFormatArg, StopArgs, TaskAction, TaskArgs, TaskDoneArgs, TaskDropArgs, TaskEditArgs,
    TaskKindArg, TaskListArgs, TaskNewArgs, TaskNoteArgs, TaskPlanArgs, TaskPriorityArg,
    TaskPriorityArgs, TaskReopenArgs, TaskSearchArgs, TaskShowArgs, TaskSizeArg, TaskSummaryArgs,
    TokenAction, TokenArgs, UsageArgs, UsageByArg, WaitArgs, WhenArg,
};
use crate::cli::{
    FocusAction, FocusArgs, LandArgs, OrchestratorAction, OrchestratorArgs, PortAction, PortArgs,
};
use crate::error::CliError;
use crate::render;
use crate::serve;

mod agent;
mod daemon;
mod hook;
mod misc;
mod orchestrator;
mod task;
mod usage;
mod workflow;
use agent::*;
use daemon::*;
use hook::*;
use misc::*;
use orchestrator::*;
pub use task::print_task_row;
use task::*;
use usage::*;
pub(crate) use workflow::spec_inputs;
use workflow::*;

pub async fn run(cli: Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Serve(args) => serve::run(&cli, args).await,
        Command::StopDaemon => stop_daemon(&cli).await,
        Command::Restart(args) => restart(&cli, args.wait, args.upgrade).await,
        Command::Doctor(args) => crate::doctor::run(&cli, args),
        Command::Init(args) => crate::init::run(args),
        Command::Launchd(args) => crate::launchd::run(&cli, args),
        Command::Systemd(args) => crate::systemd::run(&cli, args),
        Command::Rebuild(args) => rebuild(&cli, args.from_origin).await,
        Command::Daemons => daemons(&cli).await,
        Command::Status => status(&cli).await,
        Command::Spawn(args) => spawn(&cli, args).await,
        Command::Agents(args) => agents(&cli, args).await,
        Command::Show(args) => show(&cli, args).await,
        Command::Send(args) => send(&cli, args).await,
        Command::Inbox(args) => inbox(&cli, args).await,
        Command::Interrupt(args) => interrupt(&cli, args).await,
        Command::Stop(args) => stop(&cli, args).await,
        Command::Resume(args) => resume(&cli, args).await,
        Command::Renew(args) => renew(&cli, args).await,
        Command::Rm(args) => rm(&cli, args).await,
        Command::Logs(args) => logs(&cli, args).await,
        Command::Events(args) => events(&cli, args).await,
        Command::Wait(args) => wait(&cli, args).await,
        Command::Usage(args) => usage(&cli, args).await,
        Command::Cost(args) => cost(&cli, args).await,
        Command::Tui => tui(&cli).await,
        Command::Budget(args) => budget(&cli, args).await,
        Command::Token(args) => token(&cli, args).await,
        Command::Task(args) => task(&cli, args).await,
        Command::Impact(args) => impact(&cli, args).await,
        Command::Probe(args) => probe(&cli, args).await,
        Command::Land(args) => land(&cli, args).await,
        Command::Conflict(args) => conflict(&cli, args).await,
        Command::Port(args) => port(&cli, args).await,
        Command::Dep(args) => dep(&cli, args).await,
        Command::Ask(args) => ask(&cli, args).await,
        Command::Answer(args) => answer(&cli, args).await,
        Command::Claim(args) => claim(&cli, args).await,
        Command::Release(args) => release(&cli, args).await,
        Command::Ready(args) => ready(&cli, args).await,
        Command::Queue(args) => queue(&cli, args).await,
        Command::Statusline => statusline(&cli).await,
        Command::StopCheck => stop_check(&cli).await,
        Command::ArchGuard => arch_guard(&cli).await,
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
        Command::Focus(FocusArgs {
            action: FocusAction::Gate,
        }) => {
            crate::focus::run_gate();
            Ok(())
        }
        Command::Handover(args) => handover(&cli, args).await,
        Command::WaitForWake(args) if args.mail => wait_for_mail(&cli).await,
        Command::WaitForWake(_) => wait_for_wake(&cli).await,
        Command::Mail(args) => match args.action {
            crate::cli::MailAction::Run => mail_run(&cli).await,
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
        Command::Advisor(args) => crate::advisor::run(&cli, &args.action).await,
        Command::Session(args) => crate::session::run(&cli, &args.role).await,
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
    Ok(Client::new(url, token))
}

/// A client for a read-only command: proceeds with no token when `$CLAUDECODE`
/// is set and no token was given, matching the daemon's tolerance for
/// token-less GET/HEAD requests.
async fn client_for_read(cli: &Cli) -> Result<Client, CliError> {
    let (url, token) = resolve_endpoint_and_token(cli, true).await?;
    Ok(Client::new(url, token))
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
