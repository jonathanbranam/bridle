# The daemon

```
  laptop (or the same machine)                     workspace host
 ┌───────────────────────────┐          ┌────────────────────────────────────────────────┐
 │ human: CLI / TUI / GUI     │  HTTP    │ bridle daemon   (bridle serve)                  │
 │   token: human             │ ───────► │  api    axum: /v1/*  JSON + SSE events         │
 │ orchestrator agent         │  + SSE   │  store  SQLite: principals, agents, messages,  │
 │   (Claude Code, remote-    │ ◄─────── │         events, usage                           │
 │    control) token:         │          │  supervisor ── one task per agent ──┐           │
 │    external:orchestrator   │          │  containment (pgid + ps scan)        │          │
 └───────────────────────────┘          └───────────────────────────────────┬──┼──────────┘
                                                stdin: user lines, control   │  │ stdout: stream-json
                                                 ┌───────────────────────────▼──┴──────────┐
                                                 │ claude -p  (manager)   cwd = <repo>     │
                                                 │ claude -p  (worker w1) cwd = wt/w1      │─ bridle CLI via Bash,
                                                 │ claude -p  (worker w2) cwd = wt/w2      │  token: agent:<id>
                                                 └─────────────────────────────────────────┘
```

- **The daemon is the only writer** of bridle state. The CLI never opens the
  database.
- **Agents talk back through the same API**, by running `bridle` with their
  own token, which bridle injects into their environment. A bridle MCP server
  exposing the same operations is planned, not built.
- **Agents are the daemon's children**, connected by pipes, so something
  long-lived has to own them. That is why bridle is a daemon and not only a
  CLI.

## Workspace layout

```
<workspace>/
  <repo>/                   the clone (main checkout): human's + manager's cwd
  wt/<agent>/               one worktree per agent that asked for one (default `[worktrees] layout`;
                            `root`/`paired` place them elsewhere, roles-and-config.md)
  integration/              the integrator's worktree, created by the first `bridle land`
  .bridle/
    daemon.json             {project, workspace, repo, url, pid, started_at, version}
    bridle.db               SQLite (WAL)
    tokens/human            the human's token (0600, in a 0700 directory)
    daemon.log              when detached
    state/                  the `bridle/state` branch's worktree (storage.md)
    agents/<id>/
      transcript.jsonl      every stdin/stdout/stderr line, as in spike 01
      system-prompt.md      the rendered --append-system-prompt-file
      token                 the agent's token (0600), for resume
```

`bridle serve` takes the repo from `--repo` or the cwd; it must be a git
repo. The workspace defaults to the repo's parent directory. The daemon refuses
to start if this workspace's daemon is already running, or if another
workspace has registered the same project name. The
state branch's worktree goes under `<workspace>/.bridle/state/`
([[docs/design/storage#The state branch|storage]]).

**Discovery.** A client finds the daemon from the first of these that is set:

1. `--url`;
2. `$BRIDLE_URL`;
3. `--project` / `$BRIDLE_PROJECT` via the registry
   ([[docs/design/agent-host/operating-model#Several projects at once|several projects]]);
4. `.bridle/daemon.json`, found by walking up from the cwd.

Worktrees and the clone sit inside the workspace, so the walk works from
anywhere in it. A daemon found by `--url` or `$BRIDLE_URL` has no local
workspace, so the client needs `$BRIDLE_TOKEN` too.

Two environment variables are for development and deployment:
`$BRIDLE_HOME` moves `~/.bridle` (the registry), and `$BRIDLE_CLAUDE_BIN`
names the `claude` executable the daemon runs.

## Running it

`bridle serve` runs in the foreground and logs to stderr. `bridle serve
--detach` re-executes itself in a new process group with output going to
`.bridle/daemon.log`, waits up to 60 s for `/v1/health` to answer, prints the
URL and pid, and exits. If the daemon is still starting then, it says so (pid, log path, `bridle daemons`), leaves it running and exits 0. The daemon installs its signal handlers before
anything else, and ignores SIGHUP. It is a new process group, not a new session (`setsid`
would need `unsafe`), and surviving a closed terminal is unverified:
[[detached-daemon-and-its-terminal-mnzh|spike mnzh]]. On a host that should
keep bridle running, run `bridle serve` in the foreground under a service
manager (systemd, launchd).

With a port from `[projects]` and no `--listen` or `[daemon] listen`, the daemon listens on loopback
and the Tailscale IPv4 address (`tailscale ip -4`). If Tailscale is not up at start (boot, a WSL
restart), it serves loopback, then re-checks every 5 s for 5 min and binds the Tailscale address on
the same port as soon as one appears, with no restart. If none appears it logs a warning and stays on
loopback until restarted.

A project has one serving machine ([[docs/design/storage#The state branch|storage]], "Ownership"):
`serve` fetches `origin/bridle/state` first and refuses to start when its `owner.toml` names
another host. `bridle serve --take-over` claims the project (after the old daemon stopped and
pushed). It also refuses unless origin was reached and `bridle/state` and the integration
branch fast-forward cleanly to it (errors name both SHAs; no override).

On Ctrl-C, SIGTERM or `POST /v1/shutdown`, stopping every running agent
([[docs/design/agent-host/agents#Stopping|agents, stopping]]) can take up to
`stop_grace` (default 30 s) plus 5 s. Right when the shutdown sequence
starts, the daemon logs one line at `info` (so it lands on stderr in the
foreground, and in `daemon.log` when detached) naming how many agents it's
stopping and the actual cap, so a slow shutdown doesn't look hung. The HTTP listener stays up (health still
answers) until cleanup is done, and `POST /v1/shutdown` replies with that cap, which is how
`bridle stop-daemon` reports progress.

## Background loops

Besides serving the API, the daemon runs: the stall and context checks (every 30 s), the process
tracker (2 s), the budget governor (30 s tick; polls usage every 5 min, 30 s above
`hold_at`: OAuth usage endpoint over HTTP, else a throwaway `claude -p` probe for that poll), the CI watcher (when `[ci] github`), the disk monitor (`[disk] check_interval`), the orchestrator supervisor (`[orchestrator] enabled`; 10 s; [[orchestrator-supervision]]), the orchestrator wake conditions (always; 10 s; the same design), the
task state-branch flush (30 s), the claim-lease check (30 s) and the port sweep (30 s; frees
ports whose owner is gone) and the document watcher (30 s), the outbox retry loop (15 s; below), plus the daily event prune. All stop on shutdown.

**Outbox retry** (3haz, br-fvkq; `outbox.rs`). Every 15 s the daemon looks at each destination with
queued mail and tries the oldest if its backoff has run out: the first try is at once (on send), then
after 30 s, 2 min, and every 5 min; mail never expires. The backoff is in memory, so a restart tries
everything at once. Two things cut a wait short: at start-up the daemon greets every daemon in
`[projects]` of the machine config that it holds a peer token for (`POST /v1/hello`), and a daemon
that hears a greeting flushes its queue for the greeter; and a wall clock that jumped more than 60 s
between two looks (the machine slept) counts as a start-up: greet the peers, try every queue at once.
The sender is told by a note from `system` when a message is refused for good (once, when it
fails) and when it has been queued 30 min (once; retries go on). Tests drive the loop on tokio's
paused clock with a fake transport.

## Scheduled messages

(hrcn, br-9xze; `schedule.rs`.) A stored message the daemon sends to a principal at a time, once or
on a cron. Slice 1: the mechanism, per project; role priming and other timed actions (nightly
restarts, maintenance windows, machine-wide schedules) are separate.

- **Storage**: the `schedules` table ([[storage]]). Ids are `sc-` and four characters. `target` is a
  principal as `bridle send` takes it. A fired `once` becomes `done`; a `cron` stays `active` with
  its next `next_fire_at`.
- **Cron**: standard 5 fields in the schedule's IANA zone (`--tz`, else `[schedule] timezone`, else
  `America/New_York`), own parser, `chrono-tz` for the zones (nothing in the lock file handled
  them). A local time that does not exist (spring forward) is skipped that day; an ambiguous one
  (fall back) fires once, at its first occurrence. A cron that never fires (31 Feb) is refused.
- **Firing**: a loop every 15 s (`schedule::TICK`), and once at start-up, sends each `active`
  schedule whose `next_fire_at` has passed through the normal message path, from `system`, as a
  note: `Scheduled <id> (set by <creator>): <body>`. So it queues, and wakes the agent like any
  message. A target that no longer exists is logged and the schedule moves on (a `once` ends), else it
  would fail every tick. A schedule lives on one project's daemon; its target is a principal of that
  daemon (no outbox hop).
- **Missed firings** (daemon down, machine asleep): a `once` fires once, now; a `cron` fires once,
  for its latest missed occurrence (never a burst), then moves to the next future one. More than
  2 minutes late, the body ends ` (due <local time>, sent late)`.
- **Who**: the human adds for anyone and lists/removes any; an agent only for itself, and lists and
  removes only its own; everyone else (externals, peers, visitors) is refused.
- **Events**: `schedule.fired` and `schedule.missed_fired`, data `{id, target}`; none per tick.
- **Not done**: quiet hours do not hold a schedule back.

## Document review

A document is under review when its repo-relative path is a line in `.bridle/review-documents.txt`
(`bridle review add|remove|list <path>`; the daemon re-reads the file each tick). The watcher
(`doc_watch.rs`) looks at each file in the main checkout. A comment thread (the callout format in
`workflow/base/roles/document-reviewer.md`) is *pending* when its newest entry is the human's and has no mark or `[pending]`; the human is
exactly `human` or `human via <agent>`, any other author is an agent. Resolved threads (a `resolved by` line) never go. When a document's pending threads have not changed for
`[review] quiet_minutes` (default 7), the pending threads go as one batch to that document's agent
(role `document-reviewer`, in the main checkout; named `doc-<id>` when the file stem ends in a ticket ID, else `doc-<slug>-<hash>` with the slug cut to fit the 40-character name limit and a hash of the path): spawned if there is none,
resumed if stopped, else sent as a message. Its own replies end the pending state, so it is not woken
for its own edits, and a restart doesn't resend answered threads. `[review] max_agents` (default 3)
caps document agents running at once (a document needing a start waits, still due, until a slot
frees); `[review] idle_hours` (default 4) stops an idle document agent, which resumes with its next
batch. The watcher ticks every 30 s; at debug level it logs why a document is held back (not still long enough), due, or waiting for a slot. Just before sending it re-reads the file and sends only if the pending threads are unchanged, comparing without thread IDs (ad3t: a document whose threads already had IDs was never sent).

**Marks, IDs and review now (ticket ehv6).** Each entry may end its first line with one ASCII
status, `[pending|sent|read YYYY-MM-DD HH:MM EDT]` (US Eastern with its zone; only the latest is
kept). Nothing parses the times: state is which mark is present. When bridle sends a batch it first
gives each open thread without one an ID (`c<n>`, highest in the file plus one), then writes
`[sent <now>]` on each thread's newest human entry. Each tick, a document's `[sent]` human entries
become `[read]` once its agent has no unread message (a message is read when the agent lists or
wakes on it; an agent started with the text as its prompt has none), so a thread stuck at `sent`
means the agent is busy, down or out of budget. The same tick rewrites the old
` U+00B7 sent YYYY-MM-DD HH:MM` marks to the new form. A mark doesn't change who a thread's last
author is. The quiet-period send skips a thread whose newest entry is `sent` or `read`, and a
later human reply is unmarked, so pending again. `bridle review resolve <path> c3` appends
`resolved by human, <time>` to a thread. The file is edited in the working tree and not committed by bridle: the agent's next commit carries
the marks. `POST /v1/review/now` (`ReviewNowRequest {path, resend}` → `{path, agent, threads}`;
`bridle review now <path> [--resend]`; the gateway's `POST /api/v1/projects/{project}/review`)
sends the document's unmarked pending threads at once (all pending ones with `resend`), skipping
the quiet period and the agent cap, then marks them (a resent thread's old mark is replaced).
`POST /v1/review/add` (`ReviewAddRequest {path, only_if_pending}` → `{path, under_review}`) is
`bridle review add` over the API, for the gateway: the path must be a plain repo-relative existing
file (400 otherwise), and with `only_if_pending` a document with no pending thread is left alone.
`threads: 0` means nothing was unsent. The path must be one registered for review (400
otherwise). A tick and a review-now never send the same thread twice (one lock around read,
deliver and mark). Design: x8jt.

## Restart and recovery

If the daemon dies, each agent's stdin reaches EOF and the agent exits after
its current turn. Agents don't survive a restart and aren't designed to.

A clean shutdown (Ctrl-C / SIGTERM / `POST /v1/shutdown`) stops every running
agent through the normal `stop()` path
([[docs/design/agent-host/agents#Stopping|agents, stopping]]), but tags each
of those stops as shutdown-triggered, so it exits `stopped` with reason
`daemon_shutdown` instead of the usual `sigterm`/`stdin_closed`. On the next
`bridle serve` startup, `stopped` agents with that reason are treated the
same as `lost` for the resume step below, so a `resume_on_restart` role comes
back after a clean restart, not only after a crash.

`bridle serve` also runs `claude auth status` in the background at start-up (5 s limit) and logs a
warning with the fix if claude isn't logged in (nrbf). A missing claude, a timeout or any error is
"unknown" and silent; it never delays or fails start-up.

On startup the daemon reconciles:

- For every agent recorded as running, it kills the process group if the pid
  and start time still match, sweeps that process's current descendants
  ([[docs/design/agent-host/agents#Containment|containment]]), and marks the
  agent `lost` with reason `daemon_restart`, with `agent.state` and
  `agent.exited` events. The seen set isn't persisted, so a tool process that
  had already re-parented is missed.
- Written-but-unacked and held messages go back to `pending`.
- Every agent whose role has `resume_on_restart` (the default for the manager
  and orchestrator roles) is resumed with `--resume`, if it's either `lost`
  (the case above) or `stopped` with reason `daemon_shutdown` (a clean
  shutdown before this restart).
- For every role with `autostart = true` (by default only the built-in `manager`; bridle's own `project-manager` sets it in config), an agent named after the role is
  spawned with the role's `start_prompt`, unless an agent with that *role* already
  exists in any state (so a manager named `manager-2` suppresses it). Autostart goes through the normal spawn, so a
  budget `hold_at` refuses it (logged; the role isn't retried until the next start).
- Events older than 30 days are pruned, then daily.

Whether workers should resume too after a *crash* or plain restart is open:
[[do-workers-resume-after-a-daemon-restart-2fkk|do workers resume after a restart]].

### Restart in place

`POST /v1/restart` (`bridle restart`; the human and `external:orchestrator` only) upgrades the
daemon without the human: it **drains** first. From the request on the daemon is `draining`: spawns
are refused, task claims are refused, and no running agent gets a new turn. Messages, task updates
and comments, queue nudges and wakes are still stored (state `held`, in order) but not written to
the agent; the end of a turn no longer delivers the next held one. Turns already in progress run to
their end and nothing is cut off; the queue stays as it is. Interactive sessions (orchestrator,
aide, advisors) are outside the daemon's turns and are not held. There is no timeout and no 409:
when no agent is mid-turn and no spawn is in flight (checked every 500 ms) it records the running agents' ids
(`meta` key `restart.resume`), wakes the orchestrator (`restart`, with the commit), sets the
restart flag and runs the ordinary shutdown sequence above (agents stop as `daemon_shutdown`, the
state branch is flushed and pushed, `daemon.json` removed). `run` then `exec`s the binary path resolved once at start-up (`exe_path()`: Linux's `<path> (deleted)` suffix, left after a reinstall, is stripped) with
the same args (safe Rust, `CommandExt::exec`), so the PID and the terminal stay and Ctrl-C still
works; the new process rebinds the same `listen` address and clients retry through the gap. If the
exec fails (and no upgrade is pending a rollback) the daemon does not exit: the shutdown has already
closed the listener and stopped the agents, so `run` starts the daemon again in the same process
(`start_after_failed_restart`). That start listens anew, resumes the agents the restart recorded,
and starts with no drain, so the messages held meanwhile (kept in the store) are delivered. The error is
logged, recorded as a `restart.failed` event and woken to the orchestrator (`restart_failed`).

A spawn already in flight when the drain begins has its first prompt held like any message, so no
turn starts; the spawn then returns at once rather than waiting out its 8 s readiness wait, and the
prompt is delivered after the restart's resume. If the restart fails before the exec, the drain is
lifted and each running agent gets its oldest held message (the end of its turn chains the rest).

The next start, after its own resume of `resume_on_restart` roles, reads and clears the record and
resumes every recorded agent still not running, workers too, each with a note from `system` that the
daemon restarted for an upgrade, to carry on and to re-run any background job it was waiting on
(an idle agent's own shell job dies with the restart: w8bz). The held messages went back to
`pending` when the agents stopped, so the ordinary resume delivers them, oldest first;
none are lost or duplicated. It wakes the orchestrator (`restart`: commit, who
resumed, who failed); the human's inbox gets a message only if some agent failed to resume.

#### Upgrade

`bridle restart --upgrade` (`upgrade: true` in the request) builds before it restarts. The daemon
walks the integration branch's first-parent history (30 commits back) for the newest commit whose
GitHub Actions runs (the CI watcher's `gh` calls, `Gh::runs`) have all finished green; unpushed
commits have no runs and are skipped. The commit the last upgrade built is kept in `meta` key
`upgrade.built`; if the candidate is that commit or an ancestor of it there is nothing newer, and
the reply says so and nothing happens (a daemon that has never upgraded builds the newest green
commit; the binary carries no commit of its own). Otherwise the reply comes at once
(`restarting: false`, `message: "building <sha> ..."`) and the rest runs in the background, one
upgrade at a time: event `upgrade.building` (no wake), check the commit out into a throwaway detached worktree
(`<workspace>/.bridle/upgrade-src`; the human's checkout is never touched), run `cargo install
--path crates/bridle` there at normal priority with `CARGO_TARGET_DIR=<workspace>/.bridle/upgrade-target`
(kept between upgrades so builds are incremental; one-hour cap), then restart in place as above,
after the drain above (event `upgrade.draining`; `bridle status` shows `upgrade <sha> draining;
waiting on <agents mid-turn>`), recording the commit as built. A newer commit landing during the
drain does not rebuild: the restart uses the built commit. A failed build or self-check leaves the
running daemon untouched (the drain is a restart's, so it starts only after both passed): event
`upgrade.failed`, wake `upgrade_failed` (with the build output's last lines) and a note to the
human's inbox. Out of scope: other projects' daemons.

**Events, and what wakes.** Every step is an event (`bridle events --kind upgrade.`):
`upgrade.skipped` (`{commit, reason}`), `upgrade.building`, `upgrade.built`, `upgrade.draining`,
`upgrade.failed` (`{commit, error}`) and `upgrade.rolled_back` (`{error}`). Only what needs
attention wakes the orchestrator: the `restart` wake of a successful upgrade, `upgrade_failed` for
a real failure (build, self-check, rollback) which also notes the human, and `upgrade_draining`,
once, when a drain (an upgrade's or a plain restart's) is still waiting after an hour
(`{agents, spawning}`; the text names the agents still in a turn). Nothing else escalates: a stuck
turn is the stall detector's job. Skipped and building wake no one.

**Rollback.** The running binary is copied to `<workspace>/.bridle/bridle.prev` before the build
replaces it. After the build, the daemon runs the new binary's self-check (`bridle serve --check
--repo .. --workspace ..`: loads the config, never opens the database, so a newer schema isn't
applied before the restart is certain; a run that times out at 60 s is retried, up to three runs, since the new binary's first run can be slow on a loaded Intel Mac, while a real failure is not); a failure is reported like a failed build and the daemon
stays as it is. Before the restart it writes `.bridle/upgrade-pending.json`; the new process marks
it `started` as it begins and deletes it once `daemon.json` is written (serving). A new process
that fails in `start`, or finds the marker already `started` (the last attempt died before
serving, e.g. a crash), copies `bridle.prev` back over the installed binary, leaves
`upgrade-rolled-back.txt` and execs it; the restored daemon wakes the orchestrator (`upgrade_failed`,
stage `rolled_back`; event `upgrade.rolled_back`). A failed exec of the new binary rolls back the same way. Exec in place means
nothing supervises the new process: a hard crash before serving is only caught by the next start
(by hand), which then rolls back.

**Docs-only commits skip the build.** Before building, the daemon runs `git diff --name-only
<built>..<candidate>`. If nothing under `crates/`, `.cargo/`, `Cargo.toml`, `Cargo.lock` or
`rust-toolchain*` changed (the binary embeds no workflow or docs files outside tests), it records the
candidate as built, records `upgrade.skipped` (no wake) and does not restart. With no built commit yet, or if
git can't diff, it builds. **The restart's commit** (message to agents, `restart` wakes) is the
built commit (`upgrade.built`), not the integration head, which can have moved during the build.

**Automatic upgrade.** With `[daemon] self_upgrade = true` (default off; on in bridle's own
`.bridle/config.toml`) the CI watcher's tick (every minute; no loop of its own) looks for a newer
green commit exactly as above, whatever the agents are doing (the build needs no quiet point), and,
if there is one and no restart or upgrade is under way, starts the same background upgrade: build,
then drain, then restart. A turn is never cut off and nothing gives up waiting. A commit whose
upgrade failed (build or self-check) is not retried (in memory; a daemon restart or a newer commit
tries again) so a broken build doesn't loop.

**Batching (`[daemon] self_upgrade_min_interval`, default `3h`).** The automatic upgrade waits
this long after the last upgrade, then takes the newest green commit, so the landings in between go
in as one batch. The last upgrade's time is the newest stored `upgrade.built` event (not memory, so
it survives the restart the upgrade causes; not `daemon.started`, which a crash emits too). A daemon
that has never upgraded has no wait. While inside the interval a newer green commit is logged once
and not built. `0s` restores the old upgrade-at-once behaviour. An explicit `bridle restart
--upgrade` ignores the interval, which is how a critical fix goes through at once; a
critical-priority landing does not trigger an upgrade by itself (not yet needed: the orchestrator
runs the explicit command; ticket 7ufd).

**The gateway follows the upgrade by itself.** The upgrade replaces the installed `bridle` file, and
a running `bridle gateway` re-executes itself when it sees that file change (human-web-ui.md,
'Running it detached, and staying current'). The daemon does nothing for it: no hook, no quiet
point (the gateway holds no agent turns), and it works on machines with no daemon.

While a built commit waits for its quiet point (manual or automatic), new worker spawns are refused
with a 409 naming the build, and `bridle status` shows an `upgrade` line, so running workers drain
instead of being replaced as they land. If the wait gives up, the refusal is lifted; a successful
restart never lifts it (the process is exiting).

## Crates

```
crates/
  bridle-claude/   the stream-json client: AgentProcess, Event model, transcript writer.
                   No knowledge of the daemon. Spike 01's code, hardened, with its
                   fixtures as parser tests.
  bridle-api/      API types (requests, responses, events) + an async HTTP client
                   with SSE, and daemon discovery. Shared by the daemon, the CLI,
                   and future TUI/GUI/MCP.
  bridle-daemon/   store, supervisor, containment, worktrees, config, axum server.
  bridle-tui/      `bridle tui`; depends only on bridle-api (cli.md, `tui`).
  bridle-gateway/  `bridle gateway`, the human web UI's API (human-web-ui.md).
  bridle-mail/     `bridle mail run`, the email bridge (mail.md).
  bridle-spec/     the spec-file parser behind `bridle workflow spec` (specs.md).
  bridle/          the binary: clap CLI; `serve` runs bridle-daemon.
```

An MCP server (planned, not built) would be another crate depending only on `bridle-api`.
