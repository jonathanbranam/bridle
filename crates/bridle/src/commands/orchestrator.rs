//! Orchestrator commands: prime, handover, wait-for-wake, mail.

use super::*;

/// The orchestrator's startup steps, printed by `bridle prime orchestrator`
/// alongside the role prompt and current state (docs/tickets/open/
/// one-command-orchestrator-handover-d4mz.md). Kept in the binary, so there's one source of truth
/// for what a fresh orchestrator session does first; `bridle session orchestrator` runs this.
pub(super) const ORCHESTRATOR_STARTUP_STEPS: &str = "\
Check in: `bridle status`, `bridle agents`, and recent messages to human (from the \
managers).
Pending tasks (`bridle task list --state pending`; `bridle status` counts them): every task \
starts pending. Bring the human the ones worth doing, and `bridle task ready <id>` those they \
approve.
The human's open to-dos, highest priority first: `bridle task list --claimed-by human`.
Start the watcher from the latest event seq.
Keep the workforce's work moving, verify what gets merged, and bring the human only \
what needs them.
Watch your own context: hand over well before 200K.
The human will mostly reach you through Remote Control.";

/// `bridle prime orchestrator`: a fresh orchestrator session's opening
/// context in one command (docs/tickets/open/
/// one-command-orchestrator-handover-d4mz.md, step 2), read from the current
/// directory — run this from the repo root. Purely local: no daemon call.
pub(super) async fn prime(cli: &Cli, args: &PrimeArgs) -> Result<(), CliError> {
    match args.role {
        PrimeRoleArg::Orchestrator => prime_orchestrator(cli).await,
        PrimeRoleArg::Advisor => prime_advisor(),
        PrimeRoleArg::Aide => prime_aide(),
        PrimeRoleArg::Prototyper => prime_prototyper(),
        PrimeRoleArg::Designer => prime_role_file("designer"),
        PrimeRoleArg::DocumentReviewer => prime_role_file("document-reviewer"),
        PrimeRoleArg::Worker => prime_scoped(cli, args, "worker", "worker").await,
        PrimeRoleArg::Planner => prime_scoped(cli, args, "project-manager", "planner").await,
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
    let mut project_part = std::fs::read_to_string(repo.join(".bridle/roles/orchestrator.md")).ok();
    let rules = rules_section(&config, &repo, "orchestrator");
    if !rules.is_empty() {
        let part = project_part.get_or_insert_with(String::new);
        if !part.is_empty() {
            part.push_str("\n\n");
        }
        part.push_str(&rules);
    }
    let state = std::fs::read_to_string(repo.join("docs/context/orchestrator-state.md")).ok();
    let note = match client_for_read(cli).await {
        Ok(c) => c.latest_handover(Some("orchestrator")).await.ok().flatten(),
        Err(_) => None,
    };
    let project = crate::launchd::project_name(cli, &repo)?;
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
    prime_role_file("advisor")
}

fn prime_role_file(role: &str) -> Result<(), CliError> {
    let repo = std::env::current_dir().context("current directory")?;
    let config =
        bridle_daemon::config::Config::load(&repo).context("loading .bridle/config.toml")?;
    let workflow = config
        .workflow_root(&repo)
        .map_err(anyhow::Error::new)?
        .unwrap_or_else(|| repo.join("workflow"));
    let path = workflow.join(format!("base/roles/{role}.md"));
    print!(
        "{}",
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?
    );
    if let Ok(part) = std::fs::read_to_string(repo.join(format!(".bridle/roles/{role}.md"))) {
        print!("\n{part}");
    }
    let rules = rules_section(&config, &repo, role);
    if !rules.is_empty() {
        print!("\n{rules}\n");
    }
    Ok(())
}

/// The aide role's file from the resolved workflow, then the project's own
/// `.bridle/roles/aide.md` when present. Local: the aide session's opening prompt.
pub(super) fn prime_aide() -> Result<(), CliError> {
    prime_role_file("aide")
}

/// The prototyper's role file, then the project's own `.bridle/roles/prototyper.md`.
pub(super) fn prime_prototyper() -> Result<(), CliError> {
    prime_role_file("prototyper")
}

/// The role's resolved workflow rules for this project (base, packs, `.bridle/rules/`,
/// honoring each rule's `roles:`), as a `## Rules` section, or nothing when none apply.
/// The same resolver the daemon uses for spawned agents' system prompts, so the
/// interactive roles (which no daemon spawns) get what spawned ones do.
fn rules_section(config: &bridle_daemon::config::Config, repo: &Path, role: &str) -> String {
    let text = config.role_rules_text(repo, role);
    if text.is_empty() {
        String::new()
    } else {
        format!("## Rules\n\n{}", text.trim_end())
    }
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
    crate::serve::init_tracing();
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
    let aide_client = client.clone();
    let sink = std::sync::Arc::new(bridle_mail::ClientSink(client));
    bridle_mail::Bridge::new(
        std::sync::Arc::new(store),
        sink.clone(),
        cfg,
        project,
        attachments,
    )
    .with_local(std::sync::Arc::new(bridle_mail::FileLocal {
        owner_file: state.join("state").join("owner.toml"),
        client: aide_client,
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

async fn fetch_wakes(
    client: &bridle_api::Client,
    timeout: Option<u64>,
) -> Result<Vec<bridle_api::types::WakeReason>, bridle_api::ClientError> {
    Ok(client.orchestrator_wake(timeout).await?.wakes)
}

pub(super) async fn wait_for_wake(cli: &Cli, timeout: Option<u64>) -> Result<(), CliError> {
    let client = client_for(cli).await?;
    // The old route, not `GET /v1/wake`: daemons from before br-2672 answer the new one with
    // text-less reasons (the message body lost, marked read) or 403. Every daemon serves this.
    let wakes = fetch_wakes(&client, timeout).await?;
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
    match wakes
        .iter()
        .find(|w| w.reason == bridle_api::types::DAEMON_STOPPING_WAKE)
    {
        Some(w) => Err(CliError::Stopping(format!(
            "daemon {}; re-arm once it is back",
            w.text
        ))),
        None => Ok(()),
    }
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

/// After the note is recorded, the "state written, restart me" signal for the caller's own
/// session, one way for every agent. Who the caller is comes from the environment its launcher
/// set: a worker (`BRIDLE_AGENT_ID`) is resumed by the daemon, so there is nothing to signal;
/// the orchestrator is relaunched by the daemon; an aide or advisor restarts in its pane. No
/// session identity (the human at a terminal) records only.
async fn restart_after_note(cli: &Cli) -> Result<(), CliError> {
    if std::env::var_os("BRIDLE_AGENT_ID").is_some() {
        eprintln!("note recorded; the daemon resumes workers, so no restart is needed");
    } else if std::env::var("BRIDLE_AS").is_ok_and(|a| a == "orchestrator") {
        handover_done(cli).await?;
    } else if let Some(identity) = crate::session::own_session_identity() {
        crate::session::restart_own_session(cli, &identity).await?;
    }
    Ok(())
}

/// `bridle handover write|done|list|show`.
pub(super) async fn handover(cli: &Cli, args: &HandoverArgs) -> Result<(), CliError> {
    match &args.action {
        HandoverAction::Write { file, no_restart } => {
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
            if !no_restart {
                restart_after_note(cli).await?;
            }
        }
        HandoverAction::Done => {
            eprintln!(
                "note: `bridle handover done` is deprecated; `bridle handover write` now signals the restart"
            );
            handover_done(cli).await?
        }
        HandoverAction::List { role } => {
            let list = client_for_read(cli)
                .await?
                .list_handovers(role.as_deref())
                .await?;
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
        HandoverAction::Show { .. } | HandoverAction::Latest { .. } => {
            let client = client_for_read(cli).await?;
            let h =
                match &args.action {
                    HandoverAction::Latest { role } => client
                        .latest_handover(role.as_deref())
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("no handover note"))?,
                    HandoverAction::Show { id } => client.get_handover(id).await?,
                    _ => unreachable!("matched above"),
                };
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
    fn aide_prime_waits_as_aide_and_the_orchestrator_defers_to_it() {
        let aide = include_str!("../../../../workflow/base/roles/aide.md");
        assert!(aide.contains("bridle agent wake external:aide --timeout 5400"));
        assert!(aide.contains("bridle task list --claimed-by human"));
        let orch = include_str!("../../../../workflow/base/roles/orchestrator.md");
        assert!(orch.contains("external:aide"));
        assert!(!orch.contains("tell the human first thing"));
        let advisor = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(!advisor.contains("GET /v1/messages?to=human"));
    }

    #[test]
    fn advisor_prime_includes_wake_command_for_unnamed_advisor() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("bridle agent wake external:advisor --timeout 5400"),
            "advisor prime should include wake command for unnamed advisor"
        );
    }

    #[test]
    fn advisor_prime_includes_wake_command_for_named_advisor() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("bridle agent wake external:advisor/$BRIDLE_ADVISOR_NAME --timeout 5400"),
            "advisor prime should include wake command for named advisor"
        );
    }

    #[test]
    fn advisor_prime_includes_loop_instructions_and_no_mark_read_step() {
        let role = include_str!("../../../../workflow/base/roles/advisor.md");
        assert!(
            role.contains("loop back"),
            "advisor prime should mention looping"
        );
        assert!(
            !role.contains("--mark-read"),
            "what reaches an agent is read: no mark-read step in the advisor prime"
        );
    }
}

#[cfg(test)]
mod wake_tests {
    use super::fetch_wakes;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A daemon older than br-2672 serves only the old route; a hit on any other path is a 404.
    #[tokio::test]
    async fn waits_on_the_old_route_and_reads_its_reply() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let n = sock.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let (status, body) = if req.starts_with("GET /v1/orchestrator/wake") {
                (
                    "200 OK",
                    r#"{"wakes":[{"reason":"message","text":"hello from x","detail":{"id":"m-1"}}]}"#,
                )
            } else {
                ("404 Not Found", "{}")
            };
            let resp = format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            sock.write_all(resp.as_bytes()).await.unwrap();
        });
        let client = bridle_api::Client::new(url, Some("t".into()));
        let wakes = fetch_wakes(&client, Some(1)).await.unwrap();
        server.await.unwrap();
        assert_eq!(wakes.len(), 1);
        assert_eq!(wakes[0].reason, "message");
        assert_eq!(wakes[0].text, "hello from x");
    }
}
