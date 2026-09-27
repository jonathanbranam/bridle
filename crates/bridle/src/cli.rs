//! clap definitions for the `bridle` CLI. See docs/design/cli.md.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "bridle",
    version,
    about = "Run and coordinate Claude Code agents"
)]
pub struct Cli {
    /// Talk to the daemon at this URL (highest priority; see the discovery order in docs/design/agent-host/daemon.md).
    #[arg(long, global = true)]
    pub url: Option<String>,

    /// Select a daemon from the registry by project name.
    #[arg(long, global = true, env = "BRIDLE_PROJECT")]
    pub project: Option<String>,

    /// Bearer token; see docs/design/agent-host/principals.md for how the CLI picks one when this is unset.
    #[arg(long, global = true)]
    pub token: Option<String>,

    /// Print the raw API response as compact JSON instead of a human table.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the daemon.
    Serve(ServeArgs),
    /// Ask the daemon to shut down gracefully.
    StopDaemon,
    /// List every running project daemon on this machine.
    Daemons,
    /// Daemon + agents summary.
    Status,
    /// Spawn a new agent.
    Spawn(SpawnArgs),
    /// List agents.
    Agents(AgentsArgs),
    /// Show one agent's detail.
    Show(ShowArgs),
    /// Send a message to an agent or the human inbox.
    Send(SendArgs),
    /// Messages addressed to me (the calling principal).
    Inbox(InboxArgs),
    /// Interrupt a running agent's turn.
    Interrupt(InterruptArgs),
    /// Stop an agent.
    Stop(StopArgs),
    /// Resume a stopped/exited/crashed/lost agent.
    Resume(ResumeArgs),
    /// Remove an agent and its worktree.
    Rm(RmArgs),
    /// Readable rendering of an agent's transcript.
    Logs(LogsArgs),
    /// The event log.
    Events(EventsArgs),
    /// Usage and cost summary.
    Usage,
    /// Static checks on what bridle injects into agent context.
    Cost(CostArgs),
    /// Interactive terminal UI: agents list and live event tail.
    Tui,
    /// The budget governor: windows, thresholds and state; `hold`/`release`
    /// idle the account for the human.
    Budget(BudgetArgs),
    /// Token management.
    Token(TokenArgs),
    /// Claude Code's statusLine command: reads its JSON on stdin, prints a
    /// line back, and records a usage snapshot. Never fails or blocks: see
    /// docs/design/usage-and-budget.md ("Where bridle can see usage").
    Statusline,
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    /// The clone's main checkout. Defaults to the current directory.
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Defaults to the repo's parent directory.
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    /// Defaults to `[daemon] listen` in config, else `127.0.0.1:0`.
    #[arg(long)]
    pub listen: Option<SocketAddr>,
    /// Re-exec into the background; logs to `<workspace>/.bridle/daemon.log`.
    #[arg(long)]
    pub detach: bool,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("prompt_source").args(["prompt", "prompt_file"])))]
#[command(group(ArgGroup::new("workdir_source").args(["worktree", "in_repo", "cwd"])))]
pub struct SpawnArgs {
    /// A role from `<repo>/.bridle/config.toml` (built-in: worker, manager, orchestrator).
    pub role: String,
    #[arg(long)]
    pub name: Option<String>,
    /// First user message; starts the first turn.
    #[arg(long)]
    pub prompt: Option<String>,
    /// Read the first user message from a file.
    #[arg(long)]
    pub prompt_file: Option<PathBuf>,
    /// New worktree `wt/<name>` on branch `bridle/<name>`.
    #[arg(long)]
    pub worktree: bool,
    /// With `--worktree`, the ref to branch from (default `HEAD`).
    #[arg(long, requires = "worktree")]
    pub base: Option<String>,
    /// Run in the clone's main checkout.
    #[arg(long)]
    pub in_repo: bool,
    /// Run in an explicit existing directory.
    #[arg(long)]
    pub cwd: Option<PathBuf>,
    #[arg(long)]
    pub model: Option<String>,
    /// Skip the budget governor's holding/paused check for this one spawn.
    #[arg(long)]
    pub ignore_budget: bool,
}

#[derive(Debug, Args)]
pub struct AgentsArgs {
    /// Include stopped/exited/crashed/lost agents too.
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    pub agent: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
pub enum WhenArg {
    Now,
    Idle,
}

#[derive(Debug, Args)]
pub struct SendArgs {
    /// An agent id/name, or `human`.
    pub to: String,
    pub text: String,
    /// Mark this as a question (expects a reply).
    #[arg(long)]
    pub question: bool,
    #[arg(long, value_enum, default_value = "now")]
    pub when: WhenArg,
    /// This message replies to an earlier one.
    #[arg(long)]
    pub reply_to: Option<String>,
}

#[derive(Debug, Args)]
pub struct InboxArgs {
    /// Include already-read messages too (default: unread only).
    #[arg(long)]
    pub all: bool,
    /// Mark every listed message read.
    #[arg(long)]
    pub mark_read: bool,
}

#[derive(Debug, Args)]
pub struct InterruptArgs {
    pub agent: String,
    /// Also discard messages held for `--when idle` delivery.
    #[arg(long)]
    pub drop_held: bool,
}

#[derive(Debug, Args)]
pub struct StopArgs {
    pub agent: String,
    /// Skip the graceful stdin close; go straight to SIGTERM.
    #[arg(long)]
    pub now: bool,
}

#[derive(Debug, Args)]
pub struct ResumeArgs {
    pub agent: String,
    /// Skip the budget governor's holding/paused check for this one resume.
    #[arg(long)]
    pub ignore_budget: bool,
}

#[derive(Debug, Args)]
pub struct RmArgs {
    pub agent: String,
    /// Remove even with uncommitted changes in the worktree.
    #[arg(long)]
    pub force: bool,
    /// Delete the agent's branch too (default: keep it).
    #[arg(long)]
    pub delete_branch: bool,
}

#[derive(Debug, Args)]
pub struct LogsArgs {
    pub agent: String,
    /// Keep polling for new lines every second.
    #[arg(long)]
    pub follow: bool,
    /// Print transcript lines verbatim instead of rendering them.
    #[arg(long)]
    pub raw: bool,
    /// Only lines after this transcript line number.
    #[arg(long)]
    pub since: Option<u64>,
}

#[derive(Debug, Args)]
pub struct EventsArgs {
    /// Keep streaming new events over SSE.
    #[arg(long)]
    pub follow: bool,
    #[arg(long)]
    pub since: Option<i64>,
    #[arg(long)]
    pub agent: Option<String>,
    /// Prefix match, e.g. `message.` or `agent.state`.
    #[arg(long)]
    pub kind: Option<String>,
}

#[derive(Debug, Args)]
pub struct CostArgs {
    #[command(subcommand)]
    pub action: CostAction,
}

#[derive(Debug, Subcommand)]
pub enum CostAction {
    /// Measure each role's fixed context overhead (currently: the rendered
    /// system-prompt file) against the committed baseline
    /// (.bridle/cost-baseline.json).
    Audit(CostAuditArgs),
}

#[derive(Debug, Args)]
pub struct CostAuditArgs {
    /// Exit non-zero if any role grew more than the threshold over baseline
    /// (see bridle_daemon::cost_audit::GROWTH_THRESHOLD_PERCENT).
    #[arg(long)]
    pub check: bool,
}

#[derive(Debug, Args)]
pub struct BudgetArgs {
    #[command(subcommand)]
    pub action: Option<BudgetAction>,
}

#[derive(Debug, Subcommand)]
pub enum BudgetAction {
    /// Idle the account for the human: stop idle agents, wind working ones
    /// down, and hold `bridle spawn`/`resume` until `release` (or `--for`/
    /// `--until` passes).
    #[command(group(ArgGroup::new("hold_duration").args(["for_", "until"])))]
    Hold(BudgetHoldArgs),
    /// End a `hold` early.
    Release,
}

#[derive(Debug, Args)]
pub struct BudgetHoldArgs {
    /// Hold for a duration, e.g. `3h`, `45m`, `90s`.
    #[arg(long = "for")]
    pub for_: Option<String>,
    /// Hold until this local time (`HH:MM`), today or tomorrow if already past.
    #[arg(long)]
    pub until: Option<String>,
}

#[derive(Debug, Args)]
pub struct TokenArgs {
    #[command(subcommand)]
    pub action: TokenAction,
}

#[derive(Debug, Subcommand)]
pub enum TokenAction {
    /// Mint an `external:<name>` token (human only).
    Create { name: String },
    /// List external tokens: name, created-at, revoked-or-not (human only).
    List,
    /// Revoke an `external:<name>` token (human only). An agent's own token
    /// isn't revoked this way; that happens through `bridle rm`.
    Revoke { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        let mut full = vec!["bridle"];
        full.extend_from_slice(args);
        Cli::try_parse_from(full)
    }

    #[test]
    fn spawn_worktree_and_in_repo_are_mutually_exclusive() {
        let err = parse(&["spawn", "worker", "--worktree", "--in-repo"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn spawn_worktree_and_cwd_are_mutually_exclusive() {
        let err = parse(&["spawn", "worker", "--worktree", "--cwd", "/tmp/x"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn spawn_base_requires_worktree() {
        let err = parse(&["spawn", "worker", "--base", "main"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn spawn_base_with_worktree_is_fine() {
        let cli = parse(&["spawn", "worker", "--worktree", "--base", "main"]).unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert!(args.worktree);
        assert_eq!(args.base.as_deref(), Some("main"));
    }

    #[test]
    fn spawn_prompt_and_prompt_file_are_mutually_exclusive() {
        let err = parse(&[
            "spawn",
            "worker",
            "--prompt",
            "hi",
            "--prompt-file",
            "p.txt",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn spawn_plain_role_parses_with_no_workdir_override() {
        let cli = parse(&["spawn", "worker"]).unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert_eq!(args.role, "worker");
        assert!(!args.worktree);
        assert!(!args.in_repo);
        assert_eq!(args.cwd, None);
    }

    #[test]
    fn send_when_defaults_to_now() {
        let cli = parse(&["send", "w1", "hello"]).unwrap();
        let Command::Send(args) = cli.command else {
            panic!("expected send")
        };
        assert!(matches!(args.when, WhenArg::Now));
        assert_eq!(args.to, "w1");
        assert_eq!(args.text, "hello");
    }

    #[test]
    fn send_when_idle_parses() {
        let cli = parse(&["send", "human", "status?", "--when", "idle", "--question"]).unwrap();
        let Command::Send(args) = cli.command else {
            panic!("expected send")
        };
        assert!(matches!(args.when, WhenArg::Idle));
        assert!(args.question);
    }

    #[test]
    fn send_when_rejects_bad_value() {
        let err = parse(&["send", "w1", "hi", "--when", "soon"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn global_flags_work_before_and_after_the_subcommand() {
        let cli = parse(&["--json", "--project", "demo", "agents"]).unwrap();
        assert!(cli.json);
        assert_eq!(cli.project.as_deref(), Some("demo"));
        assert!(matches!(cli.command, Command::Agents(_)));
    }

    #[test]
    fn token_create_requires_a_name() {
        let err = parse(&["token", "create"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
        let cli = parse(&["token", "create", "orchestrator"]).unwrap();
        let Command::Token(t) = cli.command else {
            panic!("expected token")
        };
        let TokenAction::Create { name } = t.action else {
            panic!("expected create")
        };
        assert_eq!(name, "orchestrator");
    }

    #[test]
    fn token_list_and_revoke_parse() {
        let cli = parse(&["token", "list"]).unwrap();
        let Command::Token(t) = cli.command else {
            panic!("expected token")
        };
        assert!(matches!(t.action, TokenAction::List));

        let cli = parse(&["token", "revoke", "orchestrator"]).unwrap();
        let Command::Token(t) = cli.command else {
            panic!("expected token")
        };
        let TokenAction::Revoke { name } = t.action else {
            panic!("expected revoke")
        };
        assert_eq!(name, "orchestrator");
    }

    #[test]
    fn cost_audit_check_flag_parses() {
        let cli = parse(&["cost", "audit", "--check"]).unwrap();
        let Command::Cost(args) = cli.command else {
            panic!("expected cost")
        };
        let CostAction::Audit(audit) = args.action;
        assert!(audit.check);

        let cli = parse(&["cost", "audit"]).unwrap();
        let Command::Cost(args) = cli.command else {
            panic!("expected cost")
        };
        let CostAction::Audit(audit) = args.action;
        assert!(!audit.check);
    }

    #[test]
    fn rm_flags_parse() {
        let cli = parse(&["rm", "w1", "--force", "--delete-branch"]).unwrap();
        let Command::Rm(args) = cli.command else {
            panic!("expected rm")
        };
        assert!(args.force);
        assert!(args.delete_branch);
    }
}
