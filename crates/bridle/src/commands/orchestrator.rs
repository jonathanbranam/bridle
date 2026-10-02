//! Orchestrator commands: prime, handover, wait-for-wake, mail.

use super::*;

/// The orchestrator's startup steps, printed by `bridle prime orchestrator`
/// alongside the role prompt and current state (docs/tickets/open/
/// one-command-orchestrator-handover-d4mz.md). Kept in the binary, not
/// `scripts/claude-orchestrator`, so there's one source of truth for what a
/// fresh orchestrator session does first; the script just runs `bridle prime
/// orchestrator` for its opening prompt.
pub(super) const ORCHESTRATOR_STARTUP_STEPS: &str = "\
Check in: `bridle status`, `bridle agents`, and recent messages to human (from the \
managers).
The human's open to-dos, highest priority first: `bridle task list --claimed-by human`.
Start the watcher from the latest event seq.
Keep the workforce's work moving, verify what gets merged, and bring the human only \
what needs them.
Watch your own context: hand over well before 200K.
The human will mostly reach you through Remote Control.";

/// `bridle prime orchestrator`: a fresh orchestrator session's opening
/// context in one command (docs/tickets/open/
/// one-command-orchestrator-handover-d4mz.md, step 2), read from the current
/// directory — run this from the repo root, as
/// `scripts/claude-orchestrator` does. Purely local: no daemon call.
pub(super) async fn prime(cli: &Cli, args: &PrimeArgs) -> Result<(), CliError> {
    match args.role {
        PrimeRoleArg::Orchestrator => prime_orchestrator(cli).await,
        PrimeRoleArg::Advisor => prime_advisor(),
        PrimeRoleArg::Worker => prime_scoped(cli, args, "worker", "worker").await,
        PrimeRoleArg::Planner => prime_scoped(cli, args, "product-manager", "planner").await,
    }
}

/// Worker/planner prime: rules, facts, guides and component scope (`--component`, else
/// the agent's own `BRIDLE_COMPONENTS`). Prime is otherwise orchestrator-only; these two
/// roles each get their own view.
pub(super) async fn prime_scoped(
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

/// Best-effort: the newest handover note comes from the daemon, and without one (or without
/// a daemon) the state file's pointer stands in, when the project has one
/// (orchestrator-supervision.md, section 7). The generic role file is followed by the
/// project's own `.bridle/roles/orchestrator.md`, when present.
pub(super) async fn prime_orchestrator(cli: &Cli) -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let config =
        bridle_daemon::config::Config::load(&repo).context("loading .bridle/config.toml")?;
    let workflow = config
        .workflow_root(&repo)
        .map_err(anyhow::Error::new)?
        .unwrap_or_else(|| repo.join("workflow"));
    let role_prompt = std::fs::read_to_string(workflow.join("base/roles/orchestrator.md"))
        .with_context(|| format!("reading {}/base/roles/orchestrator.md", workflow.display()))?;
    let project_part = std::fs::read_to_string(repo.join(".bridle/roles/orchestrator.md")).ok();
    let state = std::fs::read_to_string(repo.join("docs/context/orchestrator-state.md")).ok();
    let note = match client_for_read(cli).await {
        Ok(c) => c.latest_handover().await.ok().flatten(),
        Err(_) => None,
    };
    let project = crate::launchd::project_name(cli, &repo);
    print!(
        "{}",
        render_prime_orchestrator(
            &project,
            &role_prompt,
            project_part.as_deref(),
            state.as_deref(),
            note.as_ref(),
            chrono::Utc::now()
        )
    );
    Ok(())
}

/// The advisor's role file from the resolved workflow, then the project's own
/// `.bridle/roles/advisor.md` when present. Local: the advisor session's opening prompt.
pub(super) fn prime_advisor() -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let config =
        bridle_daemon::config::Config::load(&repo).context("loading .bridle/config.toml")?;
    let workflow = config
        .workflow_root(&repo)
        .map_err(anyhow::Error::new)?
        .unwrap_or_else(|| repo.join("workflow"));
    let path = workflow.join("base/roles/advisor.md");
    print!(
        "{}",
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?
    );
    if let Ok(part) = std::fs::read_to_string(repo.join(".bridle/roles/advisor.md")) {
        print!("\n{part}");
    }
    Ok(())
}

pub(super) fn note_age(d: chrono::Duration) -> String {
    match d.num_minutes() {
        m if m < 60 => format!("{}m", m.max(0)),
        m if m < 48 * 60 => format!("{}h", m / 60),
        m => format!("{}d", m / (24 * 60)),
    }
}

/// `{project}` in the role file is the current project's name (its credentials entry).
pub(super) fn render_prime_orchestrator(
    project: &str,
    role_prompt: &str,
    project_part: Option<&str>,
    state: Option<&str>,
    note: Option<&bridle_api::Handover>,
    now: chrono::DateTime<chrono::Utc>,
) -> String {
    let current = match note {
        Some(h) => format!(
            "# Handover note ({}, written {} ago by {})\n\n{}\n\n(Older notes: `bridle handover list`. The live state is in the startup steps' commands.)",
            h.id,
            note_age(now - h.created_at),
            h.created_by,
            h.body.trim_end(),
        ),
        None => format!(
            "# Current state (no handover note recorded yet)\n\n{}",
            state.map_or("(no state file either)", str::trim_end)
        ),
    };
    let mut role = role_prompt.trim_end().replace("{project}", project);
    if let Some(part) = project_part {
        role.push_str("\n\n");
        role.push_str(part.trim_end());
    }
    format!(
        "# Role: orchestrator\n\n{role}\n\n{current}\n\n# Startup steps\n\n{ORCHESTRATOR_STARTUP_STEPS}\n"
    )
}

/// Bridle's own counts for the human (docs/tickets/resolved/
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
pub(super) async fn bridle_counts(cwd: &Path, env: &impl Env, token_path: &Path) -> Option<String> {
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

/// `bridle wait-for-wake`: the daemon holds the request until a wake is pending.
/// `bridle mail run`: the mail bridge for this daemon's project, until interrupted.
pub(super) async fn mail_run(cli: &Cli) -> Result<(), CliError> {
    let cwd = std::env::current_dir().context("current directory")?;
    let endpoint = discovery::resolve_endpoint(
        cli.url.as_deref(),
        cli.project.as_deref(),
        &cwd,
        &ProcessEnv,
    )
    .map_err(|e| CliError::Unreachable(e.to_string()))?;
    let (Some(project), Some(workspace)) = (endpoint.project.clone(), endpoint.workspace.clone())
    else {
        return Err(
            anyhow::anyhow!("mail run needs --project (or a workspace), not just --url").into(),
        );
    };
    let cfg = bridle_mail::MailConfig::load()?;
    let store =
        bridle_mail::S3Store::connect(&cfg.bucket, &cfg.prefix, cfg.region.as_deref()).await;
    let state = discovery::state_dir(&workspace);
    let attachments = state.join("inbox").join("mail");
    let client = client_for(cli).await?;
    let mailer = bridle_mail::SesMailer::connect(cfg.region.as_deref()).await;
    let tokens = bridle_mail::Tokens::load_or_create(&discovery::bridle_home().join("mail.key"))?;
    let sent = bridle_mail::Sent::open(&state.join("mail"))?;
    let sink = std::sync::Arc::new(bridle_mail::ClientSink(client));
    let advisor_pid = format!("advisor-{project}.pid");
    bridle_mail::Bridge::new(
        std::sync::Arc::new(store),
        sink.clone(),
        cfg,
        project,
        attachments,
    )
    .with_local(std::sync::Arc::new(bridle_mail::FileLocal {
        owner_file: state.join("state").join("owner.toml"),
        advisor_pid_file: discovery::bridle_home().join(advisor_pid),
        host: local_hostname(),
    }))
    .with_outbound(bridle_mail::Outbound {
        mailer: std::sync::Arc::new(mailer),
        feed: sink,
        tokens,
        sent,
    })
    .run()
    .await?;
    Ok(())
}

/// This machine's name as `bridle serve` records it in `owner.toml`.
pub(super) fn local_hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Unread mail from the email bridge, for `wait-for-wake --mail`.
pub(super) fn unread_mail(
    messages: Vec<bridle_api::types::Message>,
) -> Vec<bridle_api::types::Message> {
    messages
        .into_iter()
        .filter(|m| m.from == "external:mail")
        .collect()
}

/// `bridle wait-for-wake --mail`: the advisor's mail-only waiter. Polls its own inbox for mail
/// from the bridge and prints it; `nothing` after 25 minutes, like the orchestrator's waiter.
pub(super) async fn wait_for_mail(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let query = MessageQuery {
        to: Some("me".to_string()),
        unread: true,
        ..Default::default()
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(25 * 60);
    loop {
        let mail = unread_mail(client.list_messages(&query).await?);
        if !mail.is_empty() {
            if cli.json {
                render::print_json(&mail)?;
            } else {
                for m in &mail {
                    println!("{} from {}:\n{}", m.id, m.from, m.body);
                }
            }
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            println!("nothing");
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

pub(super) async fn wait_for_wake(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let wakes = client.orchestrator_wake().await?.wakes;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string(&wakes).map_err(anyhow::Error::from)?
        );
    } else if wakes.is_empty() {
        println!("nothing");
    } else {
        for w in &wakes {
            println!("{}: {}", w.reason, w.text);
            println!("{}", w.detail);
        }
    }
    Ok(())
}

/// `bridle handover done`: the marker only; the daemon stops and relaunches the session.
pub(super) async fn handover_done(cli: &Cli) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    let done = client.handover_done().await?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string(&done).map_err(anyhow::Error::from)?
        );
    } else {
        println!("handover marked done; the session will be relaunched shortly");
    }
    Ok(())
}

/// `bridle handover write|done|list|show`.
pub(super) async fn handover(cli: &Cli, args: &HandoverArgs) -> Result<(), CliError> {
    match &args.action {
        HandoverAction::Write { file } => {
            let body = if file.to_string_lossy() == "-" {
                std::io::read_to_string(std::io::stdin()).context("reading from stdin")?
            } else {
                std::fs::read_to_string(file)
                    .with_context(|| format!("reading {}", file.display()))?
            };
            let h = client_for(cli).await?.write_handover(&body).await?;
            if cli.json {
                render::print_json(&h)?;
            } else {
                println!("{}", h.id);
            }
        }
        HandoverAction::Done => handover_done(cli).await?,
        HandoverAction::List => {
            let list = client_for_read(cli).await?.list_handovers().await?;
            if cli.json {
                render::print_json(&list)?;
            } else if list.is_empty() {
                println!("no handover notes");
            } else {
                for h in &list {
                    let first = h.body.lines().next().unwrap_or("");
                    println!(
                        "{}  {}  {:<22} {first}",
                        h.id,
                        h.created_at.format("%Y-%m-%d %H:%M"),
                        h.created_by
                    );
                }
            }
        }
        HandoverAction::Show { id } => {
            let h = client_for_read(cli).await?.get_handover(id).await?;
            if cli.json {
                render::print_json(&h)?;
            } else {
                println!("{} by {} at {}\n", h.id, h.created_by, h.created_at);
                println!("{}", h.body.trim_end());
            }
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
            "bridle",
            "You're my orchestrator for bridle.",
            None,
            Some("## Handover, 2026-09-28"),
            None,
            chrono::Utc::now(),
        );
        assert!(out.contains("You're my orchestrator for bridle."));
        assert!(out.contains("## Handover, 2026-09-28"));
        assert!(out.contains("bridle status"));
        assert!(out.contains("Start the watcher"));
        // Sections appear in a fixed, readable order.
        let role_pos = out.find("# Role: orchestrator").unwrap();
        let state_pos = out.find("# Current state").unwrap();
        let steps_pos = out.find("# Startup steps").unwrap();
        assert!(role_pos < state_pos);
        assert!(state_pos < steps_pos);
    }

    #[test]
    fn generic_role_file_has_no_bridle_repo_specifics_and_names_the_project() {
        let role = include_str!("../../../../workflow/base/roles/orchestrator.md");
        let out =
            render_prime_orchestrator("meta-notes", role, None, None, None, chrono::Utc::now());
        for s in [
            "cargo install",
            "role-notes",
            "role notes",
            "two managers",
            "GitHub Actions",
        ] {
            assert!(!out.contains(s), "{s}");
        }
        assert!(out.contains("$1==\"meta-notes\""), "{out}");
        assert!(!out.contains("{project}"));
        assert!(out.contains("(no state file either)"));
    }

    #[test]
    fn project_part_is_appended_after_the_role_file() {
        let out = render_prime_orchestrator(
            "p",
            "generic",
            Some("PROJECT PART\n"),
            None,
            None,
            chrono::Utc::now(),
        );
        assert!(out.find("generic").unwrap() < out.find("PROJECT PART").unwrap());
        assert!(out.find("PROJECT PART").unwrap() < out.find("# Startup steps").unwrap());
    }

    #[test]
    fn prints_the_note_with_its_age_instead_of_the_state_file() {
        let now = chrono::Utc::now();
        let note = bridle_api::Handover {
            id: "h-0007".into(),
            role: "orchestrator".into(),
            project: "bridle".into(),
            body: "Carry on with br-1.".into(),
            created_at: now - chrono::Duration::hours(3),
            created_by: "external:orchestrator".into(),
        };
        let out = render_prime_orchestrator(
            "bridle",
            "role",
            None,
            Some("the state file"),
            Some(&note),
            now,
        );
        assert!(out.contains("# Handover note (h-0007, written 3h ago"));
        assert!(out.contains("Carry on with br-1."));
        assert!(!out.contains("the state file"));
        assert!(out.find("# Handover note").unwrap() < out.find("# Startup steps").unwrap());
    }

    #[test]
    fn advisor_prime_includes_wake_command_for_unnamed_advisor() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("bridle agent wake external:advisor --timeout 300"),
            "advisor prime should include wake command for unnamed advisor"
        );
    }

    #[test]
    fn advisor_prime_includes_wake_command_for_named_advisor() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("bridle agent wake external:advisor/$BRIDLE_ADVISOR_NAME --timeout 300"),
            "advisor prime should include wake command for named advisor"
        );
    }

    #[test]
    fn advisor_prime_includes_inbox_and_loop_instructions() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("bridle inbox --json --mark-read"),
            "advisor prime should instruct reading inbox with --mark-read flag"
        );
        assert!(
            role.contains("loop back"),
            "advisor prime should mention looping"
        );
    }

    #[test]
    fn advisor_prime_includes_mark_read_instruction() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("bridle inbox --json --mark-read"),
            "advisor prime should instruct to mark messages read after acting"
        );
        assert!(
            role.contains("re-fires forever"),
            "advisor prime should explain why marking read is important"
        );
    }
}
