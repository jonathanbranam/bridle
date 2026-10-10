//! Agent commands: spawn, agents, show, interrupt, stop, resume, renew, rm, logs.

use super::*;

pub(super) async fn spawn(cli: &Cli, args: &SpawnArgs) -> Result<(), CliError> {
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
pub(super) fn format_context_tokens(tokens: Option<u64>) -> String {
    match tokens {
        Some(n) => format_tokens(n),
        None => "-".to_string(),
    }
}

pub(super) fn print_agent(cli: &Cli, agent: &bridle_api::Agent) -> Result<(), CliError> {
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

pub(super) async fn agents(cli: &Cli, args: &AgentsArgs) -> Result<(), CliError> {
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

pub(super) async fn show(cli: &Cli, args: &ShowArgs) -> Result<(), CliError> {
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

pub(super) async fn interrupt(cli: &Cli, args: &InterruptArgs) -> Result<(), CliError> {
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

pub(super) async fn stop(cli: &Cli, args: &StopArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let agent = client
        .stop(&args.agent, &StopRequest { now: args.now })
        .await?;
    print_agent(cli, &agent)
}

pub(super) async fn resume(cli: &Cli, args: &crate::cli::ResumeArgs) -> Result<(), CliError> {
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

pub(super) async fn renew(cli: &Cli, args: &crate::cli::RenewArgs) -> Result<(), CliError> {
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

pub(super) async fn rm(cli: &Cli, args: &RmArgs) -> Result<(), CliError> {
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

/// `bridle agent wake <identifier>`: the daemon holds the request until it decides the
/// principal should wake. Exit 0 woken, 4 timed out, 5 superseded by a newer wait from this
/// session or stopped with `--stop`, 6 the daemon is restarting or stopping.
pub(super) async fn wake(cli: &Cli, args: &WakeArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    // The launcher's pid (`bridle session`); a bare shell has none and replaces nothing.
    let session = std::env::var("BRIDLE_SESSION_PID")
        .ok()
        .filter(|s| !s.is_empty());
    if args.stop {
        let stopped = client
            .stop_wake(&bridle_api::types::StopWakeRequest {
                principal: args.identifier.clone(),
                session: session.clone().filter(|_| args.identifier.is_none()),
            })
            .await?
            .stopped;
        if cli.json {
            render::print_json(&serde_json::json!({"stopped": stopped}))?;
        } else if stopped == 0 {
            println!("no wait open");
        } else {
            println!("stopped {stopped} wait(s)");
        }
        return Ok(());
    }
    let identifier = args.identifier.clone().unwrap_or_default();
    // On stderr so `--json` output stays clean.
    eprintln!(
        "waiting as {identifier} (pid {}, timeout {} s)",
        std::process::id(),
        args.timeout.map_or(6900, |t| t.min(6900))
    );
    let got = client
        .principal_wake(&bridle_api::types::PrincipalWakeQuery {
            principal: identifier.clone(),
            timeout_secs: args.timeout,
            session,
            pid: Some(std::process::id()),
        })
        .await?;
    if got
        .reasons
        .iter()
        .any(|r| r.reason == bridle_api::types::WAIT_SUPERSEDED_WAKE)
    {
        if cli.json {
            render::print_json(&got)?;
        }
        return Err(CliError::Superseded(
            "superseded by a newer wait (or stopped); nothing was marked read".to_string(),
        ));
    }
    let stopping = got
        .reasons
        .iter()
        .find(|r| r.reason == bridle_api::types::DAEMON_STOPPING_WAKE);
    if let Some(r) = stopping {
        if cli.json {
            render::print_json(&got)?;
        }
        return Err(CliError::Stopping(format!(
            "daemon {}; re-arm once it is back",
            r.text.as_deref().unwrap_or("restarting or shutting down")
        )));
    }
    if got.reasons.is_empty() {
        if cli.json {
            println!("{{\"reasons\":[]}}");
        }
        return Err(CliError::Timeout(format!(
            "nothing woke {identifier} before the timeout"
        )));
    }
    if cli.json {
        render::print_json(&got)?;
    } else {
        for r in &got.reasons {
            match (&r.task, &r.event) {
                (Some(task), Some(event)) => println!("{}: {task} {event}", r.reason),
                _ if !r.messages.is_empty() => {
                    for m in &r.messages {
                        println!("message {} from {}:\n{}", m.id, m.from, m.body);
                    }
                }
                // The orchestrator's reasons: the text, then the raw fact, as wait-for-wake does.
                _ if r.detail.is_some() => {
                    println!("{}: {}", r.reason, r.text.as_deref().unwrap_or_default());
                    println!("{}", r.detail.as_ref().unwrap_or(&serde_json::Value::Null));
                }
                _ => println!("{}: {}", r.reason, r.message_ids.join(" ")),
            }
        }
    }
    Ok(())
}

pub(super) async fn logs(cli: &Cli, args: &LogsArgs) -> Result<(), CliError> {
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

pub(super) fn print_transcript_line(line: &bridle_api::TranscriptLine, raw: bool) {
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
