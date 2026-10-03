//! Top-level commands: status, send, inbox, events, wait, tui, token, port, probe.

use super::*;

pub(super) async fn status(cli: &Cli) -> Result<(), CliError> {
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
        if let Some(line) = crate::focus::status_line(&discovery::bridle_home(), chrono::Utc::now())
        {
            println!("focus      {line}");
        }
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
        if let Some(sp) = &status.state_push {
            match (&sp.failing, sp.last_pushed_at) {
                (Some(why), _) if sp.diverged => println!("state      push stopped: {why}"),
                (Some(why), _) => println!("state      state push failing: {why}"),
                (None, Some(at)) => println!(
                    "state      pushed {}m ago",
                    (chrono::Utc::now() - at).num_minutes().max(0)
                ),
                (None, None) => println!("state      nothing pushed yet"),
            }
        }
        if let Some(sha) = &status.upgrade_waiting {
            println!(
                "upgrade    {sha} built-green, waiting for a quiet point; new worker spawns refused"
            );
        }
        for inc in &status.incidents {
            println!(
                "incident   {} {} ({}m)",
                inc.id,
                inc.title,
                (chrono::Utc::now() - inc.since).num_minutes().max(0)
            );
        }
        for sess in &status.sessions {
            println!(
                "session    {} {}",
                sess.identity,
                format_context_tokens(sess.tokens)
            );
        }
        let waiting = if status.waiter_open {
            "waiting"
        } else {
            "no waiter"
        };
        match status.last_wake_at {
            Some(at) => println!(
                "wake       last delivered {}m ago; {waiting}",
                (chrono::Utc::now() - at).num_minutes().max(0)
            ),
            None => println!("wake       none delivered yet; {waiting}"),
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

/// Message and note text must say something: an empty one lands in an
/// inbox as noise (hx7t). The daemon refuses it too; this fails before the call.
pub(super) fn require_body(body: String) -> Result<String, CliError> {
    if body.trim().is_empty() {
        return Err(CliError::from(anyhow::anyhow!(
            "message text must not be empty"
        )));
    }
    Ok(body)
}

pub(super) async fn send(cli: &Cli, args: &SendArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let body = require_body(read_text(&args.text, &args.text_file, "text")?)?;
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
            if let Some(name) = args
                .to
                .strip_prefix("external:")
                .and_then(|t| t.split_once('/'))
                && !msg.to.contains('/')
            {
                println!(
                    "{} isn't running; delivered to advisor",
                    name.1.split('@').next().unwrap_or(name.1)
                );
            }
        }
    }
    Ok(())
}

pub(super) async fn inbox(cli: &Cli, args: &InboxArgs) -> Result<(), CliError> {
    match &args.action {
        Some(InboxAction::Show(show_args)) => inbox_show(cli, show_args).await,
        Some(InboxAction::Read(read_args)) => inbox_read(cli, read_args).await,
        Some(InboxAction::Unread(args)) => inbox_unread(cli, args).await,
        None => inbox_list(cli, args).await,
    }
}

pub(super) async fn inbox_list(cli: &Cli, args: &InboxArgs) -> Result<(), CliError> {
    // `--mark-read` writes (POST /v1/messages/{id}/read), but inbox is
    // overwhelmingly a read command, so it gets the same anonymous-read
    // tolerance; `--mark-read` under `$CLAUDECODE` with no token still fails,
    // just from the daemon's 401 rather than a client-side check.
    let client = client_for_read(cli).await?;
    let query = MessageQuery {
        to: Some("me".to_string()),
        unread: !args.all,
        // The daemon marks what it lists read for an agent or external principal (what reaches
        // them is read) and ignores this for the human, whose reads are explicit.
        mark_read: true,
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

pub(super) async fn inbox_show(cli: &Cli, args: &InboxShowArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;
    let query = MessageQuery {
        to: Some("me".to_string()),
        id: Some(args.id.clone()),
        mark_read: true,
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

    if args.mark_read {
        client.mark_read(&message.id).await?;
    }

    Ok(())
}

pub(super) async fn inbox_read(cli: &Cli, args: &InboxReadArgs) -> Result<(), CliError> {
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

pub(super) async fn inbox_unread(cli: &Cli, args: &InboxReadArgs) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    for id in &args.ids {
        client.mark_unread(id).await?;
    }
    if cli.json {
        render::print_json(&serde_json::json!({
            "marked_unread": args.ids,
        }))?;
    } else {
        for id in &args.ids {
            println!("marked {} unread", id);
        }
    }
    Ok(())
}

pub(super) async fn events(cli: &Cli, args: &EventsArgs) -> Result<(), CliError> {
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

pub(super) fn matches(ev: &Event, args: &EventsArgs) -> bool {
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

pub(super) fn print_event(cli: &Cli, ev: &Event) -> Result<(), CliError> {
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
pub(super) async fn wait(cli: &Cli, args: &WaitArgs) -> Result<(), CliError> {
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

pub(super) fn wait_done(
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

pub(super) async fn tui(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    bridle_tui::run(client).await?;
    Ok(())
}

/// The project a `token create`/`revoke` acted on: `--project`, or the cwd's daemon.
pub(super) fn token_project(cli: &Cli) -> Result<Option<String>, CliError> {
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

pub(super) async fn token(cli: &Cli, args: &TokenArgs) -> Result<(), CliError> {
    let client = if matches!(args.action, TokenAction::List) {
        client_for_read(cli).await?
    } else {
        client_for(cli).await?
    };
    match &args.action {
        TokenAction::Create {
            name,
            machine,
            print,
        } => {
            let created = client
                .create_token(&TokenCreateRequest {
                    name: name.clone(),
                    machine: machine.clone(),
                })
                .await?;
            // Stored for the project this command talked to, so the token never has
            // to be seen; with no project (daemon found by URL) it's printed as before.
            // A visitor's token is for another machine's credentials.toml, so it is never
            // saved here.
            let project = if machine.is_some() {
                None
            } else {
                token_project(cli)?
            };
            let stored_in = match project {
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
                if *print {
                    println!("{}", created.token);
                }
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

pub(super) fn probe_line(r: &ProbeResult) -> String {
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

pub(super) async fn probe(cli: &Cli, args: &ProbeArgs) -> Result<(), CliError> {
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

pub(super) async fn port(cli: &Cli, args: &PortArgs) -> Result<(), CliError> {
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

#[cfg(test)]
mod require_body_tests {
    use super::require_body;

    #[test]
    fn rejects_empty_and_whitespace_only() {
        assert!(require_body(String::new()).is_err());
        assert!(require_body(" \n\t".to_string()).is_err());
        assert_eq!(require_body("hi".to_string()).unwrap(), "hi");
    }
}
