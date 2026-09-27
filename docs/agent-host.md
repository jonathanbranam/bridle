# Bridle — the daemon and the agent host

*Written 2026-09-27, from [`design.md`](design.md) §6 and §11,
[`research/01-agent-runtime.md`](research/01-agent-runtime.md) and the evidence in
[`spikes/01-stream-json-findings.md`](spikes/01-stream-json-findings.md). This
doc covers the part of bridle that runs agents: the long-running daemon, the
API every client uses, and the supervisor that hosts headless Claude Code.*

> **Status: design for v1, being built.** Where this doc and `design.md`
> disagree, this doc is newer and says why (§1.2). Everything here is a
> recommendation to be argued with. §13 lists what is still open.

---

## 1. Scope

### 1.1 What v1 delivers

Start bridle against a clone, in the foreground or as a background daemon.
Then drive it from a CLI that the human, the human's own orchestrator agent,
the agents bridle hosts, and later a TUI, GUI or MCP server all share:

- **spawn** headless Claude Code agents, each in its own git worktree (or a
  given directory), with a role that sets model, prompt, tools and permissions;
- **message** them. Messages go to an agent, to the human's inbox, or from one
  agent to another, with delivery tracked;
- **observe** them. You can see status, the live event stream, transcripts and
  usage;
- **control** them. You can interrupt, stop, resume after a crash or daemon
  restart, and remove them with their worktree;
- **record who did what.** Every action carries the principal that caused it:
  human, a bridle-hosted agent, an external agent (the orchestrator) or bridle
  itself.

### 1.2 Where this departs from design.md

| design.md | This doc | Why |
|---|---|---|
| One global `~/.bridle/bridle.db` across projects; `bridle` is a CLI that agents call | **One daemon per workspace** (one project clone), state in `<workspace>/.bridle/`. The CLI is a thin client of the daemon's API | Agents are child processes with open pipes, so something long-lived must own them. A per-workspace daemon can move to another machine on its own (decision §1.2.1), and cross-project views become a client-side fan-out over several daemons. The global registry (§10.3) can still exist as a list of daemons |
| Build order starts at P0 (tasks, state branch) | **The agent host comes first**, with no task store | The designer asked for it. It's also the riskiest part, now de-risked by spike 01. Tasks (P0), messages as task threads (P1) and the rest layer on the same daemon and API later |
| The driver is the human's interactive session | **Three kinds of agent**: the *external orchestrator* (the human's own agent, optional, anywhere), a *manager* agent that bridle hosts locally for synchronisation, and *workers* | See §2. The external orchestrator is the human's interface, not a part of bridle. Bridle must run without it |
| Mid-turn delivery through a `PostToolUse` hook (`bridle inbox --inject`) | **Mid-turn delivery through stdin.** Hooks aren't needed for messages in v1 | Spike 01: a stdin message sent mid-turn is folded into the running turn at the next tool boundary (S3), with an ack via `--replay-user-messages`. Hooks cost ~0.9 s each (S11) |

---

## 2. The operating model, and corrections to it

The expected flow, as stated, with corrections:

1. **Create a workspace folder.** ✔ It's the daemon's home: it holds the clone,
   the worktrees and bridle's state.
2. **Clone the main branch of the working repo** into it. ✔
3. **Run bridle pointing at the clone; it reads context from the repo and
   creates a worktree for itself if needed.** *Mostly.* Bridle reads
   `<repo>/.bridle/config.toml` if present (roles, defaults). The file is
   optional in v1. **Bridle does not need a worktree for itself**: it never
   edits code, and its v1 state is a SQLite file under `<workspace>/.bridle/`,
   not in git. (The `bridle` state branch of design.md §10.1 arrives with
   tasks in P0, and its worktree then goes under `<workspace>/.bridle/state/`.)
   **Agents** get worktrees, under `<workspace>/wt/<agent>`, on branches
   `bridle/<agent>`. The clone's own checkout is left to the human and to the
   manager.
4. **The daemon waits for commands, and can be told to "start working" and
   pick up tasks from the repo itself.** *The first half, yes. The second half
   belongs to the manager, not to bridle.* Bridle is mechanism: it spawns,
   delivers, supervises, records and (later) integrates. **Deciding what to work
   on is judgement, so it's an agent's job** (design.md §5.1). "Start working"
   means starting the manager agent: `bridle spawn manager`, or `autostart =
   true` on the role so it starts with the daemon. The manager reads the repo's
   task source and spawns workers through the same CLI. Until the task store
   exists (P0), "the repo's tasks" is whatever the manager's role prompt points
   it at.
5. **Launch the orchestrator agent, which talks to bridle through the CLI.** ✔
   It uses a token that identifies it as `external:orchestrator` (§5). It
   never needs the repo: when bridle is remote, the orchestrator's only view
   of the code is through bridle and the agents it hosts.
6. **The bridle CLI can be used at any time, from a TUI, GUI or agent.** ✔
   Because the CLI is an API client (§6), the TUI and GUI use that API directly
   rather than shelling out to the CLI, and get a live event stream.
7. **Human actions are distinguished from agent actions.** ✔ Every API call is
   authenticated by a token that names a principal, and every event records it.
   **Correction:** on one machine, as one Unix user, this is *attribution, not
   security*. An agent could read the human's token file if it went looking.
   §5.3 makes the honest path the default and keeps the human token out of
   agents' environments and worktrees.

**The IPC question is settled now: HTTP from day one.** The CLI needs to talk
to a daemon regardless, so v1 makes that channel the local API you would
otherwise add later: JSON over HTTP for commands, Server-Sent Events for the
live stream. It listens on `127.0.0.1` by default. Listening on another
interface is one flag, which is what lets the workforce run remotely while the
orchestrator and TUI stay on the laptop (§6.4).

### 2.1 Several projects at once

Several projects run at the same time on different repos, **entirely
separately**: each has its own workspace, daemon, database, roles, and later its
own tasks, workflow layers and design tree. Nothing is shared between daemons,
so one project's rules can't leak into another's.

To make that workable from one terminal, TUI or orchestrator:

- **Registry.** On start, each daemon writes
  `~/.bridle/daemons/<project>.json` (`{project, workspace, repo, url, pid,
  started_at}`), and removes it on clean shutdown. Stale entries (dead pid) are
  pruned by any reader. `project` defaults to the clone's directory name and is
  set with `--project`. Two daemons can't claim the same name.
- **Selection.** `bridle --project <name> …` (or `$BRIDLE_PROJECT`) targets a
  daemon from the registry. `bridle daemons` lists them with agent counts. Inside
  a workspace, the cwd walk (§3.1) picks the right one without flags.
- **Ports.** Each daemon listens on `127.0.0.1:0` by default, so they never
  collide. The actual URL is in `daemon.json` and the registry.
- **Tokens are per daemon.** The human has one token per workspace, and an
  external orchestrator gets one per project it's allowed to drive.
- **A TUI or orchestrator that spans projects** fans out over the registry.
  There's no cross-project daemon in v1.

**The one shared resource is the subscription budget.** All projects draw on
the same account windows (design.md §11). v1 only records usage per daemon. A
cross-project budget governor is §13 open question 1.

**Talking to an orchestrator through bridle.** If the human can't run their
own agent, they can host one in bridle (`bridle spawn orchestrator`, a role like
any other) and talk to it with `bridle send`, from a TUI or later over MCP. The
mechanism is the same as for any agent.

---

## 3. Architecture

```
  laptop (or the same machine)                     workspace host
 ┌───────────────────────────┐          ┌────────────────────────────────────────────────┐
 │ human: CLI / TUI / GUI     │  HTTP    │ bridle daemon   (bridle serve)                  │
 │   token: human             │ ───────► │  api    axum: /v1/*  JSON + SSE events         │
 │ orchestrator agent         │  + SSE   │  store  SQLite: principals, agents, messages,  │
 │   (Claude Code, remote-    │ ◄─────── │         events, usage                           │
 │    control) token:         │          │  supervisor ── one task per agent ──┐           │
 │    external:orchestrator   │          │  containment (pgid + env tag + scan) │          │
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
- **Agents talk back through the same API**, by running `bridle` with
  their own token, which bridle injects into their environment. Later they can
  use a bridle MCP server exposing the same operations (§12).

### 3.1 Workspace layout

```
<workspace>/
  <repo>/                   the clone (main checkout): human's + manager's cwd
  wt/<agent>/               one worktree per agent that asked for one
  .bridle/
    daemon.json             {pid, url, started_at, version}  — how clients find the daemon
    bridle.db               SQLite (WAL)
    tokens/human            the human's token (0600)
    daemon.log              when detached
    agents/<id>/
      transcript.jsonl      every stdin/stdout/stderr line, as in spike 01
      system-prompt.md      the rendered --append-system-prompt-file
```

**Discovery.** A client finds the daemon from the first of these that is
set:

1. `--url`;
2. `$BRIDLE_URL`;
3. `--project` / `$BRIDLE_PROJECT` via the registry (§2.1);
4. `.bridle/daemon.json`, found by walking up from the cwd.

Worktrees and the clone sit inside the workspace, so the walk works from
anywhere in it.

---

## 4. The agent host

This section encodes what spike 01 established. Fixture references are to
`spikes/stream-json/fixtures/`.

### 4.1 Spawning

`claude` is started with, always:

```
-p --input-format stream-json --output-format stream-json --verbose
--replay-user-messages                       # delivery acks (§4.3)
--session-id <uuid>        | --resume <uuid> # never both (S7: rejected unless --fork-session)
--model <role.model>  --effort <role.effort>
--append-system-prompt-file .bridle/agents/<id>/system-prompt.md
--exclude-dynamic-system-prompt-sections     # S10: shared prompt cache across worktrees
--strict-mcp-config
--permission-mode <role.permission_mode>  --permission-prompts none
--allowedTools <role.allowed_tools>  [--disallowedTools …]
--name <agent name>
```

- **`--permission-prompts none`**: anything that would prompt is denied
  automatically, and the denial shows up in `result.permission_denials`, which
  bridle surfaces as an event. v1 therefore never has to answer a
  `control_request` from claude. Routing permission prompts to questions
  (research 01 §7.4) is a later step.
- **Environment**: bridle removes every inherited `CLAUDE*` variable and any
  `BRIDLE_TOKEN` (spike surprise 12), then sets `BRIDLE_URL`, `BRIDLE_TOKEN`
  (the agent's own), `BRIDLE_AGENT_ID`, `BRIDLE_AGENT_NAME` and
  `BRIDLE_WORKSPACE`. `BRIDLE_AGENT_ID` doubles as the containment tag (§4.6).
- **Process group**: the agent is the leader of its own process group
  (`process_group(0)`).
- **cwd**: `wt/<name>` (a new worktree on branch `bridle/<name>` from the
  role's base ref), the clone itself, or an explicit path.
- **First message**: the spawn request's prompt, if given, is written to stdin
  as the first user message and starts the first turn. Otherwise the agent
  starts idle.

### 4.2 States

```
 starting ─► idle ◄──────► working ──► (interrupt) ──► idle
     │         │   send/turn end │
     │         └──── stop ───────┴──► stopping ──► stopped      (we closed it)
     └──────────────────────────────────────────► exited(code) (it ended on its own)
                                                   crashed     (EOF with no close from us, or startup error)
                                                   lost        (daemon restarted while it ran)
```

- **idle → working** on `system/init`, which is re-emitted before *every* turn
  (S2 line 13). **working → idle** on `result`. The result's `subtype`,
  `is_error` and `terminal_reason` are stored on the turn.
- **`stopped`, `exited`, `crashed` and `lost` are resumable.** `bridle resume
  <agent>` starts a new process with `--resume <session_id>`. The conversation
  and session id are kept (S7), and so are the cumulative cost counters.
- **Stall detection**: an agent that is `working` but has emitted nothing for
  `stall_after` (default 10 min) gets an `agent.stalled` event, once per turn.
- **Exit codes don't mean crash.** After a stdin close the code is 0 or 1 by
  whether the *last turn* succeeded (S5). A crash is stdout EOF that bridle
  didn't cause, and stderr is kept for it.

### 4.3 Messages and delivery

A message has a sender principal, a recipient (an agent, or `human`), a kind
(`note` or `question`, with `answer` referencing another message), a body and
a delivery record.

Delivery to an agent is always a stdin user message:

```
[bridle message m-0042 from human (Jo)]
<body>
Reply with: bridle send --reply-to m-0042 "<text>"      ← only for questions / when a reply is wanted
```

`--when` chooses the timing:

| `--when` | Agent idle | Agent working |
|---|---|---|
| `now` *(default)* | written at once; starts a turn | written at once; **folded into the current turn** at the next tool boundary (S3). If the turn is interrupted first, the message runs as the next turn (S4c) |
| `idle` | written at once | held by bridle; written when the turn's `result` arrives, so it starts a turn of its own |

**Acks.** With `--replay-user-messages`, claude echoes each stdin user message
at the moment the model is about to see it (S3 line 14). Bridle keeps a FIFO of
written-but-unacked messages per agent, and matches each echo to the oldest
pending message with identical text. The match sets `delivered_at` and emits
`message.delivered`. A message whose agent exits before the ack goes back to
`pending` and is re-delivered on resume.

**Caveat, untested (spike follow-up):** a message sent while the agent is
generating without tool calls. The expected behaviour is that it's taken at the
turn's end, but that isn't verified.

**Messages to `human`** land in the human inbox (`bridle inbox`). They are
events, so a TUI shows them live. **Agent-to-agent** messages work the same way
as human-to-agent ones, with the sending agent as principal.

### 4.4 Interrupt

`bridle interrupt <agent>` sends `{"subtype":"interrupt"}` and waits for the
correlated `control_response` receipt (S4, ~5 ms). The turn then ends with
`result/error_during_execution`, `terminal_reason: "aborted_tools"`. The
running tool's processes are killed by claude, and the agent returns to
`idle`.

**Bridle never sends `cancel_queued: true`**, because it silently drops
pending messages (S4b). Held (`--when idle`) messages are bridle's own queue,
and `bridle interrupt --drop-held` discards those explicitly, with an event.

### 4.5 Stopping

`bridle stop <agent>` runs the escalation, each step only if the previous one
didn't end the process:

1. **Close stdin.** Claude finishes the current turn and exits in about 0.5 s
   when idle (S5). The wait is bounded by `stop_grace` (default 30 s; `--now`
   skips to step 2).
2. **SIGTERM the process group.** Claude kills its Bash tool trees and exits
   143 (S6).
3. **After 3 s, SIGKILL the process group.**
4. **Sweep** (§4.6). This is always run, because tool processes live in
   *their own* process groups (S4, S6) and SIGKILL orphans them.

`bridle rm <agent>` stops the agent if it's running, then removes its
worktree:

- It refuses if the worktree has uncommitted changes, unless `--force`.
- It keeps the branch unless `--delete-branch`.
- It checks nothing still has files open under the worktree first (`lsof +D`,
  best-effort on macOS).

### 4.6 Containment

Behind a `Containment` trait, as research 01 §5.2 proposes. The macOS/v1
implementation:

- **Track**: every 2 s, and just before any stop, snapshot the process table
  (`ps -axo pid,ppid,pgid,lstart,command`). Add every descendant of the agent's
  pid to the agent's *seen* set, keyed by pid + start time so that a reused pid
  is never killed. This catches tool process groups before they re-parent.
- **Sweep**: SIGTERM every seen process still alive with a matching start time,
  wait 2 s, SIGKILL the remainder, and emit `agent.orphans_killed` with the
  count.
- **Not covered in v1**: double-forked daemons that detach between two scans.
  Env-tag scanning (`KERN_PROCARGS2`) is spike 5 in research 01. Linux gets a
  cgroup-v2 implementation when bridle moves to a VPS.

### 4.7 Daemon restart and recovery

Agents are the daemon's children through pipes. **If the daemon dies, each
agent's stdin reaches EOF and the agent exits after its current turn.** Agents
don't survive a restart and aren't designed to.

On startup the daemon reconciles:

- For every agent recorded as running, it kills any surviving process whose
  pid and start time match, sweeps the agent's seen set, and marks it `lost`.
- Pending messages stay pending.
- Agents whose role has `autostart = true`, or that were `lost` with
  `resume_on_restart` (the default for the manager), are resumed with
  `--resume`.

### 4.8 What bridle records per agent

Every raw line in and out goes to `.bridle/agents/<id>/transcript.jsonl`, in
spike 01's format. From the parsed stream, bridle keeps:

- **agent row**: state, pid and start time, session id, turn count, last event
  time, current turn's start time, cumulative cost, worktree and branch;
- **per turn**: start and end times, result subtype, `terminal_reason`,
  `usage`, and `permission_denials`;
- **account**: the latest `rate_limit_event` info (it arrives once per process,
  S8);
- **events** (§7): `agent.*`, `turn.*`, `message.*`, `tool.use` (name and
  short input, no output).

Assistant content arrives **one content block per event** (surprise 6).
Bridle emits `agent.text` per text block and doesn't try to reassemble
messages.

---

## 5. Principals and provenance

### 5.1 Principals

| Kind | Example id | Gets its token from |
|---|---|---|
| `human` | `human` | created on first start, `.bridle/tokens/human` (0600) |
| `agent` | `agent:w1` | minted at spawn, injected as `BRIDLE_TOKEN`, revoked at `rm` |
| `external` | `external:orchestrator` | `bridle token create orchestrator`, printed once and stored hashed |
| `system` | `system` | bridle itself; no token |

Every mutating request and every event records its `actor`. Commands that
are only the human's (for example `token create`, and later `accept`) refuse
other principals.

### 5.2 How the CLI picks a token

1. `--token`, then `$BRIDLE_TOKEN`.
2. Otherwise, **only if `$CLAUDECODE` is unset**, the human token file.
3. Otherwise, fail with "set `BRIDLE_TOKEN`".

Rule 2 means a Claude Code session (the human's orchestrator, or any agent)
never silently acts as the human. It has to be given an identity.

### 5.3 What this is and isn't

On one machine as one user, a token file is readable by any process of that
user, so provenance is **attribution that honest agents can't get wrong by
accident, not a security boundary**. Two things keep it honest:

- The human token is never passed into an agent's environment.
- Agent worktrees don't contain the token.

Once bridle listens on a non-loopback interface, the tokens become real
authentication, and remote deployments should also put bridle behind SSH port
forwarding, a VPN or TLS termination (§6.4).

---

## 6. The API

### 6.1 Shape

JSON over HTTP, versioned under `/v1`. Resource ids are either a stable id
(`a-7f3k2`) or the unique name (`w1`), wherever an agent is named.

| Method + path | Does |
|---|---|
| `GET /v1/health` | liveness + version (no auth) |
| `GET /v1/status` | daemon, workspace, repo, principal, agent counts by state, latest rate limits |
| `GET /v1/agents` · `POST /v1/agents` | list · spawn (`{name?, role, prompt?, workdir: worktree\|repo\|path, base?, model?}`) |
| `GET /v1/agents/{id}` | detail incl. current turn and totals |
| `POST /v1/agents/{id}/messages` | send (`{body, kind, when, reply_to?}`) |
| `POST /v1/agents/{id}/interrupt` · `/stop` · `/resume` | control |
| `DELETE /v1/agents/{id}` | `rm` (`?force&delete_branch`) |
| `GET /v1/agents/{id}/transcript?since=&limit=` | raw transcript lines |
| `GET /v1/messages?to=&from=&unread=` · `POST /v1/messages` | inbox queries · send to any recipient incl. `human` |
| `POST /v1/messages/{id}/read` | mark read |
| `GET /v1/events?since=&agent=&kind=` | the event log, JSON page |
| `GET /v1/events/stream?since=` | the same as **SSE**; resumable with `Last-Event-ID` |
| `GET /v1/usage` | per-agent and total turns, tokens, cost, latest rate limits |
| `POST /v1/tokens` | mint an `external` token (human only) |
| `POST /v1/shutdown` | graceful stop of all agents, then exit (human only) |

Errors use one shape, `{"error": {"code": "...", "message": "..."}}`, with
proper status codes.

### 6.2 Why SSE, not WebSockets, for v1

Every client-to-daemon action is already a plain request, so the push channel
only has to go one way. SSE over plain HTTP:

- works with `curl -N`;
- resumes from an event sequence number with `Last-Event-ID`;
- needs no extra protocol in the CLI.

A WebSocket endpoint can be added beside it if a GUI needs bidirectional
streaming, for example a live terminal attached to an agent.

### 6.3 The CLI

```
bridle serve   [--repo PATH] [--workspace DIR] [--project NAME] [--listen ADDR] [--detach]
bridle stop-daemon
bridle daemons                              # every running project on this machine
bridle status                               # daemon + agents summary
bridle spawn   <role> [--name N] [--prompt TEXT | --prompt-file F]
               [--worktree [--base REF] | --in-repo | --cwd PATH] [--model M]
bridle agents  [--all]                      # table; --json
bridle show    <agent>
bridle send    <agent|human> TEXT [--question] [--when now|idle] [--reply-to ID]
bridle inbox   [--all] [--mark-read]        # messages to me (human, or the calling agent)
bridle interrupt <agent> [--drop-held]
bridle stop    <agent> [--now]      bridle resume <agent>
bridle rm      <agent> [--force] [--delete-branch]
bridle logs    <agent> [--follow] [--raw]   # readable rendering of the transcript
bridle events  [--follow] [--since N] [--agent A]
bridle usage
bridle token create <name>
```

- Every command takes `--json`, which agents always use. Humans get compact
  tables.
- Exit codes: 0 ok, 1 error, 2 usage error, 3 daemon unreachable.
- `bridle serve --detach` re-executes itself in a new session with output
  going to `.bridle/daemon.log`. It waits for `/v1/health` to answer, prints
  the URL and exits. `serve` in the foreground logs to stderr.

### 6.4 Running the workforce remotely

Run `bridle serve --listen 0.0.0.0:7433` on the remote host. On the laptop:

```
export BRIDLE_URL=http://host:7433        # better: ssh -L 7433:localhost:7433 host
export BRIDLE_TOKEN=<human or external token>
```

The orchestrator and TUI then have **no filesystem access to the repo**. They
read code through agents, or through later read-only endpoints (diff, file at
ref). That is the intended split: the workforce and the repo sit together, and
control is remote. Bridle v1 has no TLS. Use SSH forwarding or a private
network.

---

## 7. Events

One append-only table with a monotonically increasing `seq`, which is also the
SSE event id:

```json
{"seq":1042,"ts":"2026-09-27T14:03:11.120Z","kind":"message.delivered",
 "actor":"human","agent":"w1","data":{"message":"m-0042"}}
```

| Kind | Emitted when |
|---|---|
| `daemon.started` / `daemon.stopping` | lifecycle |
| `agent.spawned` · `agent.state` (`{from,to}`) · `agent.exited` (`{code,signal,reason}`) · `agent.stalled` · `agent.orphans_killed` | agent lifecycle |
| `turn.started` · `turn.ended` (`{subtype,is_error,terminal_reason,usage,cost_total}`) | turn boundaries |
| `agent.text` (`{text}`, truncated to 2 KB) · `tool.use` (`{name,input_summary}`) · `permission.denied` | what the agent is doing |
| `message.sent` · `message.delivered` · `message.read` | messaging |
| `rate_limit` (`{info}`) | a `rate_limit_event` from any agent |

The event log **is** the provenance record: every row has an actor. Rows are
kept for 30 days by default. Transcripts are the full-fidelity record.

---

## 8. Roles and configuration

The daemon loads built-in defaults, then applies `<repo>/.bridle/config.toml`
over them:

```toml
[daemon]
listen      = "127.0.0.1:0"        # 0 = any free port; the chosen URL goes in daemon.json
stall_after = "10m"

[roles.worker]
model            = "sonnet"
effort           = "medium"
workdir          = "worktree"      # worktree | repo
base             = "HEAD"          # ref new worktrees branch from
permission_mode  = "acceptEdits"
allowed_tools    = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]
system_prompt    = ".bridle/roles/worker.md"     # appended; bridle adds its own preamble

[roles.manager]
model             = "sonnet"
workdir           = "repo"
autostart         = false
resume_on_restart = true
allowed_tools     = ["Bash(bridle *)", "Bash(git *)", "Read", "Glob", "Grep"]
system_prompt     = ".bridle/roles/manager.md"
```

- **Built-in roles** in v1 are `worker`, `manager` and `orchestrator`.
- **Every role's system-prompt file** is prefixed with a short bridle preamble.
  It says what bridle is, the agent's identity variables, and how to use
  `bridle send`, `inbox` and `status --json`. The preamble is **identical for
  every agent of a role**, so the prompt cache holds (design.md §11.4 rule 2).
  Agent-specific facts (name, worktree path) go in the first user message, not
  the prompt file.
- **A worker can always reach bridle**: `Bash(bridle *)` is added to every
  role's allowed tools.

The layered workflow of design.md §4 later replaces the role prompts. The
`[roles]` table stays as the place models and tools are set.

---

## 9. Usage (v1 subset of design.md §11)

Bridle stores every `turn.ended`'s `usage` and cumulative cost per agent, and
the latest `rate_limit_event` per window. `bridle usage` shows:

- per-agent and total turns and tokens;
- the cache hit ratio;
- the last known five-hour and seven-day utilisation with reset times.

Two spike findings shape this:

- **Cost counters are cumulative per session and survive `--resume`.** A turn's
  cost is therefore the difference between consecutive counters, and bridle
  stores both.
- **There's an on-demand usage query (`get_usage`), but it's undocumented.** v1
  doesn't depend on it. `bridle usage --refresh` may send it through a live
  agent if one exists, and says so in the output.

The budget governor (design.md §11.3: pause at limits, resume at reset) is not
in v1. The data it needs is recorded from day one.

---

## 10. Store (v1 schema)

SQLite in WAL mode, one connection owned by a store actor. All access goes
through async methods on a `Store` handle.

```
principals(id TEXT PK, kind, name, token_hash, created_at, revoked_at)
agents(id PK, name UNIQUE, role, state, model, session_id, pid, pid_start,
       workdir_kind, cwd, worktree, branch, created_at, updated_at,
       turns, cost_usd_total, last_event_at, exit_code, exit_signal, exit_reason,
       created_by)
turns(agent_id, n, started_at, ended_at, subtype, is_error, terminal_reason,
      input_tokens, output_tokens, cache_read, cache_write, cost_total,
      PRIMARY KEY(agent_id, n))
messages(id PK, from_principal, to_kind, to_id, kind, body, reply_to,
         when_mode, state, created_at, written_at, delivered_at, read_at)
events(seq INTEGER PK AUTOINCREMENT, ts, kind, actor, agent_id, data JSON)
rate_limits(window PK, status, utilization, resets_at, observed_at)
```

Nothing here must survive a lost database in v1 (design.md decision 2):

- There are no tasks yet.
- Transcripts are files.
- The conversations themselves live in Claude Code's session store, and are
  resumable by session id.

---

## 11. Crates

```
crates/
  bridle-claude/   the stream-json client: AgentProcess, Event model, transcript writer.
                   No knowledge of the daemon. Spike 01's code, hardened, with its
                   fixtures as parser tests.
  bridle-api/      API types (requests, responses, events) + an async HTTP client
                   with SSE. Shared by the daemon, the CLI, and future TUI/GUI/MCP.
  bridle-daemon/   store, supervisor, containment, worktrees, config, axum server.
  bridle/          the binary: clap CLI; `serve` runs bridle-daemon.
```

The TUI will be a fifth crate that depends only on `bridle-api`, and so will a
future MCP server.

---

## 12. After v1

In rough order. Each item sits on this daemon and API:

1. **Tasks and the state branch** (design.md P0): messages addressed to tasks,
   `ready`, and claims whose leases are renewed by agent activity. Bridle
   already sees every agent's activity, so hooks aren't needed for that.
2. **`bridle take` / `give`** (research 01 §5.3): interrupt, close, then hand
   the human `claude --resume <session>` in the worktree, and resume headless
   afterwards.
3. **Permission prompts as questions**: `--permission-prompts host`, with
   claude's `can_use_tool` control requests answered from role rules or turned
   into a `question` to the manager or human.
4. **The TUI**, on `bridle-api`: an agents list, an event tail, per-agent logs
   and inbox/reply.
5. **MCP server** at `/mcp` on the same listener, with tools mirroring the CLI.
   Agents could then use MCP tools instead of Bash, and the human could reach
   bridle from claude.ai, desktop or mobile. That last part needs bridle
   reachable over **public HTTPS** with auth, so it needs a tunnel or a proxy
   in front. It's not in v1, and `claude remote-control` on the orchestrator
   covers the need for now.
6. **Budget governor** and the rest of design.md §11.
7. **Integrator**: merge-tree probes and merging workers' branches, per
   design.md P5.

---

## 13. Open questions

1. **How do separate project daemons share one budget?** Projects are
   deliberately isolated, one daemon each (§2.1). They share one account,
   though, and don't know about each other's usage. Options for the budget
   governor (§12.6):
   - each daemon reads a shared `~/.bridle/budget/` ledger and lease file;
   - a tiny machine-level governor that daemons ask before dispatching;
   - static per-project `max_workers` shares.

   The first keeps daemons independent.
2. **Does a mid-turn message during tool-less generation fold in or wait?**
   This needs a follow-up spike (§4.3).
3. **`--replay-user-messages` echo matching by text** assumes claude echoes
   the text verbatim. It did in spike 01. If a future version adds a `uuid` to
   stdin user messages, match on that instead.
4. **Does the manager run in the main checkout or its own worktree?** v1 says
   the checkout (`workdir = "repo"`), since it coordinates rather than edits.
   If it starts merging, it needs its own integration worktree.
5. **Event retention** of 30 days is a guess. Transcripts grow without bound
   until `rm`.
