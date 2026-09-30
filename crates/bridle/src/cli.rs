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
    /// Restart the daemon in place once every agent is idle, then resume every agent that was running (orchestrator or human).
    Restart(RestartArgs),
    /// Check the project's setup (git, config, tools, gitignore) and say what to fix; exit 1 on a failure.
    Doctor(DoctorArgs),
    /// Scaffold `.bridle/config.toml` and `.gitignore` entries in a git repo; never overwrites.
    Init(InitArgs),
    /// Write or remove a macOS LaunchAgent that runs the daemon (never runs launchctl).
    Launchd(LaunchdArgs),
    /// Write Linux systemd user units that run this machine's project daemons (never runs systemctl).
    Systemd(SystemdArgs),
    /// Reconstruct the tasks/edges/open_questions tables from the project's
    /// state branch alone (docs/design/storage.md, "Rebuild"): the
    /// migration path for a fresh clone with no `bridle.db`. Refuses if the
    /// database already has rows in any of those tables.
    Rebuild(RebuildArgs),
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
    /// Block until a task changes state (or reaches one), or a message arrives.
    Wait(WaitArgs),
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
    /// Task records: create/show/edit/list/drop/done/reopen/plan/summary.
    /// Plan changes state: `open` -> `planned`. See docs/design/storage.md
    /// for state machine and docs/design/coordination.md for the task lifecycle.
    Task(TaskArgs),
    /// A task's declared impact: the spec ids and files it will touch
    /// (docs/design/impact-and-conflicts.md).
    Impact(ImpactArgs),
    /// Does a task's or agent's branch (or `--branch B`) merge cleanly into the integration branch; exits 1 if not.
    Probe(ProbeArgs),
    /// Merge a task's branch into the integration branch after the check passes.
    Land(LandArgs),
    /// Conflicts opened by `impact check`: list and resolve
    /// (docs/design/impact-and-conflicts.md).
    Conflict(ConflictArgs),
    /// Ports for dev servers: allocate, release, list (docs/design/worktrees-and-ports.md).
    Port(PortArgs),
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
    /// The highest queue tier with a startable task: planned, deps met, no
    /// open question, unclaimed (roles-and-lifecycle.md, "the queue"). A
    /// task outside the queue is backlog and never shown here.
    Ready(ReadyArgs),
    /// The queue: claimed tasks with their worker, then the tiers in rank
    /// order (roles-and-lifecycle.md, "the queue"). Read-only; `queue set`/
    /// `queue add-tier` edit it, PM (or human) only.
    Queue(QueueArgs),
    /// Claude Code's statusLine command: reads its JSON on stdin and prints a
    /// line back. Never fails or blocks: see docs/design/usage-and-budget.md
    /// ("Where bridle can see usage").
    Statusline,
    /// Claude Code's Stop hook for the worker role (docs/design/
    /// coordination.md, docs/spikes/05-stop-hook-findings.md): reads its
    /// JSON on stdin and blocks the stop if the calling principal has a
    /// claimed task with no thread entry since claiming it. Never fails: any
    /// error of bridle's own allows the stop rather than trapping the agent.
    StopCheck,
    /// Claude Code's PreToolUse hook: reads the hook JSON on stdin and denies
    /// an Edit/Write/MultiEdit/NotebookEdit under `design/architecture/`
    /// unless the calling worker has claimed an `arch-revision` task. Never
    /// fails: any error of bridle's own allows.
    ArchGuard,
    /// The orchestrator's supervision hooks (docs/design/agent-host/orchestrator-supervision.md).
    Orchestrator(OrchestratorArgs),
    /// Focus hours (ticket cvaq): the human's `[[focus]]` periods in `~/.bridle/config.toml`.
    Focus(FocusArgs),
    /// Wait for something the orchestrator should act on, print it and exit 0 (`nothing` after
    /// 5 minutes of quiet). Run it in the background; run it again after each exit
    /// (orchestrator-supervision.md, section 5). `external:orchestrator` only.
    WaitForWake(WaitForWakeArgs),
    /// Email bridge (docs/design/mail.md). Runs as `external:mail`: `BRIDLE_AS=mail` or `--token`.
    Mail(MailArgs),
    /// The orchestrator's handover note, kept as a record (orchestrator-supervision.md, section 7).
    Handover(HandoverArgs),
    /// Print a fresh session's opening context for a role: the role prompt,
    /// current state and startup steps.
    Prime(PrimeArgs),
    /// Layer resolution over the workflow rules (docs/design/workflow-layers.md):
    /// which layer wins each rule id, and what a project changes.
    Rules(RulesArgs),
    /// Renders the resolved workflow layers into what Claude Code reads:
    /// CLAUDE.md's managed block, .claude/skills, .claude/agents and
    /// .claude/settings.json's hooks (docs/design/workflow-layers.md,
    /// "Rendering into what the agent harness reads"). Local, like `rules
    /// explain`/`diff` — no daemon call.
    Sync,
    /// Refresh the workflow `bridle init` vendored into `.bridle/workflow/` (never automatic).
    #[command(subcommand)]
    Workflow(WorkflowAction),
    /// Validate capability spec files (docs/design/specs.md) and print each
    /// diagnostic as file:line:col: message. Local, like `rules` — no daemon call.
    Spec(SpecArgs),
    /// Goals (docs/design/goals-tier.md). Local — no daemon call.
    Goals(GoalsArgs),
    /// Tickets under docs/tickets/ (docs/design/cli.md). Local files; `new` also files a task
    /// when a daemon is reachable.
    Ticket(TicketArgs),
    /// Architecture-tier elements (docs/design/architecture-tier.md). Local, no daemon call.
    Arch(ArchArgs),
    /// Exploration findings docs (docs/design/explorations.md). Local, no daemon call.
    Explore(ExploreArgs),
    /// Tag the current tmux pane with the @bridle option (orchestrator panes use this).
    /// Local, no daemon call.
    Pane(PaneArgs),
    /// Machine-scope settings for this clone (hw6c).
    Machine(MachineArgs),
    /// Start the orchestrator or advisor session: `claude` with the role's settings, names and
    /// opening prompt, from any directory (replaces scripts/claude-orchestrator and -advisor).
    Session(SessionArgs),
    /// Start an advisor in a new tmux pane (ticket ervd); orchestrator and human only.
    Advisor(AdvisorArgs),
    /// Trace links from goals down to scenarios (docs/design/traceability.md). Local, no daemon call.
    Trace(TraceArgs),
}

#[derive(Debug, Args)]
pub struct TraceArgs {
    #[command(subcommand)]
    pub action: TraceAction,
    /// The goals directory (default `design/goals`).
    #[arg(
        long,
        value_name = "DIR",
        default_value = "design/goals",
        global = true
    )]
    pub goals: PathBuf,
    /// The architecture directory (default `design/architecture`).
    #[arg(
        long,
        value_name = "DIR",
        default_value = "design/architecture",
        global = true
    )]
    pub arch: PathBuf,
    /// The specs directory (default `design/specs`).
    #[arg(
        long,
        value_name = "DIR",
        default_value = "design/specs",
        global = true
    )]
    pub specs: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum TraceAction {
    /// Everything that depends on an element, transitively.
    Down { id: String },
    /// Everything an element rests on, up to the goals.
    Up { id: String },
    /// Requirements that trace to nothing (a warning: exit 0).
    Orphans,
    /// Links whose recorded hash differs from upstream's current one (exit 1 if any).
    Suspect,
    /// Rewrite a requirement's link hashes to the upstream elements' current ones.
    Confirm { id: String },
}

#[derive(Debug, Args)]
pub struct ExploreArgs {
    #[command(subcommand)]
    pub action: ExploreAction,
}

#[derive(Debug, Subcommand)]
pub enum ExploreAction {
    /// Check findings frontmatter; exits non-zero on any error.
    Check(ExploreCheckArgs),
    /// Scaffold design/explore/<id>/findings.md with status open.
    New(ExploreIdArgs),
    /// Set the findings doc's status to concluded.
    Conclude(ExploreIdArgs),
    /// Set the findings doc's status to abandoned.
    Abandon(ExploreIdArgs),
}

#[derive(Debug, Args)]
pub struct ExploreCheckArgs {
    /// Findings files, or directories searched recursively for `*.md`
    /// (default `design/explore`).
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ExploreIdArgs {
    /// The exploration's task id; its doc is design/explore/<id>/findings.md.
    pub id: String,
}

#[derive(Debug, Args)]
pub struct TicketArgs {
    #[command(subcommand)]
    pub action: TicketAction,
}

#[derive(Debug, Subcommand)]
pub enum TicketAction {
    /// Mint docs/tickets/open/<slug>-<id>.md with a fresh ID and print its path.
    New(TicketNewArgs),
    /// Stamp `closed:` and move an open ticket to docs/tickets/resolved/ (no commit, no task change).
    Resolve(TicketResolveArgs),
    /// Set one frontmatter field (title, repos, changes, specs, needs, see); list fields take comma-separated values.
    Set(TicketSetArgs),
    /// Check every ticket's frontmatter, IDs, needs/see and [[links]]; exits 1 listing the problems.
    Check(TicketCheckArgs),
}

#[derive(Debug, Args)]
pub struct TicketNewArgs {
    pub title: String,
    /// Repos the ticket concerns, comma-separated (default: the project name).
    #[arg(long, value_delimiter = ',')]
    pub repos: Vec<String>,
    /// Ticket ids this one needs, comma-separated.
    #[arg(long, value_delimiter = ',')]
    pub needs: Vec<String>,
    /// Related ticket ids, comma-separated.
    #[arg(long, value_delimiter = ',')]
    pub see: Vec<String>,
    /// Don't create the matching bridle task.
    #[arg(long)]
    pub no_task: bool,
}

#[derive(Debug, Args)]
pub struct TicketResolveArgs {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct TicketSetArgs {
    pub id: String,
    pub field: String,
    /// The new value; for a list field, comma-separated (empty clears it).
    pub value: String,
}

#[derive(Debug, Args)]
pub struct TicketCheckArgs {
    /// Print nothing when everything is fine.
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Debug, Args)]
pub struct GoalsArgs {
    #[command(subcommand)]
    pub action: GoalsAction,
}

#[derive(Debug, Subcommand)]
pub enum GoalsAction {
    /// List goals with id, firmness, priority, stance and title; exits 1 on parse errors.
    List(GoalsListArgs),
    /// Propose a change to a goal's firmness, priority, or stance.
    Propose(GoalsProposeProposeArgs),
}

#[derive(Debug, Args)]
pub struct GoalsListArgs {
    /// The goals directory, searched recursively for `*.md` (default `design/goals`).
    #[arg(long, value_name = "DIR", default_value = "design/goals")]
    pub root: PathBuf,
    /// Only goals with this priority (now, next, later, someday).
    #[arg(long, value_name = "P")]
    pub priority: Option<String>,
    /// Only goals with this stance (build, keep-open, unaddressed).
    #[arg(long, value_name = "S")]
    pub stance: Option<String>,
}

#[derive(Debug, Args)]
pub struct GoalsProposeProposeArgs {
    /// The goal id (e.g., g-01).
    pub goal_id: String,
    /// The change to propose: firmness=firm|priority=now|stance=build (repeatable).
    #[arg(long, value_name = "KEY=VALUE")]
    pub change: Vec<String>,
    /// Why this change is proposed.
    #[arg(long)]
    pub why: String,
    /// The goals directory (default `design/goals`).
    #[arg(long, value_name = "DIR", default_value = "design/goals")]
    pub goals_root: PathBuf,
}

#[derive(Debug, Args)]
pub struct ArchArgs {
    #[command(subcommand)]
    pub action: ArchAction,
}

#[derive(Debug, Subcommand)]
pub enum ArchAction {
    /// List the elements (id, invariant flag, title); exits non-zero, printing
    /// diagnostics, on a missing or duplicate id.
    List(ArchListArgs),
    /// Propose a change to the architecture.
    Propose(ArchProposeArgs),
}

#[derive(Debug, Args)]
pub struct ArchListArgs {
    /// Only elements marked `invariant`.
    #[arg(long)]
    pub invariants: bool,
    /// The architecture directory, searched recursively for `*.md` (default
    /// `design/architecture`, relative to the current directory).
    #[arg(long, value_name = "DIR", default_value = "design/architecture")]
    pub root: PathBuf,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("arg_source").args(["argument", "argument_file"])))]
pub struct ArchProposeArgs {
    /// The title of the proposal.
    #[arg(long)]
    pub title: String,
    /// The proposal text.
    #[arg(long)]
    pub argument: Option<String>,
    /// Read the proposal from a file (or `-` for stdin).
    #[arg(long)]
    pub argument_file: Option<PathBuf>,
    /// The architecture directory (default `design/architecture`).
    #[arg(long, value_name = "DIR", default_value = "design/architecture")]
    pub arch_root: PathBuf,
}

#[derive(Debug, Args)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub action: SpecAction,
}

#[derive(Debug, Subcommand)]
pub enum SpecAction {
    /// Parse spec files and report diagnostics; exits non-zero on any error.
    Check(SpecCheckArgs),
    /// Write a stable id into every requirement and scenario heading that lacks one.
    Id(SpecIdArgs),
    /// Export the specs for test runners; refuses (printing the diagnostics)
    /// when any spec has errors.
    Export(SpecExportArgs),
    /// Migrate another spec system's files into `design/specs`.
    Import(SpecImportArgs),
    /// List executable scenarios whose id does not appear in test sources.
    Coverage(SpecCoverageArgs),
}

#[derive(Debug, Args)]
pub struct SpecImportArgs {
    #[command(subcommand)]
    pub source: SpecImportSource,
}

#[derive(Debug, Subcommand)]
pub enum SpecImportSource {
    /// Move `<from>/<capability>/spec.md` to `<to>/<capability>.md` and assign ids.
    Openspec(SpecImportOpenspecArgs),
}

#[derive(Debug, Args)]
pub struct SpecImportOpenspecArgs {
    /// The OpenSpec specs directory (default `openspec/specs`).
    #[arg(long, value_name = "DIR", default_value = "openspec/specs")]
    pub from: PathBuf,
    /// The bridle specs directory (default `design/specs`); its `.ids` is the id ledger.
    #[arg(long, value_name = "DIR", default_value = "design/specs")]
    pub to: PathBuf,
    /// Print what would change; write nothing.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SpecFormatArg {
    /// One `.feature` per capability, executable scenarios only.
    Gherkin,
    /// The whole AST, every scenario, as one document.
    Json,
}

#[derive(Debug, Args)]
pub struct SpecExportArgs {
    #[arg(long, value_enum)]
    pub format: SpecFormatArg,
    /// Write files here. Default: gherkin goes to `.bridle/cache/features/`
    /// (gitignored), json to stdout.
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
    /// Spec files, or directories searched recursively for `*.md`. Default:
    /// the `--root` directory.
    pub paths: Vec<PathBuf>,
    /// The specs directory used when no paths are given (default
    /// `design/specs`, relative to the current directory).
    #[arg(long, value_name = "DIR")]
    pub root: Option<PathBuf>,
    /// Export only this scenario (`s-xxxx`), or all of a requirement's
    /// scenarios (`r-xxxx`). Repeatable. A requirement is kept only if one of
    /// its scenarios is selected.
    #[arg(long, value_name = "ID")]
    pub scenario: Vec<String>,
    /// Export only the scenarios in this task's declared impact (its modify,
    /// add-under and remove ids); exits 1 if the task declares none. Needs the
    /// daemon.
    #[arg(long, value_name = "TASK")]
    pub task: Option<String>,
}

#[derive(Debug, Args)]
pub struct SpecCheckArgs {
    /// Spec files, or directories searched recursively for `*.md`. Default:
    /// the `--root` directory.
    pub paths: Vec<PathBuf>,
    /// The specs directory checked when no paths are given (default
    /// `design/specs`, relative to the current directory), e.g.
    /// `openspec/specs` for a project not yet migrated.
    #[arg(long, value_name = "DIR")]
    pub root: Option<PathBuf>,
    /// Make a requirement without an id an error rather than a warning.
    #[arg(long)]
    pub require_ids: bool,
}

#[derive(Debug, Args)]
pub struct PrimeArgs {
    #[arg(value_enum)]
    pub role: PrimeRoleArg,
    /// Component to scope to (repeatable); worker/planner only. Defaults to `BRIDLE_COMPONENTS`.
    #[arg(long = "component", value_name = "ID")]
    pub components: Vec<String>,
    /// The task being worked; worker only. Its kind can add to the prime (an `explore`
    /// task gets the exploring agent's paragraph).
    #[arg(long, value_name = "ID")]
    pub task: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
pub enum PrimeRoleArg {
    Orchestrator,
    Advisor,
    Worker,
    Planner,
}

#[derive(Debug, Args)]
pub struct SpecIdArgs {
    /// Spec files, or directories searched recursively for `*.md`. Default: the `--root` directory.
    pub paths: Vec<PathBuf>,
    /// The specs directory used when no paths are given (default `design/specs`, relative to
    /// the current directory); its `.ids` file is the ledger of every id ever assigned.
    #[arg(long, value_name = "DIR")]
    pub root: Option<PathBuf>,
    /// The ledger file (default `<root>/.ids`).
    #[arg(long, value_name = "FILE")]
    pub ledger: Option<PathBuf>,
    /// Print what would change; write nothing.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct SpecCoverageArgs {
    /// The specs directory (default `design/specs`).
    #[arg(long, value_name = "DIR")]
    pub root: Option<PathBuf>,
    /// Directories to search for test sources (repeatable; default: `tests` and `test` if present).
    #[arg(long, value_name = "DIR")]
    pub tests: Vec<PathBuf>,
    /// Exit 1 if any executable scenarios are unbound; default exit 0.
    #[arg(long)]
    pub require_all: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// The project's clone. Defaults to the current directory.
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum WorkflowAction {
    /// Re-fetch the vendored workflow and print what changed.
    Update(WorkflowUpdateArgs),
}

#[derive(Debug, Args)]
pub struct WorkflowUpdateArgs {
    /// The project's clone. Defaults to the current directory.
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Fetch this tag of the bridle repo instead of the one matching this binary.
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// The project's clone. Defaults to the current directory.
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Project name, recorded as a comment (bridle takes the name from the directory).
    #[arg(long)]
    pub name: Option<String>,
    /// Integration branch. Defaults to the branch HEAD is on.
    #[arg(long)]
    pub integration: Option<String>,
    /// Workflow pack to enable (`packs = [STACK]`).
    #[arg(long, value_parser = ["python", "typescript", "rust"])]
    pub stack: Option<String>,
}

#[derive(Debug, Args)]
pub struct RestartArgs {
    /// Give up, restarting nothing, if the agents aren't all idle within this many seconds
    /// (the daemon's default is 600).
    #[arg(long)]
    pub wait: Option<u64>,
    /// First build the newest commit on main with green CI (`cargo install`, in the
    /// background), then restart into it; does nothing if none is newer than the last upgrade.
    #[arg(long)]
    pub upgrade: bool,
}

#[derive(Debug, Args)]
pub struct RebuildArgs {
    /// First fetch `origin/bridle/state`: fast-forward only, never overwrites a local branch
    /// that has state of its own (it says so and leaves both). Otherwise nothing is fetched.
    #[arg(long)]
    pub from_origin: bool,
}

#[derive(Debug, Args)]
pub struct LaunchdArgs {
    #[command(subcommand)]
    pub action: LaunchdAction,
}

#[derive(Debug, Subcommand)]
pub enum LaunchdAction {
    /// Write `~/Library/LaunchAgents/dev.bridle.<project>.plist` and print the launchctl commands.
    Install(LaunchdInstallArgs),
    /// Remove the plist and print the launchctl bootout command.
    Uninstall,
}

#[derive(Debug, Args)]
pub struct LaunchdInstallArgs {
    /// The clone's main checkout. Defaults to the current directory.
    #[arg(long)]
    pub repo: Option<PathBuf>,
    /// Defaults to the repo's parent directory.
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    /// Overwrite an existing plist.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct SystemdArgs {
    #[command(subcommand)]
    pub action: SystemdAction,
}

#[derive(Debug, Subcommand)]
pub enum SystemdAction {
    /// Write `~/.config/systemd/user/bridle-<project>.service` for `--project`, or for every
    /// project `[projects]` puts on this machine, and print the systemctl commands.
    Install(SystemdInstallArgs),
}

#[derive(Debug, Args)]
pub struct SystemdInstallArgs {
    /// Where the projects' clones live: each is `<dir>/<project>`, and `<dir>` is its
    /// workspace. Defaults to the current directory's parent.
    #[arg(long)]
    pub projects_dir: Option<PathBuf>,
    /// Overwrite existing unit files.
    #[arg(long)]
    pub force: bool,
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
    /// Claim a project whose state branch on origin names another host as owner. Run it after
    /// the old daemon has stopped and pushed.
    #[arg(long)]
    pub take_over: bool,
    /// Load the config and exit 0 (or fail): the daemon runs this on a freshly built binary
    /// before restarting into it.
    #[arg(long, hide = true)]
    pub check: bool,
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
    /// Grant this tool for this one spawn only, beyond the role's
    /// `allowed_tools` (docs/design/agent-host/roles-and-config.md). Repeat
    /// for more than one.
    #[arg(long = "allow-tool", value_name = "TOOL")]
    pub allow_tool: Vec<String>,
    /// Set an environment variable in this one spawn's process only, e.g. a
    /// secret (docs/design/agent-host/roles-and-config.md). Never written to
    /// `.bridle/config.toml` or the role. Repeat for more than one.
    #[arg(long = "env", value_name = "KEY=VALUE", value_parser = parse_env_kv)]
    pub env: Vec<(String, String)>,
    /// Skip the budget governor's holding/paused check for this one spawn.
    #[arg(long)]
    pub ignore_budget: bool,
    /// Scope the agent to this component (repeatable); sets `BRIDLE_COMPONENTS`.
    /// Default: the spawner's claimed task's list.
    #[arg(long = "component", value_name = "ID")]
    pub component: Vec<String>,
}

fn parse_env_kv(s: &str) -> Result<(String, String), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("expected KEY=VALUE, got `{s}`"))?;
    if key.is_empty() {
        return Err(format!("expected KEY=VALUE, got `{s}`"));
    }
    Ok((key.to_string(), value.to_string()))
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
    /// An agent id/name, `human`, or `role:<name>` for every live agent
    /// currently holding that role.
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
    /// About this task: the text is added as a note on its thread and the
    /// recipient gets a short message naming the task.
    #[arg(long)]
    pub task: Option<String>,
}

#[derive(Debug, Args)]
pub struct InboxArgs {
    #[command(subcommand)]
    pub action: Option<InboxAction>,
    /// Include already-read messages too (default: unread only). Only used by list.
    #[arg(long)]
    pub all: bool,
    /// Mark every listed message read. Only used by list.
    #[arg(long)]
    pub mark_read: bool,
}

#[derive(Debug, Subcommand)]
pub enum InboxAction {
    /// Show one message in full, including header and body, plus the reply command.
    Show(InboxShowArgs),
    /// Mark one or more messages read.
    Read(InboxReadArgs),
    /// Mark one or more messages unread again.
    Unread(InboxReadArgs),
}

#[derive(Debug, Args)]
pub struct InboxShowArgs {
    pub id: String,
    /// Mark the message read after showing it (by default reading it leaves it unread).
    #[arg(long)]
    pub mark_read: bool,
}

#[derive(Debug, Args)]
pub struct InboxReadArgs {
    /// One or more message ids to mark as read.
    #[arg(required = true)]
    pub ids: Vec<String>,
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
pub struct WaitArgs {
    pub task: String,
    /// Return when the task is in this state (at once if it already is);
    /// default: return on the next state change.
    #[arg(long)]
    pub until: Option<bridle_api::TaskState>,
    /// Also return when a message to me arrives (or is already unread).
    #[arg(long)]
    pub or_message: bool,
    /// Give up after this many seconds (exit 4).
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Args)]
pub struct UsageArgs {
    /// Group by role, model, or agent instead of the default per-agent breakdown.
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
    /// Print the whole resolved `[[budget.schedule]]` (every period, its
    /// span and thresholds) instead of the status.
    #[arg(long)]
    pub schedule: bool,
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
    /// Set a live `max_workers` cap (lost on daemon restart), or `--clear` it.
    /// Never stops running workers; only blocks new spawns and resumes.
    MaxWorkers(BudgetMaxWorkersArgs),
}

#[derive(Debug, Args)]
pub struct BudgetMaxWorkersArgs {
    /// The new cap.
    #[arg(required_unless_present = "clear", conflicts_with = "clear")]
    pub n: Option<u32>,
    /// Revert to the configured `max_workers`.
    #[arg(long)]
    pub clear: bool,
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
    /// Mint an `external:<name>` token (human only). It is saved in `credentials.toml` when
    /// the project is known, and printed only with `--print` (or when it can't be saved).
    Create {
        name: String,
        /// Mint a visitor, `external:<name>@<machine>`, for a principal on another machine.
        /// Always printed, to paste into that machine's `credentials.toml`.
        #[arg(long)]
        machine: Option<String>,
        /// Print the token even when it is saved.
        #[arg(long)]
        print: bool,
    },
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
    Incident,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum TaskSizeArg {
    S,
    M,
    L,
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum TaskPriorityArg {
    High,
    Normal,
    Low,
}

#[derive(Debug, Args)]
pub struct TaskPriorityArgs {
    pub task: String,
    #[arg(value_enum, ignore_case = true)]
    pub priority: TaskPriorityArg,
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
    /// Mark a task ready to build: `open` -> `planned`.
    Plan(TaskPlanArgs),
    /// Change a task's priority (high, normal, low); recorded in its thread and as an event.
    Priority(TaskPriorityArgs),
    /// Drop a task (requires a reason, recorded in its thread).
    Drop(TaskDropArgs),
    /// Mark a task integrated (merged), recording the merge commit in its thread.
    Done(TaskDoneArgs),
    /// Record how a task was implemented (replaces an earlier summary).
    Summary(TaskSummaryArgs),
    /// Bring a dropped or integrated task back.
    Reopen(TaskReopenArgs),
    /// Add a plain note to a task's thread (no question/answer semantics,
    /// doesn't affect readiness).
    Note(TaskNoteArgs),
    /// Search for tasks by words in title, body, or summary.
    Search(TaskSearchArgs),
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("body_source").args(["body", "body_file"])))]
pub struct TaskNewArgs {
    pub title: String,
    #[arg(short = 'k', long, value_enum)]
    pub kind: TaskKindArg,
    #[arg(long)]
    pub body: Option<String>,
    /// Read the task body from a file (or `-` for stdin).
    #[arg(long)]
    pub body_file: Option<PathBuf>,
    /// Scope the task to this component (repeatable); none = repo-wide.
    #[arg(long = "component", value_name = "ID")]
    pub component: Vec<String>,
    /// Estimated size, so small tasks can be picked when budget is short.
    #[arg(long, value_enum, ignore_case = true)]
    pub size: Option<TaskSizeArg>,
    /// A to-do for the human: created planned and claimed by them, with one inbox
    /// message pointing at it. Put `[at restart]` or `[at next reboot]` in the title
    /// when it must wait for one.
    #[arg(long)]
    pub for_human: bool,
    /// How soon it's wanted; ranks the human's to-dos (default normal).
    #[arg(long, value_enum, ignore_case = true)]
    pub priority: Option<TaskPriorityArg>,
}

#[derive(Debug, Args)]
pub struct TaskListArgs {
    /// Filter to tasks claimed by this principal: `me`, `human`, an agent
    /// name, or a full principal id.
    #[arg(long)]
    pub claimed_by: Option<String>,
    /// Only tasks naming this component or any descendant.
    #[arg(long, value_name = "ID")]
    pub component: Option<String>,
    /// Only tasks of this kind (`incident` lists the open incidents).
    #[arg(short = 'k', long, value_enum)]
    pub kind: Option<TaskKindArg>,
}

#[derive(Debug, Args)]
pub struct TaskShowArgs {
    pub task: String,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("body_source").args(["body", "body_file"])))]
pub struct TaskEditArgs {
    pub task: String,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    /// Read the task body from a file (or `-` for stdin).
    #[arg(long)]
    pub body_file: Option<PathBuf>,
    /// Replace the task's components with these (repeatable).
    #[arg(long = "component", value_name = "ID")]
    pub component: Vec<String>,
    /// Make the task repo-wide (clear its components).
    #[arg(long, conflicts_with = "component")]
    pub no_component: bool,
    /// Set the task's estimated size.
    #[arg(long, value_enum, ignore_case = true)]
    pub size: Option<TaskSizeArg>,
}

#[derive(Debug, Args)]
pub struct TaskPlanArgs {
    pub task: String,
}

#[derive(Debug, Args)]
pub struct TaskDropArgs {
    pub task: String,
    #[arg(long)]
    pub reason: String,
}

#[derive(Debug, Args)]
pub struct TaskDoneArgs {
    pub task: String,
    /// The merge commit. Optional only for a human to-do or an incident.
    #[arg(long, default_value = "")]
    pub commit: String,
    /// The branch that did the work.
    #[arg(long)]
    pub branch: Option<String>,
    /// For an incident: how it ended, sent to the agents that saw it.
    #[arg(long)]
    pub resolution: Option<String>,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("summary_source").required(true).args(["text", "file"])))]
pub struct TaskSummaryArgs {
    pub task: String,
    /// The summary text.
    #[arg(long)]
    pub text: Option<String>,
    /// Read the summary from a file (or `-` for stdin).
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct LandArgs {
    pub task: String,
    /// The branch to land; defaults to the claimant's.
    #[arg(long)]
    pub branch: Option<String>,
    /// Overrides `[integration] check`.
    #[arg(long, value_name = "CMD")]
    pub check_cmd: Option<String>,
    /// The commit the worker reported a green check on; a fast-forward landing of exactly this
    /// tip skips the check. Without it the check always runs.
    #[arg(long, value_name = "SHA")]
    pub checked_commit: Option<String>,
}

#[derive(Debug, Args)]
pub struct ProbeArgs {
    /// A claimed task id or an agent name.
    #[arg(conflicts_with = "branch", required_unless_present = "branch")]
    pub target: Option<String>,
    #[arg(long, value_name = "BRANCH")]
    pub branch: Option<String>,
}

#[derive(Debug, Args)]
pub struct ImpactArgs {
    #[command(subcommand)]
    pub action: ImpactAction,
}

#[derive(Debug, Subcommand)]
pub enum ImpactAction {
    /// Declare the task's impact, replacing any earlier declaration.
    Set(ImpactSetArgs),
    /// Print the task's declared impact.
    Show(ImpactShowArgs),
    /// Report overlaps between in-flight tasks' declared impact; exits 1 on a conflict.
    Check(ImpactCheckArgs),
}

#[derive(Debug, Args)]
pub struct ImpactCheckArgs {
    /// The specs directory read for the id -> capability map (default `design/specs`);
    /// unreadable specs skip the capability level.
    #[arg(long, value_name = "DIR", default_value = "design/specs")]
    pub specs: PathBuf,
}

#[derive(Debug, Args)]
pub struct ImpactSetArgs {
    pub task: String,
    /// Scenario or requirement id the task changes (repeatable).
    #[arg(long, value_name = "ID")]
    pub modify: Vec<String>,
    /// Requirement or capability id the task adds spec under (repeatable).
    #[arg(long, value_name = "ID")]
    pub add_under: Vec<String>,
    /// Spec id the task removes (repeatable).
    #[arg(long, value_name = "ID")]
    pub remove: Vec<String>,
    /// File glob the task touches (one or more).
    #[arg(long, value_name = "GLOB", num_args = 1..)]
    pub files: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ImpactShowArgs {
    pub task: String,
}

#[derive(Debug, Args)]
pub struct TaskReopenArgs {
    pub task: String,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("text_source").args(["text", "text_file"])))]
pub struct TaskNoteArgs {
    pub task: String,
    pub text: Option<String>,
    /// Read the note text from a file (or `-` for stdin).
    #[arg(long)]
    pub text_file: Option<PathBuf>,
    /// Also send this agent a short message naming the task (same as
    /// `bridle send <agent> --task <id>`).
    #[arg(long, value_name = "AGENT")]
    pub notify: Option<String>,
}

#[derive(Debug, Args)]
pub struct TaskSearchArgs {
    /// Search words; matches against title, body, and summary (all words must match, case-insensitive).
    pub words: Vec<String>,
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
pub struct OrchestratorArgs {
    #[command(subcommand)]
    pub action: OrchestratorAction,
}

#[derive(Debug, Subcommand)]
pub enum OrchestratorAction {
    /// The launcher's SessionStart hook: reads the hook JSON on stdin and records the session
    /// id and transcript path in `$BRIDLE_HOME/orchestrator.session`. Never fails.
    NoteSession,
}

#[derive(Debug, Args)]
pub struct FocusArgs {
    #[command(subcommand)]
    pub action: FocusAction,
}

#[derive(Debug, Subcommand)]
pub enum FocusAction {
    /// The UserPromptSubmit hook: inside a `quiet` period, prints context that nudges the human
    /// back to work on the first prompt and every 5 minutes after. Silent when no `[[focus]]`
    /// is configured, outside a period, or in a project with `focus_hours = false`. Never fails.
    Gate,
}

#[derive(Debug, Args)]
pub struct WaitForWakeArgs {
    /// The advisor's mail-only waiter: return when unread mail from the email bridge arrives
    /// (`nothing` after 25 minutes). `external:advisor`.
    #[arg(long)]
    pub mail: bool,
}

#[derive(Debug, Args)]
pub struct MailArgs {
    #[command(subcommand)]
    pub action: MailAction,
}

#[derive(Debug, Subcommand)]
pub enum MailAction {
    /// Poll the S3 inbound prefix and deliver the project's mail to the advisor or orchestrator;
    /// mail the human's questions and a daily digest through SES (`[mail]` in `~/.bridle/config.toml`; AWS credentials from the standard AWS chain).
    Run,
}

#[derive(Debug, Args)]
pub struct HandoverArgs {
    #[command(subcommand)]
    pub action: HandoverAction,
}

#[derive(Debug, Subcommand)]
pub enum HandoverAction {
    /// Record a new note (the newest one is what `prime orchestrator` prints). Only the human
    /// and `external:orchestrator`.
    Write {
        /// Read the note from this file, or `-` for stdin.
        #[arg(long)]
        file: PathBuf,
    },
    /// Say the state is written: the daemon stops this session and relaunches the orchestrator
    /// at once (orchestrator-supervision.md, section 6). Only the human and
    /// `external:orchestrator`. The marker only; `write` records the note.
    Done,
    /// List notes, newest first.
    List,
    /// Print one note.
    Show { id: String },
}

#[derive(Debug, Args)]
pub struct PortArgs {
    #[command(subcommand)]
    pub action: PortAction,
}

#[derive(Debug, Subcommand)]
pub enum PortAction {
    /// Allocate a free port from `[ports] range`; prints the number.
    Alloc(PortAllocArgs),
    /// Free a port you allocated.
    Release(PortReleaseArgs),
    /// List allocated ports.
    List,
}

#[derive(Debug, Args)]
pub struct PortAllocArgs {
    /// The process using the port; the daemon frees the port once it exits.
    #[arg(long)]
    pub pid: Option<i32>,
    /// What the port is for.
    #[arg(long)]
    pub label: Option<String>,
}

#[derive(Debug, Args)]
pub struct PortReleaseArgs {
    pub port: u16,
}

#[derive(Debug, Args)]
pub struct ConflictArgs {
    #[command(subcommand)]
    pub action: ConflictAction,
}

#[derive(Debug, Subcommand)]
pub enum ConflictAction {
    /// List conflicts, open ones first.
    List,
    /// Record how a conflict was settled: exactly one of the three flags.
    Resolve(ConflictResolveArgs),
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("how").required(true))]
pub struct ConflictResolveArgs {
    /// The conflict id, e.g. C12.
    pub id: String,
    /// Not a real conflict; the reason is recorded.
    #[arg(long, value_name = "REASON", group = "how")]
    pub compatible: Option<String>,
    /// `A,B`: adds a `blocks` edge, A blocks B.
    #[arg(long, value_name = "A,B", value_delimiter = ',', group = "how")]
    pub order: Option<Vec<String>>,
    /// One task absorbs the other's change.
    #[arg(long, value_name = "TASK", group = "how")]
    pub merge_into: Option<String>,
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
    /// Who to notify: an agent, `role:NAME`, `external:NAME` or `human`.
    /// Default: your spawner (an agent) or the human.
    #[arg(long)]
    pub to: Option<String>,
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
pub struct RulesArgs {
    #[command(subcommand)]
    pub action: RulesAction,
}

#[derive(Debug, Subcommand)]
pub enum RulesAction {
    /// Which layer wins a rule id, and what it shadowed.
    Explain(RulesExplainArgs),
    /// Everything the project layer (`<repo>/.bridle/rules`) does
    /// differently from the base and pack layers below it.
    Diff(RulesDiffArgs),
}

#[derive(Debug, Args)]
pub struct RulesExplainArgs {
    pub id: String,
    /// Resolve on top of this component's chain (L4, docs/design/components.md)
    /// instead of stopping at the project layer.
    #[arg(long)]
    pub component: Option<String>,
}

#[derive(Debug, Args)]
pub struct RulesDiffArgs {
    /// Diff the project layer against the layers below it. The only mode
    /// for now, so it's required rather than a silent default. Named
    /// `--project-layer`, not `--project` (docs/design/workflow-layers.md's
    /// own phrasing), because `--project` is already the global flag that
    /// selects a daemon by project name (docs/design/cli.md) and clap can't
    /// have both share that name with different types.
    #[arg(long)]
    pub project_layer: bool,
    /// Diff this component's chain against the layers below each of its
    /// components, instead of the project layer.
    #[arg(long)]
    pub component: Option<String>,
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

#[derive(Debug, Args)]
pub struct QueueArgs {
    #[command(subcommand)]
    pub action: Option<QueueAction>,
}

#[derive(Debug, Subcommand)]
pub enum QueueAction {
    /// Replace the whole queue: reorder, add or remove by resending the
    /// tiers in the shape they should be. PM (or human) only.
    Set(QueueSetArgs),
    /// Append one new tier, ranked after every existing one. PM (or human)
    /// only.
    AddTier(QueueAddTierArgs),
}

#[derive(Debug, Args)]
pub struct QueueSetArgs {
    /// One tier, as its task ids, comma-separated, in rank order. Repeat
    /// for more than one tier: `--tier tw-1,tw-2 --tier tw-3`.
    #[arg(long = "tier", value_name = "TASK,TASK,...", required = true)]
    pub tiers: Vec<String>,
}

#[derive(Debug, Args)]
pub struct QueueAddTierArgs {
    /// The new tier's task ids, equally ranked.
    #[arg(required = true)]
    pub tasks: Vec<String>,
}

#[derive(Debug, Args)]
pub struct PaneArgs {
    #[command(subcommand)]
    pub action: PaneAction,
}

#[derive(Debug, Subcommand)]
pub enum PaneAction {
    /// Set the @bridle tmux pane option to the given name.
    Tag { name: String },
    /// Clear the @bridle tmux pane option.
    Untag,
}

#[derive(Debug, Args)]
pub struct AdvisorArgs {
    #[command(subcommand)]
    pub action: AdvisorAction,
}

#[derive(Debug, Subcommand)]
pub enum AdvisorAction {
    /// Send the brief (if any) to `external:advisor` as "For advisor <name>: ...", then run
    /// `bridle session advisor <name>` in a tmux pane: a split of the orchestrator's window, or
    /// a new window (`[tmux] advisor_pane` in ~/.bridle/config.toml). Outside tmux, prints the
    /// command to run.
    Start {
        /// The advisor's name (letters, digits, `-` and `_`).
        name: String,
        /// The brief: text, or `@path` to read it from a file.
        #[arg(long)]
        brief: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub role: SessionRole,
}

#[derive(Debug, Subcommand)]
pub enum SessionRole {
    /// `bridle session orchestrator [--project <p>] [claude args]`. Records the pid and exit
    /// files the daemon's orchestrator supervisor reads.
    Orchestrator {
        /// Passed to `claude` as they are.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        claude_args: Vec<String>,
    },
    /// `bridle session advisor [--project <p>] [name] [claude args]`. A first argument is the
    /// advisor's name; the rest goes to `claude`.
    Advisor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Debug, Args)]
pub struct MachineArgs {
    #[command(subcommand)]
    pub action: MachineAction,
}

#[derive(Debug, Subcommand)]
pub enum MachineAction {
    /// Exit 1, saying why, if this clone is listed in `[machine] tools_only` of ~/.bridle/config.toml.
    ToolsOnlyCheck(ToolsOnlyArgs),
    /// Install pre-commit and pre-push hooks that refuse in a tools-only clone. Safe to re-run;
    /// refuses to overwrite a hook that isn't bridle's.
    ToolsOnlyInstall(ToolsOnlyArgs),
}

#[derive(Debug, Args)]
pub struct ToolsOnlyArgs {
    /// The clone (default: the current directory's repository).
    #[arg(long)]
    pub repo: Option<std::path::PathBuf>,
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
    fn spawn_allow_tool_is_repeatable_and_defaults_empty() {
        let cli = parse(&["spawn", "worker"]).unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert!(args.allow_tool.is_empty());

        let cli = parse(&[
            "spawn",
            "worker",
            "--allow-tool",
            "WebSearch",
            "--allow-tool",
            "WebFetch",
        ])
        .unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert_eq!(args.allow_tool, vec!["WebSearch", "WebFetch"]);
    }

    #[test]
    fn spawn_env_is_repeatable_and_defaults_empty() {
        let cli = parse(&["spawn", "worker"]).unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert!(args.env.is_empty());

        let cli = parse(&[
            "spawn",
            "worker",
            "--env",
            "PIXELLAB_TOKEN=abc123",
            "--env",
            "DEEPINFRA_TOKEN=def456",
        ])
        .unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert_eq!(
            args.env,
            vec![
                ("PIXELLAB_TOKEN".to_string(), "abc123".to_string()),
                ("DEEPINFRA_TOKEN".to_string(), "def456".to_string()),
            ]
        );
    }

    #[test]
    fn spawn_env_rejects_missing_equals() {
        let err = parse(&["spawn", "worker", "--env", "NOEQUALS"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn spawn_env_rejects_empty_key() {
        let err = parse(&["spawn", "worker", "--env", "=value"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn spawn_env_allows_value_with_embedded_equals() {
        let cli = parse(&["spawn", "worker", "--env", "URL=https://a.example/b=c"]).unwrap();
        let Command::Spawn(args) = cli.command else {
            panic!("expected spawn")
        };
        assert_eq!(
            args.env,
            vec![("URL".to_string(), "https://a.example/b=c".to_string())]
        );
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
    fn task_new_accepts_body_file() {
        let cli = parse(&[
            "task",
            "new",
            "title",
            "-k",
            "feature",
            "--body-file",
            "body.txt",
        ])
        .unwrap();
        let Command::Task(TaskArgs {
            action: TaskAction::New(args),
            ..
        }) = cli.command
        else {
            panic!("expected task new")
        };
        assert_eq!(args.title, "title");
        assert_eq!(args.body, None);
        assert_eq!(
            args.body_file.as_deref(),
            Some(std::path::Path::new("body.txt"))
        );
    }

    #[test]
    fn task_new_rejects_body_and_body_file_together() {
        let err = parse(&[
            "task",
            "new",
            "title",
            "-k",
            "feature",
            "--body",
            "text",
            "--body-file",
            "body.txt",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn task_new_body_file_accepts_dash_for_stdin() {
        let cli = parse(&["task", "new", "title", "-k", "feature", "--body-file", "-"]).unwrap();
        let Command::Task(TaskArgs {
            action: TaskAction::New(args),
            ..
        }) = cli.command
        else {
            panic!("expected task new")
        };
        assert_eq!(args.body_file.as_deref(), Some(std::path::Path::new("-")));
    }

    #[test]
    fn task_edit_accepts_body_file() {
        let cli = parse(&["task", "edit", "task-id", "--body-file", "body.txt"]).unwrap();
        let Command::Task(TaskArgs {
            action: TaskAction::Edit(args),
            ..
        }) = cli.command
        else {
            panic!("expected task edit")
        };
        assert_eq!(args.task, "task-id");
        assert_eq!(args.body, None);
        assert_eq!(
            args.body_file.as_deref(),
            Some(std::path::Path::new("body.txt"))
        );
    }

    #[test]
    fn task_edit_rejects_body_and_body_file_together() {
        let err = parse(&[
            "task",
            "edit",
            "task-id",
            "--body",
            "text",
            "--body-file",
            "body.txt",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn task_note_accepts_text_file() {
        let cli = parse(&["task", "note", "task-id", "--text-file", "note.txt"]).unwrap();
        let Command::Task(TaskArgs {
            action: TaskAction::Note(args),
            ..
        }) = cli.command
        else {
            panic!("expected task note")
        };
        assert_eq!(args.task, "task-id");
        assert_eq!(args.text, None);
        assert_eq!(
            args.text_file.as_deref(),
            Some(std::path::Path::new("note.txt"))
        );
    }

    #[test]
    fn task_note_rejects_text_and_text_file_together() {
        let err =
            parse(&["task", "note", "task-id", "text", "--text-file", "note.txt"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn task_note_text_file_accepts_dash_for_stdin() {
        let cli = parse(&["task", "note", "task-id", "--text-file", "-"]).unwrap();
        let Command::Task(TaskArgs {
            action: TaskAction::Note(args),
            ..
        }) = cli.command
        else {
            panic!("expected task note")
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
        let TokenAction::Create { name, .. } = t.action else {
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
    fn rules_explain_parses() {
        let cli = parse(&["rules", "explain", "kiss"]).unwrap();
        let Command::Rules(args) = cli.command else {
            panic!("expected rules")
        };
        let RulesAction::Explain(e) = args.action else {
            panic!("expected rules explain")
        };
        assert_eq!(e.id, "kiss");
    }

    #[test]
    fn rules_diff_project_layer_parses() {
        let cli = parse(&["rules", "diff", "--project-layer"]).unwrap();
        let Command::Rules(args) = cli.command else {
            panic!("expected rules")
        };
        let RulesAction::Diff(d) = args.action else {
            panic!("expected rules diff")
        };
        assert!(d.project_layer);
    }

    #[test]
    fn rules_diff_still_parses_the_global_project_flag() {
        let cli = parse(&["--project", "track-web", "rules", "diff", "--project-layer"]).unwrap();
        assert_eq!(cli.project.as_deref(), Some("track-web"));
        let Command::Rules(args) = cli.command else {
            panic!("expected rules")
        };
        let RulesAction::Diff(d) = args.action else {
            panic!("expected rules diff")
        };
        assert!(d.project_layer);
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
    fn task_summary_and_done_branch_parse() {
        let cli = parse(&["task", "summary", "tw-1234", "--text", "did it"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("not task")
        };
        assert!(
            matches!(args.action, TaskAction::Summary(a) if a.text.as_deref() == Some("did it"))
        );
        assert!(parse(&["task", "summary", "tw-1234"]).is_err());
        let cli = parse(&["task", "done", "tw-1", "--commit", "a", "--branch", "b"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("not task")
        };
        assert!(matches!(args.action, TaskAction::Done(a) if a.branch.as_deref() == Some("b")));
    }

    #[test]
    fn task_done_takes_an_optional_commit() {
        let cli = parse(&["task", "done", "tw-1234"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        assert!(matches!(args.action, TaskAction::Done(a) if a.commit.is_empty()));
        let cli = parse(&["task", "done", "tw-1234", "--commit", "abc123"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        assert!(matches!(args.action, TaskAction::Done(a) if a.commit == "abc123"));
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
    fn task_edit_size_none_parses() {
        let cli = parse(&["task", "edit", "tw-1234", "--size", "none"]).unwrap();
        let Command::Task(args) = cli.command else {
            panic!("expected task")
        };
        let TaskAction::Edit(a) = args.action else {
            panic!("expected task edit")
        };
        assert_eq!(a.task, "tw-1234");
        assert!(matches!(a.size, Some(TaskSizeArg::None)));
    }

    #[test]
    fn port_alloc_parses() {
        let cli = parse(&["port", "alloc", "--pid", "42", "--label", "web"]).unwrap();
        let Command::Port(args) = cli.command else {
            panic!("expected port")
        };
        let PortAction::Alloc(a) = args.action else {
            panic!("expected alloc")
        };
        assert_eq!((a.pid, a.label.as_deref()), (Some(42), Some("web")));
        assert!(parse(&["port", "release"]).is_err());
    }

    #[test]
    fn conflict_resolve_parses() {
        let cli = parse(&["conflict", "resolve", "C12", "--order", "tw-1,tw-2"]).unwrap();
        let Command::Conflict(args) = cli.command else {
            panic!("expected conflict")
        };
        let ConflictAction::Resolve(a) = args.action else {
            panic!("expected resolve")
        };
        assert_eq!(a.order, Some(vec!["tw-1".to_string(), "tw-2".to_string()]));
        assert!(parse(&["conflict", "resolve", "C12"]).is_err());
        assert!(
            parse(&[
                "conflict",
                "resolve",
                "C12",
                "--compatible",
                "x",
                "--merge-into",
                "a"
            ])
            .is_err()
        );
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
        assert!(matches!(cli.command, Command::Rebuild(_)));
    }

    #[test]
    fn handover_parses() {
        let cli = parse(&["handover", "write", "--file", "-"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Handover(HandoverArgs {
                action: HandoverAction::Write { .. }
            })
        ));
        assert!(parse(&["handover", "show", "h-0001"]).is_ok());
        assert!(parse(&["handover", "list"]).is_ok());
        assert!(matches!(
            parse(&["handover", "done"]).unwrap().command,
            Command::Handover(HandoverArgs {
                action: HandoverAction::Done
            })
        ));
    }

    #[test]
    fn wait_for_wake_parses() {
        let cli = parse(&["wait-for-wake"]).unwrap();
        assert!(matches!(cli.command, Command::WaitForWake(ref a) if !a.mail));
        let cli = parse(&["wait-for-wake", "--mail"]).unwrap();
        assert!(matches!(cli.command, Command::WaitForWake(ref a) if a.mail));
    }

    #[test]
    fn prime_orchestrator_parses() {
        let cli = parse(&["prime", "orchestrator"]).unwrap();
        let Command::Prime(args) = cli.command else {
            panic!("expected prime")
        };
        assert!(matches!(args.role, PrimeRoleArg::Orchestrator));
    }

    #[test]
    fn prime_rejects_unknown_role() {
        let err = parse(&["prime", "manager"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn prime_requires_a_role() {
        let err = parse(&["prime"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
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

    #[test]
    fn inbox_list_defaults_to_no_action() {
        let cli = parse(&["inbox"]).unwrap();
        let Command::Inbox(args) = cli.command else {
            panic!("expected inbox")
        };
        assert!(args.action.is_none());
        assert!(!args.all);
        assert!(!args.mark_read);
    }

    #[test]
    fn inbox_list_with_flags_parses() {
        let cli = parse(&["inbox", "--all", "--mark-read"]).unwrap();
        let Command::Inbox(args) = cli.command else {
            panic!("expected inbox")
        };
        assert!(args.action.is_none());
        assert!(args.all);
        assert!(args.mark_read);
    }

    #[test]
    fn inbox_show_parses() {
        let cli = parse(&["inbox", "show", "m-1234"]).unwrap();
        let Command::Inbox(args) = cli.command else {
            panic!("expected inbox")
        };
        let InboxAction::Show(show) = args.action.as_ref().expect("show action") else {
            panic!("expected show action")
        };
        assert_eq!(show.id, "m-1234");
        assert!(!show.mark_read);
    }

    #[test]
    fn inbox_show_with_mark_read_parses() {
        let cli = parse(&["inbox", "show", "m-1234", "--mark-read"]).unwrap();
        let Command::Inbox(args) = cli.command else {
            panic!("expected inbox")
        };
        let InboxAction::Show(show) = args.action.as_ref().expect("show action") else {
            panic!("expected show action")
        };
        assert_eq!(show.id, "m-1234");
        assert!(show.mark_read);
    }

    #[test]
    fn inbox_read_parses_single_id() {
        let cli = parse(&["inbox", "read", "m-1234"]).unwrap();
        let Command::Inbox(args) = cli.command else {
            panic!("expected inbox")
        };
        let InboxAction::Read(read) = args.action.as_ref().expect("read action") else {
            panic!("expected read action")
        };
        assert_eq!(read.ids, vec!["m-1234"]);
    }

    #[test]
    fn inbox_read_parses_multiple_ids() {
        let cli = parse(&["inbox", "read", "m-1234", "m-5678", "m-abcd"]).unwrap();
        let Command::Inbox(args) = cli.command else {
            panic!("expected inbox")
        };
        let InboxAction::Read(read) = args.action.as_ref().expect("read action") else {
            panic!("expected read action")
        };
        assert_eq!(read.ids, vec!["m-1234", "m-5678", "m-abcd"]);
    }

    #[test]
    fn inbox_read_requires_at_least_one_id() {
        let err = parse(&["inbox", "read"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn budget_help_parses_without_panic() {
        let cli = parse(&["budget", "--help"]).unwrap_err();
        assert_eq!(cli.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn budget_schedule_parses() {
        let cli = parse(&["budget", "--schedule"]).unwrap();
        let Command::Budget(args) = cli.command else {
            panic!("expected budget")
        };
        assert!(args.schedule);
        assert!(args.action.is_none());
    }

    #[test]
    fn status_help_parses_without_panic() {
        let cli = parse(&["status", "--help"]).unwrap_err();
        assert_eq!(cli.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn prime_help_parses_without_panic() {
        let cli = parse(&["prime", "--help"]).unwrap_err();
        assert_eq!(cli.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn task_help_parses_without_panic() {
        let cli = parse(&["task", "--help"]).unwrap_err();
        assert_eq!(cli.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn usage_help_parses_without_panic() {
        let cli = parse(&["usage", "--help"]).unwrap_err();
        assert_eq!(cli.kind(), clap::error::ErrorKind::DisplayHelp);
    }
}
