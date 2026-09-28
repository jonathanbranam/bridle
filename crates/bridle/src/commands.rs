//! Dispatch and implementation for every subcommand except `serve` (see
//! `serve.rs`). See docs/design/cli.md.

use std::path::Path;

use anyhow::Context;
use bridle_api::discovery::{self, Env, ProcessEnv};
use bridle_api::{
    BudgetHoldRequest, BudgetOverrideRequest, Client, DropTaskRequest, Edge, EdgeKind,
    EditTaskRequest, Event, EventQuery, InterruptRequest, MessageKind, MessageQuery,
    NewEdgeRequest, NewTaskRequest, RemoveEdgeQuery, RemoveQuery, RenewRequest, ResumeRequest,
    SendRequest, SpawnRequest, StopRequest, Task, TaskKind, TokenCreateRequest,
    UsageBreakdownQuery, UsageGroupBy, Workdir,
};
use chrono::{Local, TimeZone, Utc};
use futures::StreamExt;

use crate::cli::{
    AgentsArgs, AnswerArgs, AskArgs, BudgetAction, BudgetArgs, BudgetHoldArgs, ClaimArgs, Cli,
    Command, CostAction, CostArgs, CostAuditArgs, DepAction, DepArgs, DepEdgeArgs, EdgeKindArg,
    EventsArgs, InboxArgs, InterruptArgs, LogsArgs, PrimeArgs, PrimeRoleArg, ReadyArgs,
    ReleaseArgs, RmArgs, RulesAction, RulesArgs, RulesDiffArgs, RulesExplainArgs, SendArgs,
    ShowArgs, SpawnArgs, StopArgs, TaskAction, TaskArgs, TaskDropArgs, TaskEditArgs, TaskKindArg,
    TaskListArgs, TaskNewArgs, TaskNoteArgs, TaskReopenArgs, TaskShowArgs, TokenAction, TokenArgs,
    UsageArgs, UsageByArg, WhenArg,
};
use crate::error::CliError;
use crate::render;
use crate::serve;

pub async fn run(cli: Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Serve(args) => serve::run(&cli, args).await,
        Command::StopDaemon => stop_daemon(&cli).await,
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
        Command::Usage(args) => usage(&cli, args).await,
        Command::Cost(args) => cost(&cli, args).await,
        Command::Tui => tui(&cli).await,
        Command::Budget(args) => budget(&cli, args).await,
        Command::Token(args) => token(&cli, args).await,
        Command::Task(args) => task(&cli, args).await,
        Command::Dep(args) => dep(&cli, args).await,
        Command::Ask(args) => ask(&cli, args).await,
        Command::Answer(args) => answer(&cli, args).await,
        Command::Claim(args) => claim(&cli, args).await,
        Command::Release(args) => release(&cli, args).await,
        Command::Ready(args) => ready(&cli, args).await,
        Command::Statusline => statusline(&cli).await,
        Command::StopCheck => stop_check(&cli).await,
        Command::Prime(args) => prime(args).await,
        Command::Rules(args) => rules(&cli, args).await,
    }
}

/// Resolve the daemon endpoint and a token, per docs/design/agent-host/daemon.md and
/// principals.md. `resolve_endpoint` failing to find any daemon at all is exactly the
/// "daemon unreachable" case (exit 3).
async fn resolve_endpoint_and_token(cli: &Cli) -> Result<(String, Option<String>), CliError> {
    let cwd = std::env::current_dir().context("current directory")?;
    let env = ProcessEnv;
    let endpoint =
        discovery::resolve_endpoint(cli.url.as_deref(), cli.project.as_deref(), &cwd, &env)
            .map_err(|e| CliError::Unreachable(e.to_string()))?;
    let token =
        discovery::resolve_token(cli.token.as_deref(), endpoint.workspace.as_deref(), &env)?;
    Ok((endpoint.url, token))
}

async fn client_for(cli: &Cli) -> Result<Client, CliError> {
    let (url, token) = resolve_endpoint_and_token(cli).await?;
    Ok(Client::new(url, token))
}

/// Claude Code's statusLine command (docs/design/cli.md, docs/design/usage-and-budget.md
/// "Where bridle can see usage"). Reads Claude Code's JSON from stdin and prints a short
/// line back. Purely local: no daemon call, so unparseable stdin is the only way this
/// can produce a degraded line, never a slow or failed one. It no longer records a
/// snapshot with the daemon (dropped per s8kn's scope change: the context governor
/// gets account-wide windows from `get_usage` instead, and `POST /v1/statusline` /
/// `interactive_usage` stay in the daemon unused for now, not removed).
async fn statusline(_cli: &Cli) -> Result<(), CliError> {
    let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    let report = crate::statusline::parse(&input);
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
/// `decision`/`reason` JSON) only for the first claim missing one.
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
    let Ok(client) = client_for(cli).await else {
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
async fn prime(args: &PrimeArgs) -> Result<(), CliError> {
    match args.role {
        PrimeRoleArg::Orchestrator => prime_orchestrator().await,
    }
}

async fn prime_orchestrator() -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let role_prompt = std::fs::read_to_string(repo.join(".bridle/roles/orchestrator.md"))
        .context("reading .bridle/roles/orchestrator.md")?;
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
    let client = client_for(cli).await?;
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
        for (state, count) in &status.agents_by_state {
            println!("  {state:<10} {count}");
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
        ignore_budget: args.ignore_budget,
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
    let client = client_for(cli).await?;
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
    let client = client_for(cli).await?;
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

async fn send(cli: &Cli, args: &SendArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = match (&args.text, &args.text_file) {
        (Some(t), _) => t.clone(),
        (None, Some(f)) => {
            if f.to_string_lossy() == "-" {
                std::io::read_to_string(std::io::stdin()).context("reading from stdin")?
            } else {
                std::fs::read_to_string(f).with_context(|| format!("reading {}", f.display()))?
            }
        }
        (None, None) => {
            return Err(CliError::from(anyhow::anyhow!(
                "specify TEXT or --text-file"
            )));
        }
    };
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
    let client = client_for(cli).await?;
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
            println!("{} [{kind}] from {}: {}", m.id, m.from, m.body);
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
    let client = client_for(cli).await?;
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
    let client = client_for(cli).await?;
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

async fn tui(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    bridle_tui::run(client).await?;
    Ok(())
}

async fn usage(cli: &Cli, args: &UsageArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;

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
                    .map(|r| format!(", resets {}", r.format("%Y-%m-%d %H:%M UTC")))
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

fn resolve_workflow_rules(repo: &Path) -> Result<bridle_daemon::rules::Resolution, CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::rules;

    let config = Config::load(repo).context("loading .bridle/config.toml")?;
    let workflow_root = config.workflow.as_deref().map(Path::new);
    let layers = rules::discover_layers(repo, workflow_root, &config.packs);
    rules::load_and_resolve(&layers)
        .map_err(|e| CliError::from(anyhow::Error::new(e).context("resolving workflow rules")))
}

async fn rules_explain(cli: &Cli, args: &RulesExplainArgs) -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let resolution = resolve_workflow_rules(&repo)?;
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
    if !args.project_layer {
        return Err(anyhow::anyhow!("rules diff needs a mode: --project-layer").into());
    }
    let repo = std::env::current_dir().context("current directory")?;
    let resolution = resolve_workflow_rules(&repo)?;
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

async fn budget(cli: &Cli, args: &BudgetArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
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
    };
    if cli.json {
        render::print_json(&budget)?;
    } else {
        println!("state  {}", budget.state);
        if let Some(hold) = &budget.human_hold {
            let until = hold
                .until
                .map(|u| format!(", until {}", u.format("%Y-%m-%d %H:%M UTC")))
                .unwrap_or_else(|| ", until released".to_string());
            println!("hold   in force{until}");
        }
        if let Some(ov) = &budget.schedule_override {
            let period = ov.period.as_deref().unwrap_or("default");
            let until = ov
                .until
                .map(|u| format!(", until {}", u.format("%Y-%m-%d %H:%M UTC")))
                .unwrap_or_else(|| ", until cleared".to_string());
            println!("override {period}{until}");
        }
        for w in &budget.windows {
            let resets = w
                .resets_at
                .map(|r| format!(", resets {}", r.format("%Y-%m-%d %H:%M UTC")))
                .unwrap_or_default();
            let staleness = if w.stale { " (stale)" } else { "" };
            println!(
                "  {:<16} {:<12} {}{resets}{staleness}",
                w.window,
                w.state.to_string(),
                format_utilization(w.utilization)
            );
        }
        let t = &budget.thresholds;
        println!(
            "max_workers {}, wind_down_grace {}s, max_staleness {}s",
            t.max_workers, t.wind_down_grace_secs, t.max_staleness_secs
        );
    }
    Ok(())
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

async fn token(cli: &Cli, args: &TokenArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    match &args.action {
        TokenAction::Create { name } => {
            let created = client
                .create_token(&TokenCreateRequest { name: name.clone() })
                .await?;
            if cli.json {
                render::print_json(&created)?;
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
        TaskAction::Drop(a) => task_drop(cli, a).await,
        TaskAction::Reopen(a) => task_reopen(cli, a).await,
        TaskAction::Note(a) => task_note(cli, a).await,
    }
}

fn print_task_row(t: &Task) {
    println!("{:<10} {:<9} {:<8} {}", t.id, t.kind, t.state, t.title);
}

async fn task_new(cli: &Cli, args: &TaskNewArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let req = NewTaskRequest {
        title: args.title.clone(),
        kind: task_kind_arg(args.kind),
        body: args.body.clone().unwrap_or_default(),
    };
    let task = client.new_task(&req).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        print_task_row(&task);
    }
    Ok(())
}

async fn task_show(cli: &Cli, args: &TaskShowArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let task = client.get_task(&args.task).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("id          {}", task.id);
        println!("title       {}", task.title);
        println!("kind        {}", task.kind);
        println!("state       {}", task.state);
        println!("created     {}", task.created_at.to_rfc3339());
        println!("updated     {}", task.updated_at.to_rfc3339());
        if !task.body.is_empty() {
            println!();
            println!("{}", task.body);
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
    let req = EditTaskRequest {
        title: args.title.clone(),
        body: args.body.clone(),
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
    let client = client_for(cli).await?;
    let tasks = match &args.claimed_by {
        Some(claimed_by) => client.list_tasks_claimed_by(claimed_by).await?,
        None => client.list_tasks().await?,
    };
    if cli.json {
        render::print_json(&tasks)?;
    } else if tasks.is_empty() {
        println!("no tasks");
    } else {
        println!("{:<10} {:<9} {:<8} TITLE", "ID", "KIND", "STATE");
        for t in &tasks {
            print_task_row(t);
        }
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
    let task = client.note_task(&args.task, &args.text).await?;
    if cli.json {
        render::print_json(&task)?;
    } else {
        println!("noted on {}", task.id);
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
            "{:<16} {:<10} {:<9} {:<8} {}",
            p, t.id, t.kind, t.state, t.title
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
        let client = client_for(cli).await?;
        let tasks = client.ready_tasks().await?;
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
            &ProcessEnv,
        )
        .ok()
        .flatten();
        let client =
            Client::new_with_timeout(info.url.clone(), token, std::time::Duration::from_secs(5));
        if let Ok(tasks) = client.ready_tasks().await {
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
