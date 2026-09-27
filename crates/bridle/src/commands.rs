//! Dispatch and implementation for every subcommand except `serve` (see
//! `serve.rs`). See docs/agent-host.md §6.3.

use anyhow::Context;
use bridle_api::discovery::{self, ProcessEnv};
use bridle_api::{
    Client, Event, EventQuery, InterruptRequest, MessageKind, MessageQuery, RemoveQuery,
    SendRequest, SpawnRequest, StopRequest, TokenCreateRequest, Workdir,
};
use futures::StreamExt;

use crate::cli::{
    AgentsArgs, Cli, Command, EventsArgs, InboxArgs, InterruptArgs, LogsArgs, RmArgs, SendArgs,
    ShowArgs, SpawnArgs, StopArgs, TokenAction, TokenArgs, WhenArg,
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
        Command::Usage => usage(&cli).await,
        Command::Token(args) => token(&cli, args).await,
    }
}

/// Resolve the daemon endpoint and a token, per docs/agent-host.md §3.1 and
/// §5.2, then build a client for it. `resolve_endpoint` failing to find any
/// daemon at all is exactly the "daemon unreachable" case (exit 3).
async fn client_for(cli: &Cli) -> Result<Client, CliError> {
    let cwd = std::env::current_dir().context("current directory")?;
    let env = ProcessEnv;
    let endpoint =
        discovery::resolve_endpoint(cli.url.as_deref(), cli.project.as_deref(), &cwd, &env)
            .map_err(|e| CliError::Unreachable(e.to_string()))?;
    let token =
        discovery::resolve_token(cli.token.as_deref(), endpoint.workspace.as_deref(), &env)?;
    Ok(Client::new(endpoint.url, token))
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

async fn daemons(cli: &Cli) -> Result<(), CliError> {
    let list = discovery::list_registry();
    if cli.json {
        render::print_json(&list)?;
    } else if list.is_empty() {
        println!("no daemons running");
    } else {
        let url_w = list.iter().map(|d| d.url.len()).max().unwrap_or(3);
        println!("{:<16} {:<8} {:<url_w$} WORKSPACE", "PROJECT", "PID", "URL");
        for d in &list {
            println!(
                "{:<16} {:<8} {:<url_w$} {}",
                d.project, d.pid, d.url, d.workspace
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
    let agent = client.resume(&args.agent).await?;
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

async fn usage(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
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
            println!(
                "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9.4}",
                a.agent, a.name, a.role, a.turns, tokens, a.cost_usd_total
            );
        }
        println!(
            "total: {} turns, ${:.4}",
            usage.total_turns, usage.total_cost_usd
        );
        if let Some(ratio) = usage.cache_hit_ratio {
            println!("cache hit ratio: {:.1}%", ratio * 100.0);
        }
        for rl in &usage.rate_limits {
            println!("{:<10} {}", rl.window, format_utilization(rl.utilization));
        }
    }
    Ok(())
}

async fn token(cli: &Cli, args: &TokenArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let TokenAction::Create { name } = &args.action;
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
    Ok(())
}
