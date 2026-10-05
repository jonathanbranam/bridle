//! Commands called by Claude Code hooks: statusline, stop-check, arch-guard.

use super::*;

/// Claude Code's statusLine command (docs/design/cli.md, docs/design/usage-and-budget.md
/// "Where bridle can see usage"). Reads Claude Code's JSON from stdin and prints a short
/// line back. Purely local: no daemon call, so unparseable stdin is the only way this
/// can produce a degraded line, never a slow or failed one. It no longer records a
/// snapshot with the daemon (dropped per s8kn's scope change: the context governor
/// gets account-wide windows from `get_usage` instead, and `POST /v1/statusline` /
/// `interactive_usage` stay in the daemon unused for now, not removed). It does write the
/// session's context size to a local file (`statusline::record_context`, ticket c9zm).
pub(super) async fn statusline(_cli: &Cli) -> Result<(), CliError> {
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
pub(super) async fn stop_check(cli: &Cli) -> Result<(), CliError> {
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
pub(super) async fn arch_guard(cli: &Cli) -> Result<(), CliError> {
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

/// Claude Code's PreToolUse hook for Bash: refuses killing by name that the
/// `pkill`/`killall` deny rules can't match (rule `no-kill-by-name`). Pure
/// stdin to stdout; anything unparseable allows.
pub(super) fn kill_guard() -> Result<(), CliError> {
    let input: serde_json::Value = std::io::read_to_string(std::io::stdin())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);
    if let Some(reason) = crate::kill_guard::refusal(&input) {
        println!("{}", crate::arch_guard::deny_json(&reason));
    }
    Ok(())
}
