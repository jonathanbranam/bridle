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
    SendRequest, SetImpactRequest, SetSummaryRequest, SpawnRequest, SpecRef, StopRequest, Task,
    TaskKind, TaskSize, TokenCreateRequest, UsageBreakdownQuery, UsageGroupBy, Workdir, event_kind,
};
use chrono::{Local, TimeZone, Utc};
use futures::StreamExt;

use crate::cli::{
    AgentsArgs, AnswerArgs, ArchProposeArgs, AskArgs, BudgetAction, BudgetArgs, BudgetHoldArgs,
    ClaimArgs, Cli, Command, ConflictAction, ConflictArgs, CostAction, CostArgs, CostAuditArgs,
    DepAction, DepArgs, DepEdgeArgs, EdgeKindArg, EventsArgs, ImpactAction, ImpactArgs,
    InboxAction, InboxArgs, InboxReadArgs, InboxShowArgs, InterruptArgs, LogsArgs, PrimeArgs,
    PrimeRoleArg, ProbeArgs, QueueAction, QueueAddTierArgs, QueueArgs, QueueSetArgs, ReadyArgs,
    ReleaseArgs, RmArgs, RulesAction, RulesArgs, RulesDiffArgs, RulesExplainArgs, SendArgs,
    ShowArgs, SpawnArgs, SpecAction, SpecArgs, SpecExportArgs, SpecFormatArg, StopArgs, TaskAction,
    TaskArgs, TaskDoneArgs, TaskDropArgs, TaskEditArgs, TaskKindArg, TaskListArgs, TaskNewArgs,
    TaskNoteArgs, TaskPlanArgs, TaskReopenArgs, TaskSearchArgs, TaskShowArgs, TaskSizeArg,
    TaskSummaryArgs, TokenAction, TokenArgs, UsageArgs, UsageByArg, WaitArgs, WhenArg,
};
use crate::cli::{LandArgs, PortAction, PortArgs};
use crate::error::CliError;
use crate::render;
use crate::serve;

pub async fn run(cli: Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Serve(args) => serve::run(&cli, args).await,
        Command::StopDaemon => stop_daemon(&cli).await,
        Command::Doctor(args) => crate::doctor::run(&cli, args),
        Command::Init(args) => crate::init::run(args),
        Command::Launchd(args) => crate::launchd::run(&cli, args),
        Command::Rebuild => rebuild(&cli).await,
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
        Command::Prime(args) => prime(&cli, args).await,
        Command::Rules(args) => rules(&cli, args).await,
        Command::Sync => sync(&cli).await,
        Command::Spec(SpecArgs {
            action: SpecAction::Export(args),
        }) => spec_export(&cli, args).await,
        Command::Spec(args) => spec(&cli, args),
        Command::Goals(args) => crate::goals::run(&cli, args).await,
        Command::Arch(args) => arch(&cli, args).await,
        Command::Trace(args) => crate::trace::run(&cli, args),
        Command::Explore(args) => explore(&args.action),
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

/// Claude Code's statusLine command (docs/design/cli.md, docs/design/usage-and-budget.md
/// "Where bridle can see usage"). Reads Claude Code's JSON from stdin and prints a short
/// line back. Purely local: no daemon call, so unparseable stdin is the only way this
/// can produce a degraded line, never a slow or failed one. It no longer records a
/// snapshot with the daemon (dropped per s8kn's scope change: the context governor
/// gets account-wide windows from `get_usage` instead, and `POST /v1/statusline` /
/// `interactive_usage` stay in the daemon unused for now, not removed). It does write the
/// session's context size to a local file (`statusline::record_context`, ticket c9zm).
async fn statusline(_cli: &Cli) -> Result<(), CliError> {
    let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    let report = crate::statusline::parse(&input);
    crate::statusline::record_context(&report);
    let workspace_dir = crate::statusline::workspace_dir(&input);
    let folder = workspace_dir
        .as_deref()
        .and_then(|d| d.file_name())
        .map(|n| n.to_string_lossy().into_owned());
    let branch = workspace_dir
        .as_deref()
        .and_then(crate::statusline::git_branch);
    let mut line = crate::statusline::render_line(&report, folder.as_deref(), branch.as_deref());
    let cwd = workspace_dir
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if let Some(counts) =
        bridle_counts(&cwd, &ProcessEnv, &discovery::statusline_token_path()).await
    {
        line.push_str(" \u{b7} ");
        line.push_str(&counts);
    }
    println!("{line}");
    Ok(())
}

/// Claude Code's Stop hook for the worker role (docs/design/coordination.md,
/// docs/spikes/05-stop-hook-findings.md). Per the protocol there: allow
/// immediately (print nothing) when `stop_hook_active` is set, on any error
/// of bridle's own reaching the daemon, and whenever every claimed task has
/// a thread entry since it was claimed; block (print the flat
/// `decision`/`reason` JSON) only for the first claim missing one, or for a
/// finished-looking tree (clean, commits ahead) whose task lacks a summary or
/// a `done:` report.
async fn stop_check(cli: &Cli) -> Result<(), CliError> {
    let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    if input
        .get("stop_hook_active")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let Ok(client) = client_for_read(cli).await else {
        return Ok(());
    };
    let Ok(claimed) = client.list_tasks_claimed_by("me").await else {
        return Ok(());
    };
    if let Some(task) = crate::stop_check::first_unacknowledged_claim(&claimed) {
        println!(
            "{}",
            crate::stop_check::block_json(&crate::stop_check::reason_for(task))
        );
        return Ok(());
    }
    let finished = std::env::current_dir()
        .map(|d| crate::stop_check::looks_finished(&d))
        .unwrap_or(false);
    if finished && !claimed.is_empty() {
        // Blocking work: keep it off the runtime. Config errors allow.
        let reason = tokio::task::spawn_blocking(|| {
            let dir = std::env::current_dir().ok()?;
            let config = bridle_daemon::config::Config::load(&dir).ok()?;
            crate::stop_check::unchecked_head_reason(&dir, Some(config.commands.worker_check()))
        })
        .await
        .ok()
        .flatten();
        if let Some(reason) = reason {
            println!("{}", crate::stop_check::block_json(&reason));
            return Ok(());
        }
    }
    if let Some(task) = crate::stop_check::first_unreported_finish(&claimed, finished) {
        println!(
            "{}",
            crate::stop_check::block_json(&crate::stop_check::unreported_reason_for(task))
        );
    }
    Ok(())
}

/// Claude Code's PreToolUse hook guarding `design/architecture/`
/// (docs/design/architecture-tier.md). Allows (prints nothing) unless the
/// call edits a file there, the caller is a worker agent, and none of its
/// claimed tasks is an `arch-revision`; any error of bridle's own allows.
async fn arch_guard(cli: &Cli) -> Result<(), CliError> {
    let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    let Some(path) = crate::arch_guard::edited_path(&input) else {
        return Ok(());
    };
    let cwd = input.get("cwd").and_then(serde_json::Value::as_str);
    if !crate::arch_guard::is_architecture_path(&path, cwd) {
        return Ok(());
    }
    let Ok(client) = client_for_read(cli).await else {
        return Ok(());
    };
    let Ok(status) = client.status().await else {
        return Ok(());
    };
    // Humans and other non-agent principals are never guarded.
    let Some(name) = status.principal.strip_prefix("agent:") else {
        return Ok(());
    };
    let Ok(agent) = client.get_agent(name).await else {
        return Ok(());
    };
    if agent.role != "worker" {
        return Ok(());
    }
    let Ok(claimed) = client.list_tasks_claimed_by("me").await else {
        return Ok(());
    };
    if crate::arch_guard::should_deny(&path, cwd, &claimed) {
        println!(
            "{}",
            crate::arch_guard::deny_json(&crate::arch_guard::deny_reason(&path))
        );
    }
    Ok(())
}

/// The orchestrator's startup steps, printed by `bridle prime orchestrator`
/// alongside the role prompt and current state (docs/questions/open/
/// one-command-orchestrator-handover-d4mz.md). Kept in the binary, not
/// `scripts/claude-orchestrator`, so there's one source of truth for what a
/// fresh orchestrator session does first; the script just runs `bridle prime
/// orchestrator` for its opening prompt.
const ORCHESTRATOR_STARTUP_STEPS: &str = "\
Check in: `bridle status`, `bridle agents`, and recent messages to human (from the \
product manager and the development manager).
Start the watcher from the latest event seq, plus a 30-minute heartbeat.
Keep both managers' work moving, verify every merge (`just check` twice, off load) \
and push main after verifying, and bring the human only what needs them.
Watch your own context: hand over well before 200K.
The human will mostly reach you through Remote Control.";

/// `bridle prime orchestrator`: a fresh orchestrator session's opening
/// context in one command (docs/questions/open/
/// one-command-orchestrator-handover-d4mz.md, step 2), read from the current
/// directory — run this from the repo root, as
/// `scripts/claude-orchestrator` does. Purely local: no daemon call.
async fn prime(cli: &Cli, args: &PrimeArgs) -> Result<(), CliError> {
    match args.role {
        PrimeRoleArg::Orchestrator => prime_orchestrator().await,
        PrimeRoleArg::Worker => prime_scoped(cli, args, "worker", "worker").await,
        PrimeRoleArg::Planner => prime_scoped(cli, args, "product-manager", "planner").await,
    }
}

/// Worker/planner prime: rules, facts, guides and component scope (`--component`, else
/// the agent's own `BRIDLE_COMPONENTS`). Prime is otherwise orchestrator-only; these two
/// roles each get their own view.
async fn prime_scoped(
    cli: &Cli,
    args: &PrimeArgs,
    role: &str,
    title: &str,
) -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let config =
        bridle_daemon::config::Config::load(&repo).context("loading .bridle/config.toml")?;
    let raw: Vec<String> = if args.components.is_empty() {
        std::env::var("BRIDLE_COMPONENTS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    } else {
        args.components.clone()
    };
    let components = config
        .normalize_components(&raw)
        .map_err(|e| anyhow::anyhow!(e))?;
    let kind = match &args.task {
        Some(id) => Some(
            client_for_read(cli)
                .await?
                .get_task(id)
                .await
                .with_context(|| format!("looking up task {id}"))?
                .kind,
        ),
        None => None,
    };
    print!(
        "{}",
        crate::prime::render(&repo, &config, role, title, &components, kind)?
    );
    Ok(())
}

async fn prime_orchestrator() -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let role_prompt = std::fs::read_to_string(repo.join("workflow/base/roles/orchestrator.md"))
        .context("reading workflow/base/roles/orchestrator.md")?;
    let state = std::fs::read_to_string(repo.join("docs/context/orchestrator-state.md"))
        .context("reading docs/context/orchestrator-state.md")?;
    print!("{}", render_prime_orchestrator(&role_prompt, &state));
    Ok(())
}

fn render_prime_orchestrator(role_prompt: &str, state: &str) -> String {
    format!(
        "# Role: orchestrator\n\n{}\n\n# Current state\n\n{}\n\n# Startup steps\n\n{}\n",
        role_prompt.trim_end(),
        state.trim_end(),
        ORCHESTRATOR_STARTUP_STEPS,
    )
}

/// Bridle's own counts for the human (docs/questions/resolved/
/// statusline-bridle-counts-with-a-read-only-token-r7cs.md): agents working
/// and messages waiting. Attempted only when `token_path` holds a token —
/// deliberately not `$BRIDLE_TOKEN` or the workspace's human token file,
/// since statusline needs a token that works read regardless of which
/// project workspace Claude Code happens to be in, and reading `$BRIDLE_TOKEN`
/// would make every other bridle command run as this token's principal too
/// (discovery::resolve_token checks it first, unconditionally). This token is
/// not scoped read-only or otherwise: it can do whatever its principal can do
/// (docs/design/cli.md). Any failure (no token file, no daemon, timeout,
/// HTTP error) is silent to the line, logged at debug.
async fn bridle_counts(cwd: &Path, env: &impl Env, token_path: &Path) -> Option<String> {
    let token = match std::fs::read_to_string(token_path) {
        Ok(s) => {
            let s = s.trim();
            if s.is_empty() {
                tracing::debug!("statusline: token file {} is empty", token_path.display());
                return None;
            }
            s.to_string()
        }
        Err(e) => {
            tracing::debug!("statusline: no token at {}: {e}", token_path.display());
            return None;
        }
    };
    let endpoint = match discovery::resolve_endpoint(None, None, cwd, env) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("statusline: no bridle daemon found: {e}");
            return None;
        }
    };
    let timeout = std::time::Duration::from_secs(2);
    let client = Client::new_with_timeout(endpoint.url, Some(token), timeout);
    match client.status().await {
        Ok(status) => Some(crate::statusline::render_counts(&status)),
        Err(e) => {
            tracing::debug!("statusline: bridle status call failed: {e}");
            None
        }
    }
}

async fn stop_daemon(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    client.shutdown().await?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if client.health().await.is_err() {
            println!("received; shutting down gracefully, may take up to 30s");
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(CliError::Other(anyhow::anyhow!(
                "daemon did not stop within 60s"
            )));
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

async fn rebuild(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    client.rebuild().await?;
    if cli.json {
        render::print_json(&serde_json::json!({"ok": true}))?;
    } else {
        println!("rebuilt tasks/edges/open_questions from the state branch");
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct DaemonRow {
    #[serde(flatten)]
    info: bridle_api::DaemonInfo,
    /// `None` when the daemon didn't answer `/v1/health` within the timeout.
    agents: Option<u32>,
}

async fn daemons(cli: &Cli) -> Result<(), CliError> {
    let list = discovery::list_registry();
    let timeout = std::time::Duration::from_secs(1);
    let counts = futures::future::join_all(list.iter().map(|d| {
        let client = Client::new_with_timeout(d.url.clone(), None, timeout);
        async move { client.health().await.ok().map(|h| h.agent_count) }
    }))
    .await;
    let rows: Vec<DaemonRow> = list
        .into_iter()
        .zip(counts)
        .map(|(info, agents)| DaemonRow { info, agents })
        .collect();

    if cli.json {
        render::print_json(&rows)?;
    } else if rows.is_empty() {
        println!("no daemons running");
    } else {
        let url_w = rows.iter().map(|d| d.info.url.len()).max().unwrap_or(3);
        println!(
            "{:<16} {:<8} {:<url_w$} {:<7} WORKSPACE",
            "PROJECT", "PID", "URL", "AGENTS"
        );
        for d in &rows {
            let agents = d
                .agents
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string());
            println!(
                "{:<16} {:<8} {:<url_w$} {:<7} {}",
                d.info.project, d.info.pid, d.info.url, agents, d.info.workspace
            );
        }
    }
    Ok(())
}

async fn status(cli: &Cli) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let status = client.status().await?;
    if cli.json {
        render::print_json(&status)?;
    } else {
        println!("project    {}", status.daemon.project);
        println!("workspace  {}", status.daemon.workspace);
        println!("url        {}", status.daemon.url);
        println!("principal  {}", status.principal);
        println!(
            "claude     {}",
            status.claude_version.as_deref().unwrap_or("-")
        );
        println!("unread     {}", status.unread_human_messages);
        if let Some(ci) = &status.ci {
            let age = (chrono::Utc::now() - ci.completed_at).num_minutes().max(0);
            println!(
                "ci         {} {} {}m ago {}",
                &ci.sha[..ci.sha.len().min(8)],
                ci.conclusion,
                age,
                ci.url.as_deref().unwrap_or("")
            );
        }
        for (state, count) in &status.agents_by_state {
            println!("  {state:<10} {count}");
        }
        if !status.merged_leftovers.is_empty() {
            println!(
                "merged     stopped agents whose branch has landed: {} (bridle rm <name> --delete-branch)",
                status.merged_leftovers.join(", ")
            );
        }
        for rl in &status.rate_limits {
            if is_known_rate_limit_window(&rl.window) {
                println!("  {:<10} {}", rl.window, format_utilization(rl.utilization));
            }
        }
    }
    Ok(())
}

fn is_known_rate_limit_window(window: &str) -> bool {
    matches!(
        window,
        "five_hour" | "seven_day" | "seven_day_opus" | "seven_day_sonnet"
    )
}

fn format_utilization(u: Option<f64>) -> String {
    u.map(|u| format!("{:.0}%", u * 100.0))
        .unwrap_or_else(|| "-".to_string())
}

fn format_tokens(tokens: u64) -> String {
    format_tokens_impl(tokens as usize)
}

fn format_tokens_impl(tokens: usize) -> String {
    if tokens >= 10000 {
        let k = tokens as f64 / 1000.0;
        if k >= 100.0 {
            format!("{:.0}k", k)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        tokens.to_string()
    }
}

fn format_cost_dollars(cost: f64) -> String {
    if cost >= 1.0 {
        format!("{:.2}", cost)
    } else {
        format!("{:.4}", cost)
    }
}

async fn spawn(cli: &Cli, args: &SpawnArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let prompt = match (&args.prompt, &args.prompt_file) {
        (Some(p), _) => Some(p.clone()),
        (None, Some(f)) => {
            let content = if f.to_string_lossy() == "-" {
                std::io::read_to_string(std::io::stdin()).context("reading from stdin")?
            } else {
                std::fs::read_to_string(f).with_context(|| format!("reading {}", f.display()))?
            };
            Some(content)
        }
        (None, None) => None,
    };
    let workdir = if args.worktree {
        Some(Workdir::Worktree {
            base: args.base.clone(),
        })
    } else if args.in_repo {
        Some(Workdir::Repo)
    } else {
        args.cwd.as_ref().map(|p| Workdir::Path {
            path: p.to_string_lossy().into_owned(),
        })
    };
    let req = SpawnRequest {
        role: args.role.clone(),
        name: args.name.clone(),
        prompt,
        workdir,
        model: args.model.clone(),
        extra_allowed_tools: args.allow_tool.clone(),
        extra_env: args.env.clone(),
        ignore_budget: args.ignore_budget,
        components: args.component.clone(),
    };
    let agent = client.spawn(&req).await?;
    print_agent(cli, &agent)
}

/// No turn has ended yet, not a real zero-sized context.
fn format_context_tokens(tokens: Option<u64>) -> String {
    match tokens {
        Some(n) => format_tokens(n),
        None => "-".to_string(),
    }
}

fn print_agent(cli: &Cli, agent: &bridle_api::Agent) -> Result<(), CliError> {
    if cli.json {
        render::print_json(agent)?;
    } else {
        println!(
            "{} ({}) role={} state={}",
            agent.name, agent.id, agent.role, agent.state
        );
    }
    Ok(())
}

async fn agents(cli: &Cli, args: &AgentsArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let mut list = client.list_agents().await?;
    if !args.all {
        list.retain(|a| a.state.is_running());
    }
    if cli.json {
        render::print_json(&list)?;
    } else if list.is_empty() {
        println!("no agents");
    } else {
        println!(
            "{:<12} {:<14} {:<10} {:<9} {:<10} {:>5} {:>9} {:>9}",
            "ID", "NAME", "ROLE", "STATE", "MODEL", "TURNS", "COST", "CONTEXT"
        );
        for a in &list {
            println!(
                "{:<12} {:<14} {:<10} {:<9} {:<10} {:>5} {:>9} {:>9}",
                a.id,
                a.name,
                a.role,
                a.state.to_string(),
                a.model,
                a.turns,
                format_cost_dollars(a.cost_usd_total),
                format_context_tokens(a.context_tokens),
            );
        }
    }
    Ok(())
}

async fn show(cli: &Cli, args: &ShowArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let agent = client.get_agent(&args.agent).await?;
    if cli.json {
        render::print_json(&agent)?;
    } else {
        println!("id          {}", agent.id);
        println!("name        {}", agent.name);
        println!("role        {}", agent.role);
        println!("state       {}", agent.state);
        println!("model       {}", agent.model);
        println!("cwd         {}", agent.cwd);
        if let Some(w) = &agent.worktree {
            println!("worktree    {w}");
        }
        if let Some(b) = &agent.branch {
            println!("branch      {b}");
        }
        println!("turns       {}", agent.turns);
        println!("cost        ${}", format_cost_dollars(agent.cost_usd_total));
        println!("held msgs   {}", agent.held_messages);
        println!("unacked     {}", agent.unacked_messages);
        println!(
            "context     {}",
            format_context_tokens(agent.context_tokens)
        );
        if let Some(exit) = &agent.exit {
            println!(
                "exit        code={:?} signal={:?} reason={}",
                exit.code, exit.signal, exit.reason
            );
        }
    }
    Ok(())
}

pub fn read_text(
    text: &Option<String>,
    text_file: &Option<std::path::PathBuf>,
    name: &str,
) -> Result<String, CliError> {
    match (text, text_file) {
        (Some(t), _) => Ok(t.clone()),
        (None, Some(f)) => {
            if f.to_string_lossy() == "-" {
                std::io::read_to_string(std::io::stdin())
                    .context(format!("reading {} from stdin", name))
                    .map_err(CliError::from)
            } else {
                std::fs::read_to_string(f)
                    .with_context(|| format!("reading {} from {}", name, f.display()))
                    .map_err(CliError::from)
            }
        }
        (None, None) => Err(CliError::from(anyhow::anyhow!(
            "specify {} or use --{}-file",
            name,
            name.replace(' ', "-").to_lowercase()
        ))),
    }
}

async fn send(cli: &Cli, args: &SendArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = read_text(&args.text, &args.text_file, "text")?;
    let req = SendRequest {
        to: Some(args.to.clone()),
        body,
        kind: if args.question {
            MessageKind::Question
        } else {
            MessageKind::Note
        },
        when: match args.when {
            WhenArg::Now => bridle_api::When::Now,
            WhenArg::Idle => bridle_api::When::Idle,
        },
        reply_to: args.reply_to.clone(),
        task: args.task.clone(),
    };
    let msgs = client.send(&req).await?;
    if cli.json {
        render::print_json(&msgs)?;
    } else {
        for msg in &msgs {
            println!("sent {} -> {}", msg.id, msg.to);
        }
    }
    Ok(())
}

async fn inbox(cli: &Cli, args: &InboxArgs) -> Result<(), CliError> {
    match &args.action {
        Some(InboxAction::Show(show_args)) => inbox_show(cli, show_args).await,
        Some(InboxAction::Read(read_args)) => inbox_read(cli, read_args).await,
        None => inbox_list(cli, args).await,
    }
}

async fn inbox_list(cli: &Cli, args: &InboxArgs) -> Result<(), CliError> {
    // `--mark-read` writes (POST /v1/messages/{id}/read), but inbox is
    // overwhelmingly a read command, so it gets the same anonymous-read
    // tolerance; `--mark-read` under `$CLAUDECODE` with no token still fails,
    // just from the daemon's 401 rather than a client-side check.
    let client = client_for_read(cli).await?;
    let query = MessageQuery {
        to: Some("me".to_string()),
        unread: !args.all,
        ..Default::default()
    };
    let messages = client.list_messages(&query).await?;
    if args.mark_read {
        for m in &messages {
            client.mark_read(&m.id).await?;
        }
    }
    // Open questions aren't messages "to me": any task's open question is
    // relevant to whoever might answer it, so this lists every one rather
    // than filtering by recipient (there's no per-question recipient to
    // filter on — see docs/design/coordination.md, "Messages").
    let questions = client.list_open_questions().await?;
    if cli.json {
        render::print_json(&serde_json::json!({
            "messages": messages,
            "open_questions": questions,
        }))?;
    } else if messages.is_empty() && questions.is_empty() {
        println!("inbox empty");
    } else {
        for m in &messages {
            let kind = serde_json::to_value(m.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            match &m.answered_by {
                Some(by) => {
                    let line = m.answered_line.as_deref().unwrap_or_default();
                    println!(
                        "{} [{kind}] from {}: answered by {by}: {line}",
                        m.id, m.from
                    );
                }
                None => println!("{} [{kind}] from {}: {}", m.id, m.from, m.body),
            }
        }
        for q in &questions {
            let age = Utc::now() - q.asked_at;
            println!(
                "{} [question] from {}: {} ({})",
                q.task_id,
                q.asked_by,
                q.body,
                format_age(age)
            );
        }
    }
    Ok(())
}

async fn inbox_show(cli: &Cli, args: &InboxShowArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let query = MessageQuery {
        to: Some("me".to_string()),
        ..Default::default()
    };
    let messages = client.list_messages(&query).await?;
    let message = messages
        .iter()
        .find(|m| m.id == args.id)
        .ok_or_else(|| CliError::Other(anyhow::anyhow!("message {} not found", args.id)))?;

    if cli.json {
        render::print_json(message)?;
    } else {
        let kind = serde_json::to_value(message.kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        let local_time = message.created_at.with_timezone(&Local);
        println!("From: {}", message.from);
        println!("Kind: {}", kind);
        println!("Time: {}", local_time.format("%Y-%m-%d %H:%M:%S %Z"));
        if let Some(reply_to) = &message.reply_to {
            println!("Reply-To: {}", reply_to);
        }
        if let Some(by) = &message.answered_by {
            println!(
                "Answered by: {by} ({})",
                message.answered_reply.as_deref().unwrap_or("?")
            );
        }
        println!();
        println!("{}", message.body);
        println!();
        println!(
            "To reply: bridle send {} --reply-to {} \"<message>\"",
            message.from, message.id
        );
    }

    if !args.no_mark_read {
        client.mark_read(&message.id).await?;
    }

    Ok(())
}

async fn inbox_read(cli: &Cli, args: &InboxReadArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    for id in &args.ids {
        client.mark_read(id).await?;
    }
    if cli.json {
        render::print_json(&serde_json::json!({
            "marked_read": args.ids,
        }))?;
    } else {
        for id in &args.ids {
            println!("marked {} read", id);
        }
    }
    Ok(())
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

async fn ask(cli: &Cli, args: &AskArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.ask_question(&args.task, &args.text).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("asked on {}", task.id);
    }
    Ok(())
}

async fn answer(cli: &Cli, args: &AnswerArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.answer_question(&args.task, &args.text).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("answered on {}", task.id);
    }
    Ok(())
}

async fn claim(cli: &Cli, args: &ClaimArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.claim_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("claimed {}", task.id);
    }
    Ok(())
}

async fn release(cli: &Cli, args: &ReleaseArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.release_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("released {}", task.id);
    }
    Ok(())
}

async fn interrupt(cli: &Cli, args: &InterruptArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let resp = client
        .interrupt(
            &args.agent,
            &InterruptRequest {
                drop_held: args.drop_held,
            },
        )
        .await?;
    if cli.json {
        render::print_json(&resp)?;
    } else {
        println!(
            "interrupted {} (dropped {} held)",
            args.agent, resp.dropped_held
        );
    }
    Ok(())
}

async fn stop(cli: &Cli, args: &StopArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let agent = client
        .stop(&args.agent, &StopRequest { now: args.now })
        .await?;
    print_agent(cli, &agent)
}

async fn resume(cli: &Cli, args: &crate::cli::ResumeArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let agent = client
        .resume(
            &args.agent,
            &ResumeRequest {
                ignore_budget: args.ignore_budget,
            },
        )
        .await?;
    print_agent(cli, &agent)
}

async fn renew(cli: &Cli, args: &crate::cli::RenewArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let agent = client
        .renew(
            &args.agent,
            &RenewRequest {
                ignore_budget: args.ignore_budget,
            },
        )
        .await?;
    print_agent(cli, &agent)
}

async fn rm(cli: &Cli, args: &RmArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    client
        .remove(
            &args.agent,
            &RemoveQuery {
                force: args.force,
                delete_branch: args.delete_branch,
            },
        )
        .await?;
    if cli.json {
        render::print_json(&serde_json::json!({"removed": args.agent}))?;
    } else {
        println!("removed {}", args.agent);
    }
    Ok(())
}

async fn logs(cli: &Cli, args: &LogsArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let mut since = args.since;
    loop {
        let lines = client.transcript(&args.agent, since, None).await?;
        for line in &lines {
            since = Some(line.n);
            print_transcript_line(line, args.raw);
        }
        if !args.follow {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn print_transcript_line(line: &bridle_api::TranscriptLine, raw: bool) {
    if raw {
        println!("{}", line.line);
        return;
    }
    if line.dir != "out" {
        return;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&line.line) else {
        return;
    };
    for rendered in render::render_stream_json_line(&value) {
        println!("{rendered}");
    }
}

async fn events(cli: &Cli, args: &EventsArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    if args.follow {
        let mut stream = Box::pin(client.events_stream(args.since));
        while let Some(item) = stream.next().await {
            let ev: Event = item?;
            if matches(&ev, args) {
                print_event(cli, &ev)?;
            }
        }
        Ok(())
    } else {
        let query = EventQuery {
            since: args.since,
            agent: args.agent.clone(),
            kind: args.kind.clone(),
            limit: None,
        };
        let events = client.events(&query).await?;
        if cli.json {
            render::print_json(&events)?;
        } else {
            for ev in &events {
                println!("{}", render::render_event_line(ev));
            }
        }
        Ok(())
    }
}

fn matches(ev: &Event, args: &EventsArgs) -> bool {
    if let Some(agent) = &args.agent
        && ev.agent.as_deref() != Some(agent.as_str())
    {
        return false;
    }
    if let Some(kind) = &args.kind
        && !ev.kind.starts_with(kind.as_str())
    {
        return false;
    }
    true
}

fn print_event(cli: &Cli, ev: &Event) -> Result<(), CliError> {
    if cli.json {
        render::print_json(ev)?;
    } else {
        println!("{}", render::render_event_line(ev));
    }
    Ok(())
}

/// Blocks on the SSE stream (no polling) until the task's state condition or,
/// with `--or-message`, a message to the caller. The cursor is taken before
/// the initial reads so an event landing in between is replayed, not missed.
async fn wait(cli: &Cli, args: &WaitArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let cursor = client
        .events(&EventQuery {
            limit: Some(1),
            ..Default::default()
        })
        .await?
        .last()
        .map_or(0, |e| e.seq);
    let task = client.get_task(&args.task).await?;
    if args.until == Some(task.state) {
        return wait_done(cli, "state", &task.id, Some(task.state.as_str()), None);
    }
    let me = if args.or_message {
        let unread = client
            .list_messages(&MessageQuery {
                to: Some("me".to_string()),
                unread: true,
                limit: Some(1),
                ..Default::default()
            })
            .await?;
        if let Some(m) = unread.first() {
            return wait_done(cli, "message", &task.id, None, Some(&m.id));
        }
        Some(client.status().await?.principal)
    } else {
        None
    };

    let watch = async {
        let mut stream = Box::pin(client.events_stream(Some(cursor)));
        while let Some(item) = stream.next().await {
            let ev: Event = item?;
            if ev.kind == event_kind::TASK_STATE && ev.data["task"] == task.id.as_str() {
                let to = ev.data["to"].as_str().unwrap_or_default().to_string();
                if args.until.is_none_or(|u| u.as_str() == to) {
                    return wait_done(cli, "state", &task.id, Some(&to), None);
                }
            } else if let Some(me) = &me
                && ev.kind == event_kind::MESSAGE_SENT
                && ev.data["to"] == me.as_str()
            {
                let id = ev.data["message"].as_str().unwrap_or_default().to_string();
                return wait_done(cli, "message", &task.id, None, Some(&id));
            }
        }
        Err(CliError::Other(anyhow::anyhow!("event stream ended")))
    };
    let Some(secs) = args.timeout else {
        return watch.await;
    };
    match tokio::time::timeout(std::time::Duration::from_secs(secs), watch).await {
        Ok(r) => r,
        Err(_) => {
            if cli.json {
                render::print_json(&serde_json::json!({"result": "timeout", "task": task.id}))?;
            }
            Err(CliError::Timeout(format!(
                "timed out after {secs}s waiting on {}",
                task.id
            )))
        }
    }
}

fn wait_done(
    cli: &Cli,
    result: &str,
    task: &str,
    state: Option<&str>,
    message: Option<&str>,
) -> Result<(), CliError> {
    if cli.json {
        render::print_json(&serde_json::json!({
            "result": result, "task": task, "state": state, "message": message,
        }))?;
    } else if let Some(m) = message {
        println!("message {m} arrived (waiting on {task})");
    } else {
        println!("{task} is {}", state.unwrap_or_default());
    }
    Ok(())
}

async fn tui(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    bridle_tui::run(client).await?;
    Ok(())
}

async fn usage(cli: &Cli, args: &UsageArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;

    let since = args
        .since
        .as_deref()
        .map(|s| {
            parse_duration(s)
                .map(|d| Utc::now() - d)
                .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --since duration: {s:?}")))
        })
        .transpose()?;

    // `--by role|model` (and `--since` on its own) go through the turns
    // ledger directly; the plain per-agent view keeps using the existing
    // endpoint, unfiltered, as before.
    if since.is_some() || matches!(args.by, Some(UsageByArg::Role) | Some(UsageByArg::Model)) {
        let by = match args.by {
            Some(UsageByArg::Role) => UsageGroupBy::Role,
            Some(UsageByArg::Model) => UsageGroupBy::Model,
            Some(UsageByArg::Agent) | None => UsageGroupBy::Agent,
        };
        let breakdown = client
            .usage_breakdown(&UsageBreakdownQuery {
                since,
                by: Some(by),
            })
            .await?;
        if cli.json {
            render::print_json(&breakdown)?;
        } else {
            // `wall` only means anything grouped by agent (see UsageGroup::wall_seconds).
            let show_wall = by == UsageGroupBy::Agent;
            if show_wall {
                println!(
                    "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8} {:>8}",
                    "KEY", "TURNS", "TOKENS", "COST", "CACHE", "BUSY", "WALL"
                );
            } else {
                println!(
                    "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8}",
                    "KEY", "TURNS", "TOKENS", "COST", "CACHE", "BUSY"
                );
            }
            for g in &breakdown.groups {
                let tokens =
                    g.tokens.input + g.tokens.output + g.tokens.cache_read + g.tokens.cache_write;
                let cache = g
                    .cache_hit_ratio
                    .map(|r| format!("{:.1}%", r * 100.0))
                    .unwrap_or_else(|| "-".to_string());
                let busy = format_duration_secs(g.busy_seconds);
                if show_wall {
                    let wall = g
                        .wall_seconds
                        .map(format_duration_secs)
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8} {:>8}",
                        g.key,
                        g.turns,
                        format_tokens(tokens),
                        format_cost_dollars(g.cost_usd_total),
                        cache,
                        busy,
                        wall
                    );
                } else {
                    println!(
                        "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8}",
                        g.key,
                        g.turns,
                        format_tokens(tokens),
                        format_cost_dollars(g.cost_usd_total),
                        cache,
                        busy
                    );
                }
            }
            let t = &breakdown.total_tokens;
            let total_tokens = t.input + t.output + t.cache_read + t.cache_write;
            println!(
                "total: {} turns, {} tokens, ${}",
                breakdown.total_turns,
                format_tokens(total_tokens),
                format_cost_dollars(breakdown.total_cost_usd)
            );
            if let Some(ratio) = breakdown.cache_hit_ratio {
                println!("cache hit ratio: {:.1}%", ratio * 100.0);
            }
        }
        return Ok(());
    }

    let usage = client.usage().await?;
    if cli.json {
        render::print_json(&usage)?;
    } else {
        println!(
            "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9} {:>8} {:>8}",
            "ID", "NAME", "ROLE", "TURNS", "TOKENS", "COST", "BUSY", "WALL"
        );
        for a in &usage.agents {
            let tokens =
                a.tokens.input + a.tokens.output + a.tokens.cache_read + a.tokens.cache_write;
            let name = if a.removed {
                format!("{} (rm)", a.name)
            } else {
                a.name.clone()
            };
            let wall = a
                .wall_seconds
                .map(format_duration_secs)
                .unwrap_or_else(|| "-".to_string());
            println!(
                "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9} {:>8} {:>8}",
                a.agent,
                name,
                a.role,
                a.turns,
                format_tokens(tokens),
                format_cost_dollars(a.cost_usd_total),
                format_duration_secs(a.busy_seconds),
                wall
            );
        }
        let t = &usage.total_tokens;
        let total_tokens = t.input + t.output + t.cache_read + t.cache_write;
        println!(
            "total: {} turns, {} tokens, ${}",
            usage.total_turns,
            format_tokens(total_tokens),
            format_cost_dollars(usage.total_cost_usd)
        );
        if let Some(ratio) = usage.cache_hit_ratio {
            println!("cache hit ratio: {:.1}%", ratio * 100.0);
        }
        for rl in &usage.rate_limits {
            if is_known_rate_limit_window(&rl.window) {
                let resets = rl
                    .resets_at
                    .map(|r| format!(", resets {}", local_time(r)))
                    .unwrap_or_default();
                println!(
                    "{:<10} {}{resets}",
                    rl.window,
                    format_utilization(rl.utilization)
                );
            }
        }
        if !usage.interactive_today.is_empty() {
            println!("today, interactive (bridle statusline):");
            for row in &usage.interactive_today {
                let cost = row
                    .cost_usd
                    .map(|c| format!("${}", format_cost_dollars(c)))
                    .unwrap_or_else(|| "-".to_string());
                let ctx = match row.context_used_percentage {
                    Some(p) => format!("{:.0}%", p * 100.0),
                    None => "-".to_string(),
                };
                println!(
                    "  {} {:<10} {:>9} ctx {ctx}",
                    row.observed_at.format("%H:%M:%S"),
                    row.model.as_deref().unwrap_or("-"),
                    cost
                );
            }
        }
    }
    Ok(())
}

/// `bridle cost audit`: a static, local check (no daemon involved — it reads
/// `.bridle/config.toml` and `.bridle/cost-baseline.json` from the current
/// directory) of what bridle injects into agent context, per
/// docs/design/usage-and-budget.md ("Tracking token use over time") and
/// `bridle_daemon::cost_audit`.
async fn cost(cli: &Cli, args: &CostArgs) -> Result<(), CliError> {
    match &args.action {
        CostAction::Audit(audit_args) => cost_audit(cli, audit_args).await,
    }
}

async fn cost_audit(cli: &Cli, args: &CostAuditArgs) -> Result<(), CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::cost_audit::{self, RoleAudit};

    let repo = std::env::current_dir().context("current directory")?;
    let config = Config::load(&repo).context("loading .bridle/config.toml")?;
    let current = cost_audit::measure(&config, &repo);
    let baseline = cost_audit::Baseline::load(&repo)
        .context("loading .bridle/cost-baseline.json")?
        .unwrap_or_default();
    let rows = cost_audit::compare(&current, &baseline);

    if cli.json {
        render::print_json(&rows)?;
    } else {
        println!(
            "{:<14} {:>9} {:>9} {:>8}",
            "ROLE", "CURRENT", "BASELINE", "CHANGE"
        );
        for r in &rows {
            print_cost_row(r);
        }
    }

    if args.check {
        let failing: Vec<&RoleAudit> = rows.iter().filter(|r| r.over_threshold).collect();
        if !failing.is_empty() {
            for r in &failing {
                eprintln!(
                    "role {} grew {:.1}% over baseline (threshold {:.0}%)",
                    r.role,
                    r.change_percent.unwrap_or(0.0),
                    cost_audit::GROWTH_THRESHOLD_PERCENT
                );
            }
            return Err(CliError::Other(anyhow::anyhow!(
                "{} role(s) exceeded the cost-growth threshold",
                failing.len()
            )));
        }
    }
    Ok(())
}

fn print_cost_row(r: &bridle_daemon::cost_audit::RoleAudit) {
    let baseline = r
        .baseline_tokens
        .map(format_tokens_impl)
        .unwrap_or_else(|| "-".to_string());
    let change = r
        .change_percent
        .map(|c| format!("{c:+.1}%"))
        .unwrap_or_else(|| "new".to_string());
    let flag = if r.over_threshold { " !" } else { "" };
    println!(
        "{:<14} {:>9} {:>9} {:>8}{flag}",
        r.role,
        format_tokens_impl(r.current_tokens),
        baseline,
        change
    );
}

/// `bridle rules explain`/`diff`: local and static, like `cost audit` — no
/// daemon call, just `.bridle/config.toml` and the layer directories it
/// points at, read from the current directory (docs/design/workflow-layers.md,
/// `bridle_daemon::rules`).
async fn rules(cli: &Cli, args: &RulesArgs) -> Result<(), CliError> {
    match &args.action {
        RulesAction::Explain(e) => rules_explain(cli, e).await,
        RulesAction::Diff(d) => rules_diff(cli, d).await,
    }
}

fn resolve_workflow_rules(
    repo: &Path,
    component: Option<&str>,
) -> Result<bridle_daemon::rules::Resolution, CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::rules;

    let config = Config::load(repo).context("loading .bridle/config.toml")?;
    let workflow_root = config.workflow.as_deref().map(Path::new);
    let mut layers = rules::discover_layers(repo, workflow_root, &config.packs);
    if let Some(id) = component {
        let chain = rules::discover_component_layers(repo, &config, id)
            .ok_or_else(|| anyhow::anyhow!("no component {id:?} in .bridle/config.toml"))?;
        layers.extend(chain);
    }
    rules::load_and_resolve(&layers)
        .map_err(|e| CliError::from(anyhow::Error::new(e).context("resolving workflow rules")))
}

async fn rules_explain(cli: &Cli, args: &RulesExplainArgs) -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let resolution = resolve_workflow_rules(&repo, args.component.as_deref())?;
    let Some(rule) = bridle_daemon::rules::explain(&resolution, &args.id) else {
        return Err(anyhow::anyhow!("no rule with id {:?} in any layer", args.id).into());
    };

    if cli.json {
        render::print_json(rule)?;
        return Ok(());
    }

    println!("{}: won by {}", rule.id, rule.winning_layer());
    for entry in &rule.history {
        let action = match entry.override_kind {
            None => "defines".to_string(),
            Some(kind) => format!("{kind}s it"),
        };
        print!("  {} {action}", entry.layer);
        if let Some(reason) = &entry.reason {
            print!(" ({reason})");
        }
        println!();
    }
    match rule.state() {
        bridle_daemon::rules::RuleState::Active { severity, body, .. } => {
            if let Some(s) = severity {
                println!("severity: {s}");
            }
            println!("{body}");
        }
        bridle_daemon::rules::RuleState::Disabled { reason } => {
            println!("disabled: {reason}");
        }
    }
    Ok(())
}

async fn rules_diff(cli: &Cli, args: &RulesDiffArgs) -> Result<(), CliError> {
    if !args.project_layer && args.component.is_none() {
        return Err(anyhow::anyhow!(
            "rules diff needs a mode: --project-layer or --component <id>"
        )
        .into());
    }
    let repo = std::env::current_dir().context("current directory")?;
    let resolution = resolve_workflow_rules(&repo, args.component.as_deref())?;
    if args.component.is_some() {
        let diffs = bridle_daemon::rules::diff_components(&resolution);
        if cli.json {
            render::print_json(&diffs)?;
            return Ok(());
        }
        if diffs.is_empty() {
            println!("the component chain changes nothing");
        }
        for d in &diffs {
            let action = d
                .diff
                .override_kind
                .map(|k| format!("{k}s"))
                .unwrap_or_else(|| "defines".to_string());
            print!("{}: {} {action}", d.component, d.diff.id);
            if let Some(reason) = &d.diff.reason {
                print!(" ({reason})");
            }
            println!();
        }
        return Ok(());
    }
    let diffs = bridle_daemon::rules::diff_project(&resolution);

    if cli.json {
        render::print_json(&diffs)?;
        return Ok(());
    }

    if diffs.is_empty() {
        println!("project layer changes nothing");
        return Ok(());
    }
    for d in &diffs {
        let action = d
            .override_kind
            .map(|k| format!("{k}s"))
            .unwrap_or_else(|| "defines".to_string());
        print!("{} {action}", d.id);
        if let Some(reason) = &d.reason {
            print!(" ({reason})");
        }
        println!();
    }
    Ok(())
}

/// `bridle sync`: local and static, like `rules explain`/`diff` — renders
/// the resolved workflow layers into CLAUDE.md, .claude/skills,
/// .claude/agents and .claude/settings.json's hooks
/// (docs/design/workflow-layers.md, `bridle_daemon::sync`).
async fn sync(cli: &Cli) -> Result<(), CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::rules;

    let repo = std::env::current_dir().context("current directory")?;
    let config = Config::load(&repo).context("loading .bridle/config.toml")?;
    let workflow_root = config.workflow.as_deref().map(Path::new);
    let layers = rules::discover_layers(&repo, workflow_root, &config.packs);
    let report = bridle_daemon::sync::sync(&repo, &layers, &config.commands, &config.branches)
        .map_err(|e| CliError::from(anyhow::Error::new(e).context("syncing workflow layers")))?;

    if cli.json {
        render::print_json(&report)?;
        return Ok(());
    }

    println!(
        "CLAUDE.md: {}",
        if report.claude_md_changed {
            "updated"
        } else {
            "unchanged"
        }
    );
    let list = |items: &[String]| {
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    };
    println!("skills: {}", list(&report.skills));
    println!("agents: {}", list(&report.agents));
    println!("hooks: {}", list(&report.hook_events));
    Ok(())
}

async fn budget(cli: &Cli, args: &BudgetArgs) -> Result<(), CliError> {
    // Only the no-action form (`bridle budget`) is a read; every `BudgetAction`
    // writes, so it keeps the client-side check.
    let client = if args.action.is_none() {
        client_for_read(cli).await?
    } else {
        client_for(cli).await?
    };
    let budget = match &args.action {
        None => client.budget().await?,
        Some(BudgetAction::Hold(hold_args)) => {
            let until = resolve_hold_until(hold_args)?;
            client.budget_hold(&BudgetHoldRequest { until }).await?
        }
        Some(BudgetAction::Release) => client.budget_release().await?,
        Some(BudgetAction::Override(o)) => {
            let period = (o.period != "default").then(|| o.period.clone());
            let until = o.until.as_deref().map(parse_local_until).transpose()?;
            client
                .budget_override(&BudgetOverrideRequest { period, until })
                .await?
        }
        Some(BudgetAction::OverrideClear) => client.budget_override_clear().await?,
        Some(BudgetAction::MaxWorkers(a)) => {
            client
                .budget_max_workers(&MaxWorkersRequest { max_workers: a.n })
                .await?
        }
    };
    if cli.json {
        render::print_json(&budget)?;
    } else {
        if args.schedule {
            print_schedule(&budget);
            return Ok(());
        }
        println!("state  {}", budget.state);
        for reason in &budget.reasons {
            println!("why    {reason}");
        }
        if let Some(hold) = &budget.human_hold {
            let until = hold
                .until
                .map(|u| format!(", until {}", local_time(u)))
                .unwrap_or_else(|| ", until released".to_string());
            println!("hold   in force{until}");
        }
        if let Some(ov) = &budget.schedule_override {
            let period = ov.period.as_deref().unwrap_or("default");
            let until = ov
                .until
                .map(|u| format!(", until {}", local_time(u)))
                .unwrap_or_else(|| ", until cleared".to_string());
            println!("override {period}{until}");
        }
        let fh = &budget.five_hour;
        let from = match (fh.source.as_str(), &fh.period) {
            ("default", _) => "default".to_string(),
            (src, Some(p)) => format!("{src} {p}"),
            (src, None) => format!("{src} default"),
        };
        println!(
            "five_hour thresholds ({from}): hold {}, wind_down {}, stop {}",
            fh.hold_at, fh.wind_down_at, fh.stop_at
        );
        if let Some(span) = &fh.span {
            println!("  span {}", format_span(span));
        }
        if let Some(n) = fh.max_workers {
            println!("  period max_workers {n} (applied while overridden)");
        }
        match &fh.next_change {
            Some(n) => println!(
                "  next change {} -> {}: hold {}, wind_down {}, stop {}",
                local_time(n.at),
                n.period.as_deref().unwrap_or("default"),
                n.hold_at,
                n.wind_down_at,
                n.stop_at
            ),
            None => println!("  next change: none scheduled"),
        }
        for w in &budget.windows {
            let resets = w
                .resets_at
                .map(|r| format!(", resets {}", local_time(r)))
                .unwrap_or_default();
            let age = match w.age_secs {
                Some(s) => format!(", read {}", format_age(chrono::Duration::seconds(s as i64))),
                None => ", no reading".to_string(),
            };
            let status = match w.status.as_deref() {
                Some("allowed") | None => String::new(),
                Some(s) => format!(", status {s}"),
            };
            let staleness = if w.stale { " (stale)" } else { "" };
            println!(
                "  {:<16} {:<12} {}{resets}{status}{age}{staleness}",
                w.window,
                w.state.to_string(),
                format_utilization(w.utilization)
            );
        }
        let t = &budget.thresholds;
        let max_workers = match budget.max_workers_override {
            Some(n) => format!("{n} (override; configured {})", t.max_workers),
            None => t.max_workers.to_string(),
        };
        println!(
            "max_workers {max_workers}, wind_down_grace {}s, max_staleness {}s",
            t.wind_down_grace_secs, t.max_staleness_secs
        );
    }
    Ok(())
}

/// A time for the human, in the machine's local timezone (`bridle budget`'s
/// own choice; `--json` stays UTC).
fn local_time(t: chrono::DateTime<Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M %Z")
        .to_string()
}

fn format_span(s: &bridle_api::types::ScheduleSpan) -> String {
    format!("{} {}-{}", s.days.join(","), s.start, s.end)
}

/// `bridle budget --schedule`: the resolved periods in match order (the
/// first match wins), start/end in host-local time.
fn print_schedule(budget: &bridle_api::types::BudgetStatus) {
    if budget.schedule.is_empty() {
        println!("no [[budget.schedule]] periods; the plain [budget] thresholds always apply");
        return;
    }
    for p in &budget.schedule {
        println!(
            "{:<12} {}  hold {}, wind_down {}, stop {}{}",
            p.name,
            p.span
                .as_ref()
                .map_or_else(|| "override only".to_string(), format_span),
            p.hold_at,
            p.wind_down_at,
            p.stop_at,
            p.max_workers
                .map(|n| format!(", max_workers {n}"))
                .unwrap_or_default()
        );
    }
}

/// `--for 3h` (a plain duration from now) or `--until 18:00` (a local
/// `HH:MM`, resolved to the next occurrence: today if still ahead, else
/// tomorrow); neither given holds until `bridle budget release`.
fn resolve_hold_until(args: &BudgetHoldArgs) -> Result<Option<chrono::DateTime<Utc>>, CliError> {
    if let Some(for_) = &args.for_ {
        let dur = parse_duration(for_)
            .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --for duration: {for_:?}")))?;
        return Ok(Some(Utc::now() + dur));
    }
    if let Some(until) = &args.until {
        return Ok(Some(parse_local_until(until)?));
    }
    Ok(None)
}

/// `HH:MM` local time, resolved to the next occurrence: today if still
/// ahead, else tomorrow. Shared by `bridle budget hold --until` and
/// `bridle budget override --until`.
fn parse_local_until(until: &str) -> Result<chrono::DateTime<Utc>, CliError> {
    let (h, m) = until
        .split_once(':')
        .and_then(|(h, m)| Some((h.parse::<u32>().ok()?, m.parse::<u32>().ok()?)))
        .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --until time: {until:?}")))?;
    let now = Local::now();
    let mut target = now
        .date_naive()
        .and_hms_opt(h, m, 0)
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --until time: {until:?}")))?;
    if target <= now {
        target += chrono::Duration::days(1);
    }
    Ok(target.with_timezone(&Utc))
}

/// A plain `<n><unit>` duration (`s`/`m`/`h`/`d`), matching config.toml's.
fn parse_duration(s: &str) -> Option<chrono::Duration> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.len().checked_sub(1)?);
    let n: i64 = num.parse().ok()?;
    match unit {
        "s" => Some(chrono::Duration::seconds(n)),
        "m" => Some(chrono::Duration::minutes(n)),
        "h" => Some(chrono::Duration::hours(n)),
        "d" => Some(chrono::Duration::days(n)),
        _ => None,
    }
}

/// The project a `token create`/`revoke` acted on: `--project`, or the cwd's daemon.
fn token_project(cli: &Cli) -> Result<Option<String>, CliError> {
    let cwd = std::env::current_dir().context("current directory")?;
    let endpoint = discovery::resolve_endpoint(
        cli.url.as_deref(),
        cli.project.as_deref(),
        &cwd,
        &ProcessEnv,
    )
    .map_err(|e| CliError::Unreachable(e.to_string()))?;
    Ok(endpoint.project)
}

async fn token(cli: &Cli, args: &TokenArgs) -> Result<(), CliError> {
    let client = if matches!(args.action, TokenAction::List) {
        client_for_read(cli).await?
    } else {
        client_for(cli).await?
    };
    match &args.action {
        TokenAction::Create { name } => {
            let created = client
                .create_token(&TokenCreateRequest { name: name.clone() })
                .await?;
            // Stored for the project this command talked to, so the token never has
            // to be seen; with no project (daemon found by URL) it's printed as before.
            let stored_in = match token_project(cli)? {
                Some(project) => {
                    let path = discovery::credentials_path();
                    discovery::store_credential(&path, name, &project, &created.token).map_err(
                        |e| {
                            CliError::Other(anyhow::anyhow!(
                                "token created but not saved (it is shown once): {e}: {}",
                                created.token
                            ))
                        },
                    )?;
                    Some((path, project))
                }
                None => None,
            };
            if cli.json {
                render::print_json(&created)?;
            } else if let Some((path, project)) = stored_in {
                println!(
                    "principal {}: token for project {project} saved in {}",
                    created.principal,
                    path.display()
                );
            } else {
                println!("{}", created.token);
                eprintln!(
                    "principal {} — this token is shown once; store it now",
                    created.principal
                );
            }
        }
        TokenAction::List => {
            let tokens = client.list_tokens().await?;
            if cli.json {
                render::print_json(&tokens)?;
            } else if tokens.is_empty() {
                println!("no external tokens");
            } else {
                println!("{:<20} {:<26} REVOKED", "NAME", "CREATED");
                for t in &tokens {
                    println!(
                        "{:<20} {:<26} {}",
                        t.name,
                        t.created_at.to_rfc3339(),
                        t.revoked
                    );
                }
            }
        }
        TokenAction::Revoke { name } => {
            client.revoke_token(name).await?;
            println!("revoked {name}");
            if let Some(project) = token_project(cli)? {
                let path = discovery::credentials_path();
                if discovery::remove_credential(&path, name, &project)
                    .map_err(|e| CliError::Other(e.into()))?
                {
                    println!(
                        "removed its entry for project {project} from {}",
                        path.display()
                    );
                }
            }
        }
    }
    Ok(())
}

fn task_kind_arg(k: TaskKindArg) -> TaskKind {
    match k {
        TaskKindArg::Feature => TaskKind::Feature,
        TaskKindArg::Bug => TaskKind::Bug,
        TaskKindArg::Chore => TaskKind::Chore,
        TaskKindArg::Question => TaskKind::Question,
        TaskKindArg::Research => TaskKind::Research,
        TaskKindArg::Explore => TaskKind::Explore,
        TaskKindArg::ArchRevision => TaskKind::ArchRevision,
        TaskKindArg::ReEvaluate => TaskKind::ReEvaluate,
    }
}

async fn task(cli: &Cli, args: &TaskArgs) -> Result<(), CliError> {
    match &args.action {
        TaskAction::New(a) => task_new(cli, a).await,
        TaskAction::Show(a) => task_show(cli, a).await,
        TaskAction::Edit(a) => task_edit(cli, a).await,
        TaskAction::List(a) => task_list(cli, a).await,
        TaskAction::Plan(a) => task_plan(cli, a).await,
        TaskAction::Drop(a) => task_drop(cli, a).await,
        TaskAction::Done(a) => task_done(cli, a).await,
        TaskAction::Summary(a) => task_summary(cli, a).await,
        TaskAction::Reopen(a) => task_reopen(cli, a).await,
        TaskAction::Note(a) => task_note(cli, a).await,
        TaskAction::Search(a) => task_search(cli, a).await,
    }
}

/// Whether the project's `.bridle/config.toml` (read from the current
/// directory, like `bridle rules`) defines any components. Best effort: the
/// reminder it gates is only a nudge.
fn project_has_components() -> bool {
    std::env::current_dir()
        .ok()
        .and_then(|d| bridle_daemon::config::Config::load(&d).ok())
        .is_some_and(|c| !c.components.is_empty())
}

fn size_str(size: Option<TaskSize>) -> &'static str {
    size.map_or("-", TaskSize::as_str)
}

fn task_size_arg_to_opt(arg: TaskSizeArg) -> Option<TaskSize> {
    match arg {
        TaskSizeArg::S => Some(TaskSize::S),
        TaskSizeArg::M => Some(TaskSize::M),
        TaskSizeArg::L => Some(TaskSize::L),
        TaskSizeArg::None => Some(TaskSize::None),
    }
}

pub fn print_task_row(t: &Task) {
    println!(
        "{:<10} {:<9} {:<8} {:<4} {}",
        t.id,
        t.kind,
        t.state,
        size_str(t.size),
        t.title
    );
}

async fn task_new(cli: &Cli, args: &TaskNewArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = if args.body.is_some() || args.body_file.is_some() {
        read_text(&args.body, &args.body_file, "body")?
    } else {
        String::new()
    };
    let req = NewTaskRequest {
        title: args.title.clone(),
        kind: task_kind_arg(args.kind),
        body,
        components: args.component.clone(),
        size: args.size.and_then(task_size_arg_to_opt),
    };
    let task = client.new_task(&req).await?;
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

async fn task_show(cli: &Cli, args: &TaskShowArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let task = client.get_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("id          {}", task.id);
        println!("title       {}", task.title);
        println!("kind        {}", task.kind);
        println!("state       {}", task.state);
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
        println!("updated     {}", task.updated_at.to_rfc3339());
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

async fn task_edit(cli: &Cli, args: &TaskEditArgs) -> Result<(), CliError> {
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
    };
    let task = client.edit_task(&args.task, &req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

async fn task_list(cli: &Cli, args: &TaskListArgs) -> Result<(), CliError> {
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
    if cli.json {
        render::print_json(&tasks)?;
    } else if tasks.is_empty() {
        println!("no tasks");
    } else {
        println!(
            "{:<10} {:<9} {:<8} {:<4} TITLE",
            "ID", "KIND", "STATE", "SIZE"
        );
        for t in &tasks {
            print_task_row(t);
        }
    }
    Ok(())
}

async fn task_plan(cli: &Cli, args: &TaskPlanArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.plan_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

async fn task_drop(cli: &Cli, args: &TaskDropArgs) -> Result<(), CliError> {
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

async fn land(cli: &Cli, args: &LandArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let r = client
        .land_task(
            &args.task,
            &LandRequest {
                branch: args.branch.clone(),
                check_cmd: args.check_cmd.clone(),
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

async fn task_done(cli: &Cli, args: &TaskDoneArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let req = DoneTaskRequest {
        commit: args.commit.clone(),
        branch: args.branch.clone(),
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
    if task.summary.is_none() {
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

async fn impact(cli: &Cli, args: &ImpactArgs) -> Result<(), CliError> {
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
fn spec_map(dir: &Path) -> BTreeMap<String, SpecRef> {
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

fn probe_line(r: &ProbeResult) -> String {
    match r.outcome {
        ProbeOutcome::Clean => format!("{} merges cleanly into {}", r.branch, r.against),
        ProbeOutcome::Conflict => format!(
            "textual conflict of {} with {}: {}",
            r.branch,
            r.against,
            r.paths.join(", ")
        ),
        ProbeOutcome::Unsupported => "unsupported git: merge probe needs git 2.38 or newer".into(),
    }
}

async fn probe(cli: &Cli, args: &ProbeArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let r = client
        .probe(&ProbeRequest {
            target: args.target.clone(),
            branch: args.branch.clone(),
        })
        .await?;
    if cli.json {
        render::print_json(&r)?;
    } else {
        println!("{}", probe_line(&r));
    }
    if r.outcome == ProbeOutcome::Conflict {
        return Err(CliError::Other(anyhow::anyhow!("merge conflict")));
    }
    Ok(())
}

async fn impact_check(cli: &Cli, client: &Client, specs: &Path) -> Result<(), CliError> {
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

async fn task_summary(cli: &Cli, args: &TaskSummaryArgs) -> Result<(), CliError> {
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

async fn task_reopen(cli: &Cli, args: &TaskReopenArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.reopen_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

async fn task_note(cli: &Cli, args: &TaskNoteArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let text = read_text(&args.text, &args.text_file, "text")?;
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
            println!("noted on {}, notified {to}", args.task);
        }
        return Ok(());
    }
    let task = client.note_task(&args.task, &text).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("noted on {}", task.id);
    }
    Ok(())
}

async fn task_search(cli: &Cli, args: &TaskSearchArgs) -> Result<(), CliError> {
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

fn edge_kind_arg(k: EdgeKindArg) -> EdgeKind {
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
fn resolve_edge_args(args: &DepEdgeArgs) -> Result<(String, String, EdgeKind), CliError> {
    match (&args.blocked_by, &args.to) {
        (Some(blocker), None) => Ok((blocker.clone(), args.task.clone(), EdgeKind::Blocks)),
        (None, Some(to)) => Ok((args.task.clone(), to.clone(), edge_kind_arg(args.kind))),
        (None, None) => Err(CliError::from(anyhow::anyhow!(
            "specify --to <task> or --blocked-by <task>"
        ))),
        (Some(_), Some(_)) => unreachable!("clap's conflicts_with_all rules this out"),
    }
}

fn print_edge_row(e: &Edge) {
    println!("{:<10} {:<16} {}", e.from, e.kind, e.to);
}

async fn port(cli: &Cli, args: &PortArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    match &args.action {
        PortAction::Alloc(a) => {
            let p = client
                .alloc_port(&AllocPortRequest {
                    pid: a.pid,
                    label: a.label.clone(),
                })
                .await?;
            if cli.json {
                render::print_json(&p)?;
            } else {
                println!("{}", p.port);
            }
        }
        PortAction::Release(a) => {
            let p = client.release_port(a.port).await?;
            if cli.json {
                render::print_json(&p)?;
            } else {
                println!("released {}", p.port);
            }
        }
        PortAction::List => {
            let list = client.list_ports().await?;
            if cli.json {
                render::print_json(&list)?;
            } else if list.is_empty() {
                println!("no ports allocated");
            } else {
                for p in &list {
                    println!(
                        "{:<6} {:<14} {:<10} {:<7} {}",
                        p.port,
                        p.agent,
                        p.task.as_deref().unwrap_or("-"),
                        p.pid.map(|n| n.to_string()).unwrap_or_else(|| "-".into()),
                        p.label.as_deref().unwrap_or("")
                    );
                }
            }
        }
    }
    Ok(())
}

async fn conflict(cli: &Cli, args: &ConflictArgs) -> Result<(), CliError> {
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

async fn dep(cli: &Cli, args: &DepArgs) -> Result<(), CliError> {
    match &args.action {
        DepAction::Add(a) => dep_add(cli, a).await,
        DepAction::Rm(a) => dep_rm(cli, a).await,
    }
}

async fn dep_add(cli: &Cli, args: &DepEdgeArgs) -> Result<(), CliError> {
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

async fn dep_rm(cli: &Cli, args: &DepEdgeArgs) -> Result<(), CliError> {
    let (from, to, kind) = resolve_edge_args(args)?;
    let client = client_for(cli).await?;
    client
        .remove_edge(&RemoveEdgeQuery { from, to, kind })
        .await?;
    Ok(())
}

fn print_task_row_with_project(project: Option<&str>, t: &Task) {
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

async fn ready(cli: &Cli, args: &ReadyArgs) -> Result<(), CliError> {
    // `--role` has nothing to filter on yet: tasks don't carry a role field
    // (P0-1 gap, docs/design/cli.md). Accepted, not rejected, so a caller
    // scripting ahead of that field landing doesn't need to special-case it.
    let _ = &args.role;

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

async fn queue(cli: &Cli, args: &QueueArgs) -> Result<(), CliError> {
    match &args.action {
        None => queue_show(cli).await,
        Some(QueueAction::Set(a)) => queue_set(cli, a).await,
        Some(QueueAction::AddTier(a)) => queue_add_tier(cli, a).await,
    }
}

/// Splits `--tier tw-1,tw-2 --tier tw-3` into the ordered `Vec<Vec<String>>`
/// `set_queue`/`add_queue_tier` want.
fn split_tier(s: &str) -> Vec<String> {
    s.split(',')
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

async fn queue_set(cli: &Cli, args: &QueueSetArgs) -> Result<(), CliError> {
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

async fn queue_add_tier(cli: &Cli, args: &QueueAddTierArgs) -> Result<(), CliError> {
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
struct QueueTaskRow {
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
}

#[derive(serde::Serialize)]
struct QueueTierRow {
    rank: usize,
    tasks: Vec<QueueTaskRow>,
}

#[derive(serde::Serialize)]
struct QueueView {
    claimed: Vec<Task>,
    tiers: Vec<QueueTierRow>,
}

/// `bridle queue`'s read-only view: claimed tasks with their worker, then
/// the tiers in rank order, each task marked startable or blocked
/// (roles-and-lifecycle.md, "the queue"). Composed client-side from three
/// plain reads (queue, all tasks, ready tasks) rather than a bespoke server
/// endpoint, since none of the three needs PM/human-only write access.
async fn queue_show(cli: &Cli) -> Result<(), CliError> {
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
            let mark = if t.startable { "startable" } else { "blocked" };
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
mod bridle_counts_tests {
    use super::bridle_counts;

    #[tokio::test]
    async fn missing_token_file_skips_the_daemon_call() {
        let dir = tempfile::tempdir().unwrap();
        let env = |_key: &str| None;
        let token_path = dir.path().join("statusline.token");
        assert_eq!(bridle_counts(dir.path(), &env, &token_path).await, None);
    }

    #[tokio::test]
    async fn empty_token_file_skips_the_daemon_call() {
        let dir = tempfile::tempdir().unwrap();
        let token_path = dir.path().join("statusline.token");
        std::fs::write(&token_path, "  \n").unwrap();
        let env = |_key: &str| None;
        assert_eq!(bridle_counts(dir.path(), &env, &token_path).await, None);
    }

    #[tokio::test]
    async fn token_but_no_daemon_found_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let token_path = dir.path().join("statusline.token");
        std::fs::write(&token_path, "tok\n").unwrap();
        let env = |_key: &str| None;
        assert_eq!(bridle_counts(dir.path(), &env, &token_path).await, None);
    }

    #[tokio::test]
    async fn token_set_but_daemon_unreachable_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let token_path = dir.path().join("statusline.token");
        std::fs::write(&token_path, "tok\n").unwrap();
        let env = |key: &str| match key {
            // Port 0 refuses connections outright, so this fails fast
            // rather than exercising the full 2s timeout.
            "BRIDLE_URL" => Some("http://127.0.0.1:0".to_string()),
            _ => None,
        };
        assert_eq!(bridle_counts(dir.path(), &env, &token_path).await, None);
    }
}

#[cfg(test)]
mod prime_tests {
    use super::render_prime_orchestrator;

    #[test]
    fn includes_role_prompt_state_and_startup_steps() {
        let out = render_prime_orchestrator(
            "You're my orchestrator for bridle.",
            "## Handover, 2026-09-28",
        );
        assert!(out.contains("You're my orchestrator for bridle."));
        assert!(out.contains("## Handover, 2026-09-28"));
        assert!(out.contains("bridle status"));
        assert!(out.contains("30-minute heartbeat"));
        // Sections appear in a fixed, readable order.
        let role_pos = out.find("# Role: orchestrator").unwrap();
        let state_pos = out.find("# Current state").unwrap();
        let steps_pos = out.find("# Startup steps").unwrap();
        assert!(role_pos < state_pos);
        assert!(state_pos < steps_pos);
    }
}

#[derive(serde::Serialize)]
struct SpecDiagnostic {
    file: String,
    line: usize,
    column: usize,
    severity: &'static str,
    message: String,
}

#[derive(serde::Serialize)]
struct SpecCheckReport {
    files: usize,
    errors: usize,
    warnings: usize,
    diagnostics: Vec<SpecDiagnostic>,
}

/// Every `*.md` under `path` (or `path` itself if it's a file), sorted so
/// output is stable.
fn spec_files(path: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if path.is_dir() {
        let mut entries = std::fs::read_dir(path)
            .with_context(|| format!("reading {}", path.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
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

/// The spec files named by `paths`, else those under `root` (default
/// `design/specs`): the defaults every `bridle spec` subcommand shares.
pub(crate) fn spec_inputs(paths: &[PathBuf], root: Option<&Path>) -> anyhow::Result<Vec<PathBuf>> {
    let default = [root.map_or_else(|| PathBuf::from("design/specs"), Path::to_path_buf)];
    let roots = if paths.is_empty() {
        &default[..]
    } else {
        paths
    };
    let mut files = Vec::new();
    for r in roots {
        spec_files(r, &mut files)?;
    }
    Ok(files)
}

/// `bridle spec check`: local, no daemon call (docs/design/specs.md).
fn spec(cli: &Cli, args: &SpecArgs) -> Result<(), CliError> {
    let args = match &args.action {
        SpecAction::Check(args) => args,
        SpecAction::Export(_) => unreachable!("dispatched to spec_export"),
        SpecAction::Id(args) => return crate::specid::run(cli, args),
        SpecAction::Import(args) => return crate::spec_import::run(cli, args),
        SpecAction::Coverage(args) => return crate::spec_coverage::run(cli, args),
    };
    let files = spec_inputs(&args.paths, args.root.as_deref())?;

    let mut diagnostics = Vec::new();
    for file in &files {
        let name = file.display().to_string();
        match bridle_spec::parse_file(file) {
            Ok(spec) => {
                for r in spec.requirements.iter().filter(|r| r.id.is_none()) {
                    diagnostics.push(SpecDiagnostic {
                        file: name.clone(),
                        line: r.line,
                        column: 1,
                        severity: if args.require_ids { "error" } else { "warning" },
                        message: format!(
                            "requirement {:?} has no id (expected '{{#r-xxxx}}' after the title)",
                            r.title
                        ),
                    });
                }
            }
            Err(bridle_spec::ParseFileError::Diagnostics(ds)) => {
                diagnostics.extend(ds.into_iter().map(|d| SpecDiagnostic {
                    file: d.file,
                    line: d.line,
                    column: d.column,
                    severity: "error",
                    message: d.message,
                }));
            }
            Err(e) => return Err(anyhow::Error::new(e).into()),
        }
    }
    let count = |sev| diagnostics.iter().filter(|d| d.severity == sev).count();
    let report = SpecCheckReport {
        files: files.len(),
        errors: count("error"),
        warnings: count("warning"),
        diagnostics,
    };

    if cli.json {
        render::print_json(&report)?;
    } else {
        for d in &report.diagnostics {
            let prefix = if d.severity == "warning" {
                "warning: "
            } else {
                ""
            };
            println!("{}:{}:{}: {prefix}{}", d.file, d.line, d.column, d.message);
        }
        println!(
            "{} file(s) checked: {} error(s), {} warning(s)",
            report.files, report.errors, report.warnings
        );
    }
    if report.errors > 0 {
        return Err(anyhow::anyhow!("spec check found {} error(s)", report.errors).into());
    }
    Ok(())
}

/// `bridle arch list`: local, no daemon call (docs/design/architecture-tier.md).
async fn arch(cli: &Cli, args: &crate::cli::ArchArgs) -> Result<(), CliError> {
    match &args.action {
        crate::cli::ArchAction::List(args) => arch_list(cli, args),
        crate::cli::ArchAction::Propose(args) => arch_propose(cli, args).await,
    }
}

fn arch_list(cli: &Cli, args: &crate::cli::ArchListArgs) -> Result<(), CliError> {
    let files = spec_inputs(std::slice::from_ref(&args.root), None)?;
    let mut elements = match bridle_spec::arch::parse_files(&files) {
        Ok(els) => els,
        Err(ds) => {
            for d in &ds {
                eprintln!("{d}");
            }
            return Err(anyhow::anyhow!("arch list found {} error(s)", ds.len()).into());
        }
    };
    if args.invariants {
        elements.retain(|e| e.invariant);
    }
    if cli.json {
        render::print_json(&elements)?;
    } else {
        for e in &elements {
            let flag = if e.invariant { " invariant" } else { "" };
            println!("{}{flag}  {}  ({}:{})", e.id, e.title, e.file, e.line);
        }
    }
    Ok(())
}

async fn arch_propose(cli: &Cli, args: &ArchProposeArgs) -> Result<(), CliError> {
    let files = spec_inputs(std::slice::from_ref(&args.arch_root), None)?;
    if let Err(ds) = bridle_spec::arch::parse_files(&files) {
        for d in &ds {
            eprintln!("{d}");
        }
        return Err(anyhow::anyhow!("arch parse found {} error(s)", ds.len()).into());
    }

    let argument = if args.argument.is_some() || args.argument_file.is_some() {
        read_text(&args.argument, &args.argument_file, "argument")?
    } else {
        String::new()
    };

    let client = client_for(cli).await?;
    let req = NewTaskRequest {
        title: args.title.clone(),
        kind: TaskKind::ArchRevision,
        body: argument,
        components: Vec::new(),
        size: None,
    };
    let task = client.new_task(&req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

/// `bridle explore`: local, no daemon call (docs/design/explorations.md).
fn explore(action: &crate::cli::ExploreAction) -> Result<(), CliError> {
    use crate::cli::ExploreAction as A;
    let (id, status) = match action {
        A::Check(args) => {
            let files = spec_inputs(&args.paths, Some(Path::new("design/explore")))?;
            let mut errors = 0;
            for f in &files {
                let name = f.display().to_string();
                let text = std::fs::read_to_string(f).with_context(|| format!("reading {name}"))?;
                if let Err(ds) = bridle_spec::explore::parse_str(&name, &text) {
                    errors += ds.len();
                    ds.iter().for_each(|d| println!("{d}"));
                }
            }
            println!("{} file(s) checked: {errors} error(s)", files.len());
            if errors > 0 {
                return Err(anyhow::anyhow!("explore check found {errors} error(s)").into());
            }
            return Ok(());
        }
        A::New(a) => (&a.id, None),
        A::Conclude(a) => (&a.id, Some("concluded")),
        A::Abandon(a) => (&a.id, Some("abandoned")),
    };
    if id.is_empty() || id.contains(['/', '\\']) || id.starts_with('.') {
        return Err(anyhow::anyhow!("invalid exploration id '{id}'").into());
    }
    let path = Path::new("design/explore").join(id).join("findings.md");
    let Some(status) = status else {
        if path.exists() {
            return Err(anyhow::anyhow!("{} already exists", path.display()).into());
        }
        std::fs::create_dir_all(path.parent().expect("has a parent"))
            .with_context(|| format!("creating {}", path.display()))?;
        std::fs::write(&path, bridle_spec::explore::scaffold(id))
            .with_context(|| format!("writing {}", path.display()))?;
        println!("{}", path.display());
        return Ok(());
    };
    let name = path.display().to_string();
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {name}"))?;
    match bridle_spec::explore::set_status(&name, &text, status) {
        Ok(out) => std::fs::write(&path, out).with_context(|| format!("writing {name}"))?,
        Err(ds) => {
            ds.iter().for_each(|d| eprintln!("{d}"));
            return Err(anyhow::anyhow!("{name} does not check; not changing its status").into());
        }
    }
    println!("{name}: status {status}");
    Ok(())
}

/// `bridle spec export`: local, except `--task`, which asks the daemon for the task's
/// declared impact (docs/design/specs-to-tests.md).
async fn spec_export(cli: &Cli, args: &SpecExportArgs) -> Result<(), CliError> {
    let files = spec_inputs(&args.paths, args.root.as_deref())?;

    let mut selection: Option<HashSet<String>> = None;
    if !args.scenario.is_empty() {
        selection = Some(args.scenario.iter().cloned().collect());
    }
    if let Some(task) = &args.task {
        let impact = client_for_read(cli).await?.get_task(task).await?.impact;
        if impact.is_empty() {
            return Err(anyhow::anyhow!("{task} declares no impact; nothing to select").into());
        }
        selection.get_or_insert_with(HashSet::new).extend(
            impact
                .modify
                .into_iter()
                .chain(impact.add_under)
                .chain(impact.remove),
        );
    }

    let mut specs = Vec::new();
    let mut diagnostics = Vec::new();
    for file in &files {
        match bridle_spec::parse_file(file) {
            Ok(mut spec) => {
                if let Some(ids) = &selection {
                    crate::spec_export::select(&mut spec, ids);
                    if spec.requirements.is_empty() {
                        continue;
                    }
                }
                specs.push((file, spec));
            }
            Err(bridle_spec::ParseFileError::Diagnostics(ds)) => diagnostics.extend(ds),
            Err(e) => return Err(anyhow::Error::new(e).into()),
        }
    }
    if !diagnostics.is_empty() {
        for d in &diagnostics {
            eprintln!("{d}");
        }
        return Err(
            anyhow::anyhow!("not exporting: {} spec diagnostic(s)", diagnostics.len()).into(),
        );
    }

    let write = |dir: &Path, name: &str, text: &str| -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        let path = dir.join(name);
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    };
    match args.format {
        SpecFormatArg::Gherkin => {
            let dir = args
                .out
                .clone()
                .unwrap_or_else(|| PathBuf::from(".bridle/cache/features"));
            let mut written = Vec::new();
            for (file, spec) in &specs {
                let cap = crate::spec_export::capability(file);
                let text = crate::spec_export::gherkin(&cap, spec);
                written.push(
                    write(&dir, &format!("{cap}.feature"), &text)?
                        .display()
                        .to_string(),
                );
            }
            if cli.json {
                render::print_json(&written)?;
            } else {
                for p in &written {
                    println!("{p}");
                }
            }
        }
        SpecFormatArg::Json => {
            let doc = crate::spec_export::JsonExport {
                version: crate::spec_export::JSON_VERSION,
                specs: specs
                    .iter()
                    .map(|(f, s)| crate::spec_export::json_spec(f, s))
                    .collect(),
            };
            match &args.out {
                Some(dir) => {
                    let text = serde_json::to_string_pretty(&doc).context("encoding json")?;
                    let path = write(dir, "specs.json", &format!("{text}\n"))?;
                    println!("{}", path.display());
                }
                None => render::print_json(&doc)?,
            }
        }
    }
    Ok(())
}
