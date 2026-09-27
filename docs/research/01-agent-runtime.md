# Research 01 — The agent runtime: spawning, hosting, watching, cleaning up

*2026-09-27. Two parallel research passes, one on Claude Code's own
capabilities (docs at code.claude.com, checked against the installed CLI
v2.1.283) and one on multiplexers, PTYs, process containment and prior art.
It answers the designer's question: should bridle host agents in tmux, and can
Claude Code's rendering be made static enough to supervise that way?*

> **Research, not a decision.** §7 is a recommendation. Claims marked
> **[unverified]** could not be confirmed from docs or source. **[tested]**
> marks the one thing actually run.

---

## 1. What bridle has to do

For each task:

1. create a branch and a git worktree (in the project's layout, including
   harness's paired siblings);
2. start an agent in it with explicit, custom instructions;
3. monitor it through bridle messages **and** by process, including anything
   the agent spawns;
4. when it's done, merge the branch, remove the worktree, and shut the agent
   and all its processes down.

And beyond that: deliver messages to it (mid-turn and when idle), let the human
watch and step in, and survive the agent misbehaving.

---

## 2. The four ways to host a Claude Code agent

| | **A. Interactive TUI in tmux** | **B. Interactive TUI in a PTY bridle owns** | **C. Headless `claude -p` stream-json, stdin held open** | **D. Agent SDK sidecar (TS/Python)** |
|---|---|---|---|---|
| Input | `tmux send-keys` / `paste-buffer`, fragile | writes to the PTY, still keystrokes | **typed JSON lines on stdin** | SDK calls |
| Output / state | screen scraping + hooks | virtual screen (vt100) + hooks | **typed JSON events on stdout** + hooks | typed SDK messages |
| Wake an idle agent | type into it | type into it | **write a user line to stdin** [tested] | `streamInput()` |
| Interrupt | send Esc/Ctrl-C | same | **`control_request: interrupt`** [tested] | `interrupt()` |
| Human watches | `tmux attach`, free | bridle must build attach | bridle renders the event stream | same as C |
| Human steps in | type in the pane | type in the pane | stop, then `claude --resume <id>` interactively (§5.3) | same as C |
| PID ownership | `#{pane_pid}` | direct | **direct** | indirect unless custom spawner |
| Rust effort | low | **high** (query replies, resize, repaint, attach) | medium (protocol client) | low in Rust, plus a sidecar runtime |
| Main risk | Enter swallowed after paste; prompt-string detection breaks on UI changes; flicker before tmux 3.7 | reimplementing a terminal | control protocol is **semi-internal**, documented only through SDK types | extra runtime and IPC hop |

Also looked at and ruled out as the worker host:

- **`claude --bg` / agent view** (`claude agents`, `attach`, `stop`). It has a
  good state API, but Claude Code's own daemon supervises these sessions:
  restarts them, reaps idle ones, and hands child processes to the next
  process. That competes with bridle owning the PIDs.
- **Agent teams** (experimental). Teammates only spawn in interactive sessions,
  not from `-p` or the SDK. One team per session, no nesting.
- **Remote Control / channels.** They need a claude.ai login and are built for
  a different job (reaching sessions from other devices). Channels are
  interesting as an injection path, but they're a research preview.

---

## 3. The designer's question: can Claude Code render statically for tmux?

**Yes, largely.** In order of effect:

| Setting | Effect |
|---|---|
| `CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1` | forces the **classic** renderer (main screen, native scrollback). Overrides `tui` and `CLAUDE_CODE_NO_FLICKER` |
| `CLAUDE_AX_SCREEN_READER=1` / `--ax-screen-reader` | flat text, no borders or animations. The most static mode available |
| settings `prefersReducedMotion: true`, `spinnerTipsEnabled: false`, `syntaxHighlightingDisabled: true` | no spinner, shimmer or flash |
| `CLAUDE_CODE_DISABLE_MOUSE=1` | stops mouse capture fighting tmux |
| `CLAUDE_CODE_DISABLE_TERMINAL_TITLE=1`, `FORCE_HYPERLINK=0` | fewer escape sequences |
| `NO_COLOR` | only if set in the environment before launch |
| **tmux ≥ 3.7** (ideally 3.7d **[unverified that 3.7d is tagged]**) | tmux ≤ 3.6 doesn't implement synchronized output (DEC 2026), which is the main cause of flicker. 3.7b/c had redraw regressions |

**Can bridle read what the terminal reports?** Technically yes: `capture-pane
-p -J` for snapshots, control mode (`tmux -C`) for a live `%output` stream.
Every tool that relies on this reports the same failures:

- "Screen changed" gets confused by spinners and timers;
- prompt-string detection (`"No, and tell Claude what to do differently"`,
  `"❯ "`) breaks when Claude Code's UI wording changes, and differs between
  renderers;
- snapshots can catch a half-drawn frame;
- **text followed too quickly by Enter gets swallowed** by paste detection.
  Gas City now bracketed-pastes, waits for the text to appear, and then sends
  Enter with a retry.

Screen scraping works as a fallback ("is it stuck on a permission prompt?"),
not as the source of truth. **The same information is available in structured
form**, which is the key finding below.

---

## 4. The structured channels Claude Code already provides

### 4.1 Headless stream-json is a real bidirectional session  [tested]

```
claude -p --input-format stream-json --output-format stream-json --verbose \
       --session-id <uuid> --name <task> …
```

- **stdin** takes one JSON object per line and stays open across turns:
  - `{"type":"user","message":{"role":"user","content":"…"},"parent_tool_use_id":null}`
    starts a turn, or is queued if a turn is running;
  - `{"type":"control_request","request_id":"r1","request":{"subtype":"interrupt"}}`
    ends the current turn cleanly and returns a receipt listing still-queued
    messages.
- **stdout** emits typed events: `system/init` (with a `capabilities` list for
  feature detection), assistant text, `tool_use`/`tool_result`, `api_retry`,
  `result` (cost, `session_id`), and optionally hook events and subagent text.
- The process exits 0 when stdin closes. Tested on v2.1.283: two turns, 12 s
  apart, with an interrupt between them, on one `session_id`.

**Caveat:** the control-message format beyond user messages and interrupt is
documented only through the SDK's TypeScript types. Pin the Claude Code
version, feature-detect with `capabilities`, and keep the protocol client
small.

### 4.2 Launch flags bridle would use

| Flag | Purpose |
|---|---|
| `--session-id <uuid>` | bridle picks the id up front, so it knows it before the first event |
| `--name <task-id>` | the display name other sessions address it by |
| `--append-system-prompt-file <f>` | the role and task instructions bridle renders (not `--system-prompt`, which replaces Claude Code's own) |
| `--settings <json>` | bridle's hooks and per-agent settings **injected at launch**, instead of written into the repo's `.claude/settings.json` |
| `--setting-sources user,project` | control which settings files load |
| `--mcp-config <json>` + `--strict-mcp-config` | a bridle MCP server exposing `send`/`inbox`/`ask`/`claim`… as tools |
| `--permission-mode acceptEdits\|auto\|dontAsk` | a headless worker can't show a dialog |
| `--permission-prompt-tool <mcp tool>` | **route permission requests to bridle**, where they can become questions to the driver or human |
| `--permission-prompts none` | alternative: anything that would prompt is denied |
| `--allowedTools` / `--disallowedTools` | per role, e.g. workers don't get `git push` |
| `--model`, `--effort`, `--max-budget-usd` | per role, from `workflow.toml` |
| `--resume <session-id>` | continue a session after a restart, or for human takeover |

**Don't use `claude -w`.** Its worktree location can't be configured (it's
always `<repo>/.claude/worktrees/<name>`), `-p` leaves a git lock behind and
never cleans up, and it can't do the paired layout. Bridle runs `git worktree
add` itself and launches `claude` with that worktree as its cwd. Claude Code
then treats it as a linked worktree and stops the agent editing the main
checkout.

### 4.3 Hooks: identity, status and injection

- Hooks **fire inside Agent-tool subagents** and carry `agent_id` and
  `agent_type`. This resolves design §14 open question 2 in favour of both
  worker models being possible.
- Every hook gets `session_id`, `cwd` and `transcript_path`. The environment
  includes `CLAUDE_CODE_SESSION_ID` and `CLAUDE_PID`, and bridle adds
  `BRIDLE_AGENT`.
- Status events: `UserPromptSubmit`/`PreToolUse` mean working; `Stop` (with
  `last_assistant_message` and `background_tasks`) means the turn is done;
  `Notification` with `notification_type: permission_prompt` means blocked (but
  `idle_prompt` does **not** mean it needs input); `SessionEnd` has a `reason`.
- Injection: `hookSpecificOutput.additionalContext` (up to 10k characters) on
  `PostToolUse`, `UserPromptSubmit`, `SessionStart`, `Stop` and others. This
  is the mid-turn message path from design §6.4.
- **`Stop` → `{"decision":"block","reason":…}`** keeps the agent working. This
  is the stop-check from design §6.4: "you still hold a claim with no handoff".
- **`asyncRewake: true`** hooks that exit 2 wake an idle interactive session.
- `http` hook type: POSTs the hook JSON straight to an endpoint, so bridle
  doesn't need a process per hook call if it runs a local listener.

### 4.4 Built-in cross-session messaging

Claude Code has `ListAgents` + `SendMessage` between local sessions (v2.1.224+),
over a per-session Unix socket. It delivers between tool calls, and **starts a
new turn if the target is idle**. It works for `-p` sessions (not `--bare`)
when `crossSessionInbound: "accept"` is set.

It's useful, but it shouldn't be bridle's backbone:

- the message format for posting into the socket from outside is **not
  documented**, so bridle can't reliably inject;
- messages are plain text, ephemeral and capped (50 queued, dedupe, rate
  limits), and go around bridle's store, so nothing reaches the task thread or
  git;
- addressing is by session name, not by task.

**Use it as a complement**: an agent can still `SendMessage` a peer. Durable
messages go through bridle.

### 4.5 Transcripts and telemetry

- `~/.claude/projects/…/<session>.jsonl` is explicitly **internal and
  version-dependent**, and written asynchronously. Fine for a human to browse,
  but not a contract to build on.
- OpenTelemetry (`CLAUDE_CODE_ENABLE_TELEMETRY=1`, OTLP) gives cost, tokens,
  commits and tool events. It's worth wiring later for a cost dashboard.

---

## 5. Supervising the process tree

### 5.1 The macOS problem

A process group is advisory. Anything the agent starts that calls `setsid()`
or double-forks is re-parented to launchd, and `killpg` misses it. macOS has
no cgroups and no subreaper, and `kqueue` can't follow forks (`NOTE_TRACK`
returns ENOTSUP). Some things are outside any tree entirely: `brew services`,
Docker containers (they run in a VM), and a tmux server the agent started
itself.

### 5.2 The practical approach, in layers

1. **Spawn** `claude` as a new session and process-group leader. Record
   `(pid, start_time)` so a reused PID is never killed.
2. **Tag the environment**: `BRIDLE_AGENT=<id>`, a per-agent `TMUX_TMPDIR`,
   `COMPOSE_PROJECT_NAME=bridle-<id>`, and the agent's allocated `PORT`s.
   Environment survives setsid, double-fork and nohup, and on macOS a
   same-user process's environment is readable through `sysctl
   KERN_PROCARGS2` (**[unverified]** how complete `sysinfo`'s `environ()` is).
3. **Scan every 1–2 s** (`libproc`/`sysinfo`) and add descendants to a
   "seen" set, which catches processes before they re-parent. Also match on
   a cwd inside the worktree.
4. **Teardown loop**: SIGTERM the process group plus every tagged or seen PID
   whose start time matches, wait about 3 s, SIGKILL, rescan, and repeat until
   nothing matches. Claude Code's own SIGTERM handling kills running Bash trees
   and runs `SessionEnd` hooks (exit 143).
5. **Check before removing the worktree**: `lsof` for listening ports held by
   tagged PIDs, `lsof +D <worktree>` for open files, `docker compose -p
   bridle-<id> down`, then `git worktree unlock`/`remove`.
6. **Abstract it**: a `Containment` trait, implemented on macOS as pgid +
   env tag + scan. The Linux implementation for a future VPS is a cgroup v2
   scope with `cgroup.kill`, which is atomic and nothing escapes it, plus
   subreaper and pidfd.

### 5.3 Human takeover without a terminal multiplexer

Headless workers don't need tmux for the human to step in, because sessions can
be resumed:

1. `bridle take <agent>`: bridle interrupts the worker, closes its stdin and
   waits for exit (the tree stays tagged, and the task keeps its claim).
2. It runs `claude --resume <session-id>` **interactively in the worktree**, in
   the human's own terminal. It's the same conversation with full context.
3. When the human exits, `bridle give <agent>` restarts it headless with
   `--resume <session-id>`, and the worker carries on.

To watch without taking over, `bridle watch <agent>` renders the stream-json
events as a readable log: assistant text, tool calls with short arguments,
results, cost. `bridle say <agent> "…"` injects a line without taking over.

---

## 6. Prior art, briefly

| Tool | Hosting | Status from | Lesson for bridle |
|---|---|---|---|
| claude-squad | tmux | hash of `capture-pane` + prompt strings | scraping works, and needs constant upkeep as the UI changes |
| Gas Town / Gas City | tmux, keystroke injection | prompt prefix, `pane_current_command`, Stop hook | swallowed-Enter bug, fixed with bracketed paste + settle + retry |
| SwarmForge | tmux, one worktree per role | outbox files + a handoff daemon | user tmux config (`pane-base-index`) and a read-only attach broke `send-keys`. **Use a private socket and your own config** |
| uzi | tmux + worktree, port management | screen **[unverified]** | per-agent dev-server ports are a common need |
| ccmanager | node-pty, no tmux | per-agent output parsers | owning the PTY is viable |
| herdr, asd, shpool | own PTY + vt100 (Rust) | process names + screen manifests | the option B path already exists as crates |
| Conductor, Vibe Kanban | headless / SDK **[unverified]** | structured stream | the more polished tools moved off screen scraping |
| container-use, Sculptor | containers | their own | heavier isolation than worktrees; not needed on one trusted machine |

Across all of them the same complaints recur: swallowed Enter, fragile prompt
detection, `node_modules` per worktree, port collisions, orphaned worktrees
and dev servers, flicker, and **agents killing their own tmux host** (so never
let an agent see the supervisor's `TMUX` socket).

---

## 7. Recommendation

**Workers run headless (option C). The driver runs interactive. tmux is
optional and never the control channel.**

```
                     bridle (daemon: supervisor + integrator + store)
             stdin: user lines,      │   ▲  stdout: stream-json events
             interrupts              ▼   │
   ┌────────────────────────────────────────────────┐
   │ claude -p (worker)  cwd = worktree              │── hooks (http) ──► bridle
   │  tools: bridle MCP (send/ask/inbox/claim…)      │
   │  permission-prompt-tool → bridle                │
   └────────────────────────────────────────────────┘
           ▲ bridle take / give   (claude --resume, interactive, human's terminal)

   driver: interactive claude in the human's terminal (optionally tmux, classic renderer)
           hears bridle through hooks + background `bridle wait`
```

1. **Spawn**:
   1. `git worktree add` in the project's layout;
   2. run setup, e.g. `npm install --prefer-offline`;
   3. render the instructions file;
   4. start `claude -p` in stream-json mode with `--session-id`, `--name`,
      `--append-system-prompt-file`, `--settings` (bridle hooks),
      `--mcp-config` (bridle tools), permission mode and prompt tool, allowed
      tools and model from the role;
   5. write the first user line: "claim task X and begin";
   6. record the PID and environment tag.
2. **Messages**. Durable messages go into bridle's store. Delivery:
   - if the agent is idle, a stdin user line, which wakes it;
   - if it's mid-turn, `PostToolUse` `additionalContext`, which arrives between
     tool calls.

   Agents send with bridle MCP tools, or `bridle send` via Bash.
3. **Monitoring** uses three sources:
   - the stream-json events (what the agent is doing);
   - hooks (status, permission blocks, stop);
   - the process scanner (what the agent spawned, and whether it's alive).

   A stall detector flags an agent that is "working" but has emitted nothing
   for N minutes.
4. **Permissions become questions.** `--permission-prompt-tool` routes to
   bridle. Bridle auto-answers from the role's allow rules, and anything else
   becomes a `question` on the task (design §6.3), so the worker isn't stuck
   behind a dialog nobody can see.
5. **Done**:
   1. `Stop` with the task in `in_review` triggers review;
   2. once review passes, the integrator merges (with a `merge-tree` probe
      first);
   3. close stdin, wait, then run the teardown loop (§5.2);
   4. check `lsof`, then `git worktree remove`;
   5. release the ports.
6. **Human in the loop**: `bridle watch`, `bridle say`, and `bridle take`/`give`
   (§5.3).
7. **The driver** stays an ordinary interactive session: the human's. It hears
   bridle through the SessionStart/PostToolUse hooks and background `bridle
   wait`. If the designer wants it inside tmux, use a private socket
   (`tmux -L bridle -f <bridle's conf>`), tmux ≥ 3.7, and
   `CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1` plus reduced motion. tmux is then
   only a place to keep the session alive, not something bridle reads from.

**Keep the host behind a trait** (`AgentHost`: spawn / send / interrupt /
events / attach / kill), with option C as the only implementation at first.
Option A (tmux) is the obvious second implementation, for agents that other CLIs
(Codex, Gemini) can only run interactively. Option D becomes worth it only if
the raw control protocol turns out too unstable to use from Rust.

---

## 8. Spikes to run before committing

Each is small and settles an unknown this report couldn't:

1. **Stream-json client in Rust** (written up as [`spikes/01-stream-json-client.md`](docs/spikes/01-stream-json-client.md)): a 200-line prototype that spawns, sends two
   messages, interrupts, receives `result`, and resumes. Also check what
   `system/init` capabilities v2.1.283 advertises.
2. **`--permission-prompt-tool` → bridle MCP**: confirm the request and
   response shape, and that a slow answer (minutes, while a human decides)
   doesn't time out the turn.
3. **Hook mid-turn injection**: `PostToolUse` `additionalContext` from an
   `http` hook reaches a `-p` worker, and latency stays under ~20 ms when there
   is nothing to deliver.
4. **Takeover round trip**: headless → interrupt → `claude --resume`
   interactive → exit → headless `--resume`. Confirm context is intact and
   nothing is lost from the stdin queue.
5. **Containment**: have an agent start `npm run dev` with `nohup … &`, and a
   double-forked daemon. Confirm the scan-and-kill loop finds and kills both,
   and see what `sysinfo`'s `environ()` returns on macOS.
6. **MCP child lifetime**: whether stdio MCP servers outlive the `claude`
   process that started them (the docs are ambiguous).
7. **Paired worktree + `npm install` timing** for harness + track-web. This is
   research 09's still-open stage C.

---

## Sources

Claude Code docs (code.claude.com/docs/en/…): `headless`, `cli-reference`,
`agent-sdk/overview`, `agent-sdk/streaming-vs-single-mode`,
`agent-sdk/typescript`, `agent-sdk/hooks`, `agent-sdk/custom-tools`, `hooks`,
`cross-session-messaging`, `agent-teams`, `channels`, `channels-reference`,
`remote-control`, `worktrees`, `sessions`, `agent-view`, `fullscreen`,
`terminal-config`, `env-vars`, `settings-reference`, `statusline`,
`monitoring-usage`, `tools-reference`.

tmux: PR 4744 (DEC 2026), PR 5612 and issues 5619/5470 (3.7 regressions), 3.7
CHANGES. Claude Code issues 78412, 37283 (flicker), 23513 (swallowed Enter),
29787 (agent kills its host), 27562/63591/36943 (`--worktree`/`--tmux` edge
cases), 12048 (`idle_prompt`).

Prior art: github.com/smtg-ai/claude-squad (`session/tmux/tmux.go`);
github.com/steveyegge/gastown `docs/agent-provider-integration.md`;
gastownhall/gascity PR 5708; github.com/unclebob/swarm-forge issues 80, 82;
github.com/devflowinc/uzi; github.com/kbwo/ccmanager; crates.io herdr, shpool;
github.com/benenen/asd; github.com/stravu/crystal; conductor.build;
github.com/BloopAI/vibe-kanban; github.com/dagger/container-use;
imbue.com/blog/sculptor-announce; zellij.dev programmatic-control docs.

Process containment: sysinfo issue 1415; daintreehq/daintree issue 12203;
wazuh PR 39580 (`KERN_PROCARGS2`); iximiuz.com on Linux process termination;
`PR_SET_CHILD_SUBREAPER(2)`.
