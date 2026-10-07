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
ports whose owner is gone) and the document watcher (30 s), plus the daily event prune. All stop on shutdown.

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
daemon without the human: the request waits for a quiet point (every running agent `idle`, checked
every 500 ms, up to `wait_secs`, default 600). At the timeout it answers 409 naming the busy agents
and does nothing: work is never cut off. At a quiet point it records the running agents' ids
(`meta` key `restart.resume`), wakes the orchestrator (`restart`, with the commit), sets the
restart flag and runs the ordinary shutdown sequence above (agents stop as `daemon_shutdown`, the
state branch is flushed and pushed, `daemon.json` removed). `run` then `exec`s the binary path resolved once at start-up (`exe_path()`: Linux's `<path> (deleted)` suffix, left after a reinstall, is stripped) with
the same args (safe Rust, `CommandExt::exec`), so the PID and the terminal stay and Ctrl-C still
works; the new process rebinds the same `listen` address and clients retry through the gap. If the
exec fails the daemon stays cleanly stopped, as after `stop-daemon`.

The next start, after its own resume of `resume_on_restart` roles, reads and clears the record and
resumes every recorded agent still not running, workers too, each with a note from `system` that the
daemon restarted for an upgrade and to carry on. It wakes the orchestrator (`restart`: commit, who
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
with the same quiet-point wait, recording the commit as built. A failed build or self-check, or
(for a manual upgrade) no quiet point after the build, leaves the running daemon untouched: event
`upgrade.failed`, wake `upgrade_failed` (with the build output's last lines) and a note to the
human's inbox. Out of scope: other projects' daemons.

**Events, and what wakes.** Every step is an event (`bridle events --kind upgrade.`):
`upgrade.skipped` (`{commit, reason}`), `upgrade.building`, `upgrade.built`, `upgrade.waiting`
and `upgrade.gave_up` (`{commit, busy, error}`), `upgrade.failed` (`{commit, error}`) and
`upgrade.rolled_back` (`{error}`). Only what needs attention wakes the orchestrator: the `restart`
wake of a successful upgrade, `upgrade_failed` for a real failure (build, self-check, rollback,
manual no-quiet-point) which also notes the human, and `upgrade_failed` (no human note) when the
automatic upgrade has found no quiet point for three hours. Skipped and building wake no one.

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
`.bridle/config.toml`) the CI watcher's tick (every minute; no loop of its own) also checks for a
quiet point: no running agent mid-turn (a budget hold winds workers down to idle or stopped, which
counts). If so, and no restart or upgrade is under way, it looks for a newer green commit exactly as
above and, if there is one, starts the same background upgrade (build, restart with
the ten-minute quiet-point wait), so a turn is never cut off. A commit whose upgrade failed is not
retried (in memory; a daemon restart or a newer commit tries again) so a broken build doesn't loop.
A wait that ends with agents still busy is not a failure: `upgrade.waiting`, no wake, no human
note, and the next quiet tick builds (incrementally) and tries again. After three hours of that for
one commit it records `upgrade.gave_up`, wakes `upgrade_failed` once and stops retrying it.

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
