//! Dispatch and implementation for every subcommand except `serve` (see
//! `serve.rs`). See docs/design/cli.md.

use anyhow::Context;
use bridle_api::discovery::{self, ProcessEnv};
use bridle_api::{
    BudgetHoldRequest, Client, DropTaskRequest, EditTaskRequest, Event, EventQuery,
    InterruptRequest, MessageKind, MessageQuery, NewTaskRequest, RemoveQuery, ResumeRequest,
    SendRequest, SpawnRequest, StopRequest, Task, TaskKind, TokenCreateRequest,
    UsageBreakdownQuery, UsageGroupBy, Workdir,
};
use chrono::{Local, TimeZone, Utc};
use futures::StreamExt;

use crate::cli::{
    AgentsArgs, BudgetAction, BudgetArgs, BudgetHoldArgs, Cli, Command, CostAction, CostArgs,
    CostAuditArgs, EventsArgs, InboxArgs, InterruptArgs, LogsArgs, RmArgs, SendArgs, ShowArgs,
    SpawnArgs, StopArgs, TaskAction, TaskArgs, TaskDropArgs, TaskEditArgs, TaskKindArg,
    TaskNewArgs, TaskReopenArgs, TaskShowArgs, TokenAction, TokenArgs, UsageArgs, UsageByArg,
    WhenArg,
};
use crate::error::CliError;
use crate::render;
use crate::serve;

pub async fn run(cli: Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Serve(args) => serve::run(&cli, args).await,
        Command::StopDaemon => stop_daemon(&cli).await,
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
        Command::Rm(args) => rm(&cli, args).await,
        Command::Logs(args) => logs(&cli, args).await,
        Command::Events(args) => events(&cli, args).await,
        Command::Usage(args) => usage(&cli, args).await,
        Command::Cost(args) => cost(&cli, args).await,
        Command::Tui => tui(&cli).await,
        Command::Budget(args) => budget(&cli, args).await,
        Command::Token(args) => token(&cli, args).await,
        Command::Task(args) => task(&cli, args).await,
        Command::Statusline => statusline(&cli).await,
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

/// How long `bridle statusline` waits on the daemon before giving up. Claude
/// Code runs this on every render of the human's prompt, so it must never be
/// what makes the terminal feel slow (docs/design/usage-and-budget.md).
const STATUSLINE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Claude Code's statusLine command (docs/design/cli.md, docs/design/usage-and-budget.md
/// "Where bridle can see usage"). Reads Claude Code's JSON from stdin, prints a short
/// line back, and best-effort records a snapshot with the daemon. Never returns an
/// error and never blocks beyond `STATUSLINE_TIMEOUT`: an unreachable daemon, no
/// project daemon at all, a slow response, or unparseable stdin all just mean nothing
/// gets recorded, not a failed or slow status line.
async fn statusline(cli: &Cli) -> Result<(), CliError> {
    let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    let report = crate::statusline::parse(&input);
    println!("{}", crate::statusline::render_line(&report));

    if let Ok((url, token)) = resolve_endpoint_and_token(cli).await {
        let client = Client::new_with_timeout(url, token, STATUSLINE_TIMEOUT);
        let _ = tokio::time::timeout(STATUSLINE_TIMEOUT, client.report_statusline(&report)).await;
    }
    Ok(())
}

async fn stop_daemon(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    client.shutdown().await?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if client.health().await.is_err() {
            println!("daemon stopped");
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
            println!("  {:<10} {}", rl.window, format_utilization(rl.utilization));
        }
    }
    Ok(())
}

fn format_utilization(u: Option<f64>) -> String {
    u.map(|u| format!("{:.0}%", u * 100.0))
        .unwrap_or_else(|| "-".to_string())
}

async fn spawn(cli: &Cli, args: &SpawnArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let prompt = match (&args.prompt, &args.prompt_file) {
        (Some(p), _) => Some(p.clone()),
        (None, Some(f)) => {
            Some(std::fs::read_to_string(f).with_context(|| format!("reading {}", f.display()))?)
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
        ignore_budget: args.ignore_budget,
    };
    let agent = client.spawn(&req).await?;
    print_agent(cli, &agent)
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
            "{:<12} {:<14} {:<10} {:<9} {:<10} {:>5} {:>9}",
            "ID", "NAME", "ROLE", "STATE", "MODEL", "TURNS", "COST"
        );
        for a in &list {
            println!(
                "{:<12} {:<14} {:<10} {:<9} {:<10} {:>5} {:>9.4}",
                a.id,
                a.name,
                a.role,
                a.state.to_string(),
                a.model,
                a.turns,
                a.cost_usd_total
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
        println!("cost        ${:.4}", agent.cost_usd_total);
        println!("held msgs   {}", agent.held_messages);
        println!("unacked     {}", agent.unacked_messages);
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
    let req = SendRequest {
        to: Some(args.to.clone()),
        body: args.text.clone(),
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
    let msg = client.send(&req).await?;
    if cli.json {
        render::print_json(&msg)?;
    } else {
        println!("sent {} -> {}", msg.id, msg.to);
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
    if cli.json {
        render::print_json(&messages)?;
    } else if messages.is_empty() {
        println!("inbox empty");
    } else {
        for m in &messages {
            let kind = serde_json::to_value(m.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            println!("{} [{kind}] from {}: {}", m.id, m.from, m.body);
        }
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
            println!(
                "{:<20} {:>5} {:>12} {:>9} {:>8}",
                "KEY", "TURNS", "TOKENS", "COST", "CACHE"
            );
            for g in &breakdown.groups {
                let tokens =
                    g.tokens.input + g.tokens.output + g.tokens.cache_read + g.tokens.cache_write;
                let cache = g
                    .cache_hit_ratio
                    .map(|r| format!("{:.1}%", r * 100.0))
                    .unwrap_or_else(|| "-".to_string());
                println!(
                    "{:<20} {:>5} {:>12} {:>9.4} {:>8}",
                    g.key, g.turns, tokens, g.cost_usd_total, cache
                );
            }
            let t = &breakdown.total_tokens;
            println!(
                "total: {} turns, {} tokens, ${:.4}",
                breakdown.total_turns,
                t.input + t.output + t.cache_read + t.cache_write,
                breakdown.total_cost_usd
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
            "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9}",
            "ID", "NAME", "ROLE", "TURNS", "TOKENS", "COST"
        );
        for a in &usage.agents {
            let tokens =
                a.tokens.input + a.tokens.output + a.tokens.cache_read + a.tokens.cache_write;
            let name = if a.removed {
                format!("{} (rm)", a.name)
            } else {
                a.name.clone()
            };
            println!(
                "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9.4}",
                a.agent, name, a.role, a.turns, tokens, a.cost_usd_total
            );
        }
        let t = &usage.total_tokens;
        println!(
            "total: {} turns, {} tokens, ${:.4}",
            usage.total_turns,
            t.input + t.output + t.cache_read + t.cache_write,
            usage.total_cost_usd
        );
        if let Some(ratio) = usage.cache_hit_ratio {
            println!("cache hit ratio: {:.1}%", ratio * 100.0);
        }
        for rl in &usage.rate_limits {
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
        if !usage.interactive_today.is_empty() {
            println!("today, interactive (bridle statusline):");
            for row in &usage.interactive_today {
                let cost = row
                    .cost_usd
                    .map(|c| format!("${c:.4}"))
                    .unwrap_or_else(|| "-".to_string());
                let ctx = match (row.context_used_tokens, row.context_max_tokens) {
                    (Some(u), Some(m)) if m > 0 => format!("{:.0}%", (u as f64 / m as f64) * 100.0),
                    _ => "-".to_string(),
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
        .map(|b| b.to_string())
        .unwrap_or_else(|| "-".to_string());
    let change = r
        .change_percent
        .map(|c| format!("{c:+.1}%"))
        .unwrap_or_else(|| "new".to_string());
    let flag = if r.over_threshold { " !" } else { "" };
    println!(
        "{:<14} {:>9} {:>9} {:>8}{flag}",
        r.role, r.current_tokens, baseline, change
    );
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
        return Ok(Some(target.with_timezone(&Utc)));
    }
    Ok(None)
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
        TaskAction::List => task_list(cli).await,
        TaskAction::Drop(a) => task_drop(cli, a).await,
        TaskAction::Reopen(a) => task_reopen(cli, a).await,
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

async fn task_list(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let tasks = client.list_tasks().await?;
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
