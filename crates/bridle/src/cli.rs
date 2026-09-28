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
    /// Reconstruct the tasks/edges/open_questions tables from the project's
    /// state branch alone (docs/design/storage.md, "Rebuild"): the
    /// migration path for a fresh clone with no `bridle.db`. Refuses if the
    /// database already has rows in any of those tables.
    Rebuild,
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
    /// Stop an agent and start its replacement fresh, in the same
    /// worktree/branch/role/model.
    Renew(RenewArgs),
    /// Remove an agent and its worktree.
    Rm(RmArgs),
    /// Readable rendering of an agent's transcript.
    Logs(LogsArgs),
    /// The event log.
    Events(EventsArgs),
    /// Usage and cost summary.
    Usage(UsageArgs),
    /// Static checks on what bridle injects into agent context.
    Cost(CostArgs),
    /// Interactive terminal UI: agents list and live event tail.
    Tui,
    /// The budget governor: windows, thresholds and state; `hold`/`release`
    /// idle the account for the human.
    Budget(BudgetArgs),
    /// Token management.
    Token(TokenArgs),
    /// Task records: create/show/edit/list/drop/reopen
    /// (docs/design/storage.md). Scoped for now to open/planned/dropped/
    /// reopened; claimed/in_review/integrated/accepted arrive with later
    /// tasks.
    Task(TaskArgs),
    /// Add or remove a coordination edge between two tasks
    /// (docs/design/coordination.md).
    Dep(DepArgs),
    /// Ask a question against a task: blocks it from being ready until
    /// answered (docs/design/coordination.md, "Questions do not stop work").
    Ask(AskArgs),
    /// Answer a task's open question, clearing the block `ask` set.
    Answer(AnswerArgs),
    /// Claim a ready task for the calling principal: `planned` -> `claimed`.
    Claim(ClaimArgs),
    /// Release the calling principal's own claim: `claimed` -> `planned`.
    Release(ReleaseArgs),
    /// List every ready task: planned, with no open `blocks` edge naming an
    /// unresolved blocker (roles-and-lifecycle.md, "ready is computed").
    Ready(ReadyArgs),
    /// Claude Code's statusLine command: reads its JSON on stdin, prints a
    /// line back, and records a usage snapshot. Never fails or blocks: see
    /// docs/design/usage-and-budget.md ("Where bridle can see usage").
    Statusline,
    /// Claude Code's Stop hook for the worker role (docs/design/
    /// coordination.md, docs/spikes/05-stop-hook-findings.md): reads its
    /// JSON on stdin and blocks the stop if the calling principal has a
    /// claimed task with no thread entry since claiming it. Never fails: any
    /// error of bridle's own allows the stop rather than trapping the agent.
    StopCheck,
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
#[command(group(ArgGroup::new("text_source").args(["text", "text_file"])))]
pub struct SendArgs {
    /// An agent id/name, or `human`.
    pub to: String,
    /// Message body.
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,
    /// Read the message body from a file (or `-` for stdin).
    #[arg(long)]
    pub text_file: Option<PathBuf>,
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
pub struct RenewArgs {
    pub agent: String,
    /// Skip the budget governor's holding/paused check for this one renew.
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
pub struct UsageArgs {
    /// Group by role or model instead of the default per-agent breakdown.
    #[arg(long)]
    pub by: Option<UsageByArg>,
    /// Only turns started within this long, e.g. `30d`, `12h`, `45m`.
    #[arg(long)]
    pub since: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
pub enum UsageByArg {
    Role,
    Model,
    Agent,
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
    /// Force a `[[budget.schedule]]` period's thresholds (or `default` for
    /// the plain `[budget]` ones), like a thermostat: until `--until`, or
    /// else until the schedule would next change on its own.
    Override(BudgetOverrideArgs),
    /// Cancel an active `override` and revert to the schedule right away.
    OverrideClear,
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
pub struct BudgetOverrideArgs {
    /// A `[[budget.schedule]]` period name, or `default` for the plain
    /// `[budget]` thresholds.
    pub period: String,
    /// Force it until this local time (`HH:MM`), today or tomorrow if
    /// already past. Without this, it lasts until the schedule would next
    /// change on its own.
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

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum TaskKindArg {
    Feature,
    Bug,
    Chore,
    Question,
    Research,
    Explore,
    ArchRevision,
    ReEvaluate,
}

#[derive(Debug, Args)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub action: TaskAction,
}

#[derive(Debug, Subcommand)]
pub enum TaskAction {
    /// Create a task, open, with title and kind.
    New(TaskNewArgs),
    /// Show one task in full, including its body and thread.
    Show(TaskShowArgs),
    /// Change a task's title or body (not its state).
    Edit(TaskEditArgs),
    /// List every task: id, title, kind, state.
    List(TaskListArgs),
    /// Drop a task (requires a reason, recorded in its thread).
    Drop(TaskDropArgs),
    /// Bring a dropped task back.
    Reopen(TaskReopenArgs),
    /// Add a plain note to a task's thread (no question/answer semantics,
    /// doesn't affect readiness).
    Note(TaskNoteArgs),
}

#[derive(Debug, Args)]
pub struct TaskNewArgs {
    pub title: String,
    #[arg(short = 'k', long, value_enum)]
    pub kind: TaskKindArg,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct TaskListArgs {
    /// Filter to tasks claimed by this principal: `me`, `human`, an agent
    /// name, or a full principal id.
    #[arg(long)]
    pub claimed_by: Option<String>,
}

#[derive(Debug, Args)]
pub struct TaskShowArgs {
    pub task: String,
}

#[derive(Debug, Args)]
pub struct TaskEditArgs {
    pub task: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
}

#[derive(Debug, Args)]
pub struct TaskDropArgs {
    pub task: String,
    #[arg(long)]
    pub reason: String,
}

#[derive(Debug, Args)]
pub struct TaskReopenArgs {
    pub task: String,
}

#[derive(Debug, Args)]
pub struct TaskNoteArgs {
    pub task: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum EdgeKindArg {
    Blocks,
    Parent,
    DiscoveredFrom,
    Related,
    Supersedes,
    Duplicates,
}

#[derive(Debug, Args)]
pub struct DepArgs {
    #[command(subcommand)]
    pub action: DepAction,
}

#[derive(Debug, Subcommand)]
pub enum DepAction {
    /// Add an edge.
    Add(DepEdgeArgs),
    /// Remove an edge.
    Rm(DepEdgeArgs),
}

#[derive(Debug, Args)]
pub struct AskArgs {
    pub task: String,
    pub text: String,
}

#[derive(Debug, Args)]
pub struct AnswerArgs {
    pub task: String,
    pub text: String,
}

#[derive(Debug, Args)]
pub struct ClaimArgs {
    pub task: String,
}

#[derive(Debug, Args)]
pub struct ReleaseArgs {
    pub task: String,
}

#[derive(Debug, Args)]
pub struct DepEdgeArgs {
    /// The edge's `from` task, unless `--blocked-by` is given.
    pub task: String,
    /// Sugar for `--kind blocks --to <task>` with `from`/`to` swapped: reads
    /// as "`<task>` is blocked by `<other>`".
    #[arg(long, value_name = "TASK", conflicts_with_all = ["to", "kind"])]
    pub blocked_by: Option<String>,
    /// The edge's `to` task.
    #[arg(long, value_name = "TASK")]
    pub to: Option<String>,
    #[arg(long, value_enum, default_value = "blocks")]
    pub kind: EdgeKindArg,
}

#[derive(Debug, Args)]
pub struct ReadyArgs {
    /// Fan out across every daemon registered on this machine (`bridle
    /// daemons`), not just the one `--url`/`--project`/discovery resolves.
    #[arg(long)]
    pub all: bool,
    /// Filter by role. A stub for now: tasks don't carry a role field yet
    /// (P0-1 gap), so this has nothing to filter on and is accepted but
    /// ignored.
    #[arg(long)]
    pub role: Option<String>,
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
        assert_eq!(args.text.as_deref(), Some("hello"));
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
    fn send_accepts_text_file() {
        let cli = parse(&["send", "w1", "--text-file", "msg.txt"]).unwrap();
        let Command::Send(args) = cli.command else {
            panic!("expected send")
        };
        assert_eq!(args.to, "w1");
        assert_eq!(args.text, None);
        assert_eq!(
            args.text_file.as_deref(),
            Some(std::path::Path::new("msg.txt"))
        );
    }

    #[test]
    fn send_rejects_text_and_text_file_together() {
        let err = parse(&["send", "w1", "hello", "--text-file", "msg.txt"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn spawn_prompt_file_accepts_dash_for_stdin() {
        let cli = parse(&["spawn", "worker", "--prompt-file", "-"]).unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert_eq!(args.prompt_file.as_deref(), Some(std::path::Path::new("-")));
    }

    #[test]
    fn send_text_file_accepts_dash_for_stdin() {
        let cli = parse(&["send", "w1", "--text-file", "-"]).unwrap();
        let Command::Send(args) = cli.command else {
            panic!("expected send")
        };
        assert_eq!(args.text_file.as_deref(), Some(std::path::Path::new("-")));
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

    #[test]
    fn task_new_takes_title_and_kind() {
        let cli = parse(&["task", "new", "Add foo", "-k", "feature", "--body", "desc"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        let TaskAction::New(a) = args.action else {
            panic!("expected task new")
        };
        assert_eq!(a.title, "Add foo");
        assert!(matches!(a.kind, TaskKindArg::Feature));
        assert_eq!(a.body.as_deref(), Some("desc"));
    }

    #[test]
    fn task_new_kind_uses_kebab_case_arch_revision() {
        let cli = parse(&["task", "new", "x", "-k", "arch-revision"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        let TaskAction::New(a) = args.action else {
            panic!("expected task new")
        };
        assert!(matches!(a.kind, TaskKindArg::ArchRevision));
    }

    #[test]
    fn task_new_requires_kind() {
        let err = parse(&["task", "new", "Add foo"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn task_drop_requires_reason() {
        let err = parse(&["task", "drop", "tw-1234"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);

        let cli = parse(&["task", "drop", "tw-1234", "--reason", "budget cut"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        let TaskAction::Drop(a) = args.action else {
            panic!("expected task drop")
        };
        assert_eq!(a.task, "tw-1234");
        assert_eq!(a.reason, "budget cut");
    }

    #[test]
    fn task_reopen_and_show_and_list_parse() {
        let cli = parse(&["task", "reopen", "tw-1234"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        assert!(matches!(args.action, TaskAction::Reopen(a) if a.task == "tw-1234"));

        let cli = parse(&["task", "show", "tw-1234"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        assert!(matches!(args.action, TaskAction::Show(a) if a.task == "tw-1234"));

        let cli = parse(&["task", "list"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        assert!(matches!(args.action, TaskAction::List(a) if a.claimed_by.is_none()));

        let cli = parse(&["task", "list", "--claimed-by", "me"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        assert!(
            matches!(args.action, TaskAction::List(a) if a.claimed_by.as_deref() == Some("me"))
        );
    }

    #[test]
    fn task_edit_takes_title_or_body() {
        let cli = parse(&["task", "edit", "tw-1234", "--title", "new title"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        let TaskAction::Edit(a) = args.action else {
            panic!("expected task edit")
        };
        assert_eq!(a.task, "tw-1234");
        assert_eq!(a.title.as_deref(), Some("new title"));
        assert_eq!(a.body, None);
    }

    #[test]
    fn dep_add_blocked_by_parses() {
        let cli = parse(&["dep", "add", "tw-7fa2", "--blocked-by", "tw-c0f1"]).unwrap();
        let Command::Dep(args) = cli.command else {
            panic!("expected dep")
        };
        let DepAction::Add(a) = args.action else {
            panic!("expected dep add")
        };
        assert_eq!(a.task, "tw-7fa2");
        assert_eq!(a.blocked_by.as_deref(), Some("tw-c0f1"));
        assert_eq!(a.to, None);
    }

    #[test]
    fn dep_add_to_and_kind_parses() {
        let cli = parse(&[
            "dep",
            "add",
            "tw-7fa2",
            "--to",
            "hx-19ab",
            "--kind",
            "discovered-from",
        ])
        .unwrap();
        let Command::Dep(args) = cli.command else {
            panic!("expected dep")
        };
        let DepAction::Add(a) = args.action else {
            panic!("expected dep add")
        };
        assert_eq!(a.task, "tw-7fa2");
        assert_eq!(a.to.as_deref(), Some("hx-19ab"));
        assert!(matches!(a.kind, EdgeKindArg::DiscoveredFrom));
    }

    #[test]
    fn dep_add_rejects_kind_alongside_blocked_by() {
        let err = parse(&[
            "dep",
            "add",
            "tw-7fa2",
            "--blocked-by",
            "tw-c0f1",
            "--kind",
            "parent",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn dep_rm_parses() {
        let cli = parse(&[
            "dep", "rm", "tw-7fa2", "--to", "tw-c0f1", "--kind", "related",
        ])
        .unwrap();
        let Command::Dep(args) = cli.command else {
            panic!("expected dep")
        };
        let DepAction::Rm(a) = args.action else {
            panic!("expected dep rm")
        };
        assert_eq!(a.task, "tw-7fa2");
        assert_eq!(a.to.as_deref(), Some("tw-c0f1"));
        assert!(matches!(a.kind, EdgeKindArg::Related));
    }

    #[test]
    fn rebuild_parses() {
        let cli = parse(&["rebuild"]).unwrap();
        assert!(matches!(cli.command, Command::Rebuild));
    }

    #[test]
    fn ready_parses_all_and_role() {
        let cli = parse(&["ready", "--all", "--role", "worker"]).unwrap();
        let Command::Ready(args) = cli.command else {
            panic!("expected ready")
        };
        assert!(args.all);
        assert_eq!(args.role.as_deref(), Some("worker"));

        let cli = parse(&["ready"]).unwrap();
        let Command::Ready(args) = cli.command else {
            panic!("expected ready")
        };
        assert!(!args.all);
        assert_eq!(args.role, None);
    }
}
