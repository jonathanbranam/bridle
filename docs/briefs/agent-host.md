# Brief: the agent host and daemon

As of 2026-09-29. Status words: **built** (in the code and CHANGELOG), **partly built**,
**planned** (design docs only). Companion to [[docs/briefs/specs|specs]] and
[[docs/briefs/tasks|tasks]]. Source of truth for the details: `docs/design/agent-host/`.

## What it is

One daemon per project (`bridle serve`) owns every agent as a child process: each agent is a
headless `claude -p` speaking stream-json over pipes. The daemon is the only writer of state
(SQLite `bridle.db`, plus a state branch in git for tasks). The CLI, the TUI and the agents
themselves all talk to it over local HTTP with a token; agents run `bridle` from their own shell
with their own token. **Built.**

## The daemon and how it starts

`bridle init` scaffolds `.bridle/config.toml`; `bridle doctor` checks the setup and says what to
fix. `bridle serve` runs in the foreground (`--detach` backgrounds it). On macOS,
`bridle launchd install` writes a per-project LaunchAgent that runs `bridle serve`,
restarting it only after a crash (a deliberate `bridle stop-daemon` stays stopped); it prints the
`launchctl` commands and never runs them. `bridle daemons` lists every project daemon on the
machine. **Built** (macOS only).

On start the daemon marks agents that were running as `lost`, resumes roles set to
`resume_on_restart` (manager and orchestrator by default; not workers), and autostarts roles with
`autostart` (the manager by default; bridle's own config adds the product manager). A clean
shutdown stops every agent first, so a restart brings the same roles back. **Built.** Workers do
not come back after a restart; whether they should is open (`2fkk`). It also runs background loops:
stall and context checks, budget governor, disk monitor, claim leases, port sweep, a CI watcher
(only with `[ci] github = true`). **Built.**

## Spawning and worktrees

`bridle spawn` starts an agent in a role. A worker gets its own worktree at `wt/<name>` on branch
`bridle/<name>`, cut from the role's base; manager and orchestrator work in the clone. Config can
run a setup command in each new worktree, copy files like `.env`, clone the `target/` cache
(macOS), place worktrees elsewhere, or create paired sibling-repo worktrees. A failed spawn is
rolled back. `bridle rm` stops the agent and removes the worktree, refusing if it is dirty, has
open files or holds an unmerged branch unless `--force`. Ports for dev servers come from
`bridle port`. **Built.** `max_workers` (default 2) caps *worker* spawns and resumes (409 at the
cap); change it live with `bridle budget max-workers`. **Built.**

## Agent states

`starting`, `idle`, `working`, `stopping`, then the ends: `stopped` (we closed it), `exited` (it
ended on its own), `crashed`, `lost` (daemon restarted). The last four can be resumed
(`bridle resume`, same session and cost counters). `bridle interrupt` ends the current turn.
An agent silent for 10 minutes while working gets a stall event (only an event; nothing acts on it).
**Built.**

## Messages and the inbox

`bridle send <agent|human|role:<name>|external:<name>>` queues a message. To a live agent it is
written to stdin at once and folded into the current turn, or with `--when idle` held until the
turn ends. Delivery is acknowledged by claude's echo, and an unacked message goes back to pending
and is redelivered on resume. Messages to the human, or to an orchestrator's own principal, wait
in `bridle inbox`. A reply from a principal listed in `[messages] answer_for_human` (the
orchestrator by default) closes the human's message as "answered by ...", which you can overrule.
The daemon also sends system notes: context handoff, "main moved", "task filed", conflicts, budget
notices. Messages live in SQLite only, not on the state branch. **Built.**

## Containment

Each agent leads its own process group. Every 2 seconds the daemon records its descendants from
the process table; `bridle stop` closes stdin, then SIGTERM, then SIGKILL, then sweeps anything
recorded (tool processes live in their own groups and would otherwise be orphaned).
**Built for macOS and Linux via `ps`.** A process that double-forks between two scans can
escape. The `Containment` trait for Linux cgroups exists but is unused (**planned**).
Agents run with `--permission-prompts none`: anything that would prompt is denied, not asked.

## The budget governor

Account-wide, driven by the rate-limit windows Claude reports (five-hour, seven-day). States:
`normal`, `holding`, `winding_down`, `paused`. Defaults, on the five-hour window: **hold at 80%**
(no new spawns, resumes or messages to idle agents), **wind down at 90%** (working agents are told
to finish, 85% on `seven_day_opus`), **stop at 95%** (pause: agents stopped), resume below 70%. A
project may lower a threshold, never raise it. Stale readings (over 10 minutes) count against
you: hold, then wind down. `bridle budget` shows the state and why; `budget --schedule` and
`budget override <period>` swap thresholds by time of day (`[[budget.schedule]]`). `budget hold` and
`release` idle the whole account for you. After a pause the manager, PM and orchestrator resume
first. All **built**. The cross-daemon sharing of one budget is **planned** (`xypj`, `hb0q`).

## Context governing and renewal

Each agent's context size is measured every turn (`bridle status`, `bridle statusline`). Past a
per-role threshold (default 120k tokens for workers, 200k for others) it gets a "Context handoff:"
note, and is renewed when that turn ends or after 5 minutes regardless. `bridle renew` does the
same by hand: stop, then start a fresh session in the same worktree, branch, role and model,
with a note pointing at the task thread. A renew is not blocked by a budget hold. **Built.**
Watching the orchestrator's own context is only a wake-up (`CONTEXT <tokens>` at 140k) in the
launcher script; **partly built** (`c9zm`).

## Roles, permissions and the Stop hook

Built-in roles: `worker` (edits files, `acceptEdits`), `manager` and `orchestrator` (`dontAsk`,
mostly `bridle` and `git` commands, read-only tools); the PM is set in bridle's config. Claude
Code's own messaging, scheduling and remote-trigger tools are denied by default; memory is off;
agents load only project settings. Roles come from `.bridle/config.toml` and the shared workflow
layers (`bridle sync` renders them into `CLAUDE.md`, skills and hooks). Enforcement is by role in
a few places: workers may not use agent lifecycle endpoints, only the PM or human edits the queue,
and budget hold and token management are human-only. Otherwise permissions are prompts plus
tool allowlists, not a general system. **Built.**

Hooks: **`bridle stop-check`** (Stop hook, worker role) blocks a worker from ending while it holds
a claimed task with no thread note, no summary or `done:` report, or (when the project sets
`commands.check_worker`) no recorded passing check for its HEAD. **`bridle arch-guard`** denies
edits under `design/architecture/` unless the worker claimed an `arch-revision` task. Both fail
open on their own errors. **Built.**

## Usage, status and the TUI

`bridle status` (daemon, agents, budget, last CI result, merged-but-not-removed branches),
`bridle usage [--by ...]` (cost, tokens, busy and wall time per agent), `bridle logs <agent>`
(readable transcript), `bridle events [--follow]`, `bridle cost` (audit of what bridle injects
into agent context). `bridle tui` shows the agents list, live events and the inbox (open, reply,
scroll). It has no panels for tasks or the queue (`y496`). **Built**, TUI **partly**.

## What you touch

Read: `status`, `inbox`, `agents`, `logs`, `queue`, `budget`, `tui`. Act: `send` or `inbox` replies,
`budget hold|release|max-workers`, `spawn`, `stop`, `resume`, `renew`, `rm`, `token create`,
`stop-daemon`. The human's inbox is meant for questions, blockers and decisions only.
The orchestrator stays running through a launcher script and an in-session watcher; **partly
built**, and the pane-tag supervision inside bridle is **planned** (`fx7x`).

## Gaps and rough edges

- Workers don't resume after a daemon restart; a worker mid-task is `lost` and needs a manager to
  resume or renew it.
- Cost and rate-limit numbers come only from what Claude reports; a hold at 80% can strand
  queued work until the window resets, and there is no maintenance work during holds (`m7wn`).
- A stall is reported, not acted on. An agent idle with a live background task looks finished (`w8bz`).
- Containment can miss double-forked processes; a closed terminal after `serve --detach` is
  unverified (use launchd).
- Retention: events prune at 30 days; transcripts and worktrees are never pruned (`34wz`).
- `bridle land` with a worktree that has the integration branch checked out needs it clean.
- Claude Code updates itself; the daemon notices a new version but only the manual
  `just test-contract` decides if it still works.
- Agents can't ask permission questions; a denied tool just fails.

## Where design docs disagree with the code

- `docs/design/agent-host/daemon.md` says to run under launchd yourself; `bridle launchd install`
  now writes the plist (CHANGELOG, br-936d).
- `docs/design/cli.md` ("rebuild") and `tasks.rs` say claims are lost on rebuild; the code restores
  them (noted in the tasks brief).
- `docs/design/usage-and-budget.md` is much longer than what is built: cross-daemon budget sharing
  and some maintenance-during-hold ideas are ideas only.

## Decisions for you

1. Should workers resume after a restart, or should a manager re-spawn them?
2. Are 80/90/95 the right budget thresholds, and 120k the right worker context limit?
3. Is macOS-only `launchd install` enough, or do you need a Linux service file (the NUC host)?
