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
  own token, which bridle injects into their environment. Later they can use a
  bridle MCP server exposing the same operations.
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
`.bridle/daemon.log`, waits up to 15 s for `/v1/health` to answer, prints the
URL and pid, and exits. The daemon installs its signal handlers before
anything else, and ignores SIGHUP. It is a new process group, not a new session (`setsid`
would need `unsafe`), and surviving a closed terminal is unverified:
[[detached-daemon-and-its-terminal-mnzh|spike mnzh]]. On a host that should
keep bridle running, run `bridle serve` in the foreground under a service
manager (systemd, launchd).

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
tracker (2 s), the budget governor (30 s tick; polls `get_usage` every 5 min, 30 s above
`hold_at`), the CI watcher (when `[ci] github`), the disk monitor (`[disk] check_interval`), the orchestrator supervisor (`[orchestrator] enabled`; 10 s; [[orchestrator-supervision]]), the orchestrator wake conditions (always; 10 s; the same design), the
task state-branch flush (30 s), the claim-lease check (30 s) and the port sweep (30 s; frees
ports whose owner is gone), plus the daily event prune. All stop on shutdown.

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
- For every role with `autostart = true` (by default only the built-in `manager`; bridle's own `product-manager` sets it in config), an agent named after the role is
  spawned with the role's `start_prompt`, unless an agent with that *role* already
  exists in any state (so a manager named `manager-2` suppresses it). Autostart goes through the normal spawn, so a
  budget `hold_at` refuses it (logged; the role isn't retried until the next start).
- Events older than 30 days are pruned, then daily.

Whether workers should resume too is open:
[[do-workers-resume-after-a-daemon-restart-2fkk|do workers resume after a restart]].

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
  bridle/          the binary: clap CLI; `serve` runs bridle-daemon.
```

A TUI will be another crate that depends only on `bridle-api`, and so will an
MCP server.
