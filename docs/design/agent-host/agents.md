# Agents: spawning, states, stopping, containment

Everything here encodes what [spike 01](docs/spikes/01-stream-json-findings.md)
established; S-numbers are its scenarios. Each agent is one headless
`claude -p` stream-json process, owned by the daemon through its pipes.

## Spawning

`claude` is started with, always:

```
-p --input-format stream-json --output-format stream-json --verbose
--replay-user-messages                       # delivery acks (messages)
--session-id <uuid>        | --resume <uuid> # never both (S7: rejected unless --fork-session)
--model <role.model>  [--effort <role.effort>]
--append-system-prompt-file .bridle/agents/<id>/system-prompt.md
--exclude-dynamic-system-prompt-sections     # S10: shared prompt cache across worktrees
--strict-mcp-config
--permission-mode <role.permission_mode>  --permission-prompts none
--allowedTools <tool>…  [--disallowedTools <tool>…]
--name <agent name>
--settings '{"autoMemoryEnabled":false,"autoDreamEnabled":false}'
[--max-budget-usd <role.max_budget_usd>]
```

- **`--permission-prompts none`**: anything that would prompt is denied
  automatically, and the denial shows up in `result.permission_denials`, which
  bridle surfaces as an event. Bridle therefore never has to serve an answerer
  for claude's prompts. Routing permission prompts to questions is later work
  ([[docs/proposal/build-order|build order]]); it needs an MCP tool
  (`--permission-prompt-tool`), not a `control_request` bridle answers on this
  same channel — see
  [[docs/design/agent-host/messages#Permission prompts as questions|messages.md]].
- **Environment**: bridle removes every inherited `CLAUDE*` variable and any
  `BRIDLE_TOKEN` (spike surprise 12), then sets `BRIDLE_URL`, `BRIDLE_TOKEN`
  (the agent's own), `BRIDLE_AGENT_ID`, `BRIDLE_AGENT_NAME`,
  `BRIDLE_WORKSPACE` and `BRIDLE_PROJECT`. Other `BRIDLE_*` variables are
  inherited.
- **`PATH` starts with the daemon's own binary directory**, so an agent's
  `bridle` is always the version supervising it, installed or not.
- **No memory**: every agent gets
  `--settings '{"autoMemoryEnabled":false,"autoDreamEnabled":false}'`, so
  Claude Code's auto memory is off whatever the project's or user's settings
  say ([[docs/proposal/decisions|decision 8]]).
- **Process group**: the agent is the leader of its own process group.
- **cwd**: `wt/<name>` (a new worktree on branch `bridle/<name>` from the
  role's base ref), the clone itself, or an explicit path.
- **Names** are `[a-z0-9][a-z0-9-]{0,39}`, unique (409 on a clash), and
  default to `<role>-<n>`.
- **Identity**: an agent's name, role, cwd and branch are always in its
  system prompt (`render_system_prompt`), as a short sentence after the
  role's shared, cacheable preamble. This holds regardless of whether the
  spawn had a first message.
- **First message**: the spawn request's prompt, or else the role's
  `start_prompt`, is sent as a normal message from the spawner and starts
  the first turn. With neither, the agent starts idle.
- **A failed spawn is rolled back**: worktree, branch, token and agent row, and
  the API returns 500.
- **Spawn waits for readiness, if a first message was sent.** `system/init` is
  a turn-start marker, re-emitted before every turn, not a one-time startup
  handshake (S2), so there's nothing to wait for until a turn actually
  starts. When the spawn had a prompt or `start_prompt`, `spawn` waits up to
  `SPAWN_READY_TIMEOUT` (8s), after sending it, for either that turn's
  `system/init` or the process's exit, whichever comes first, so a bad model
  name or expired auth usually shows up as a `crashed` agent in the spawn
  response itself, not only later via polling. An idle spawn with no first
  message returns as soon as the process is started, since no turn has
  begun to wait on. Either way this is a wait, not a guarantee: a slow init
  that outlasts the timeout is still returned as-is, with no error.
  `POST /v1/agents` always returns 2xx once the process has been started;
  the agent's `state` is the readiness signal, not the status code.
- **Spend cap**: below.

## Spend cap

A role's `max_budget_usd` is passed as `--max-budget-usd`. Claude applies it
per process and checks it after each model call, so a turn can overshoot it
([spike 02](docs/spikes/02-budget-cap-findings.md)). When a turn ends with
`error_max_budget_usd`, bridle emits `agent.budget_exhausted` and stops the
agent (exit reason `budget_exhausted`): every later turn would fail without
calling the model. Its messages wait, pending. `bridle resume` starts a new
process, and so grants a fresh allowance. Account-wide limits belong to the
[[docs/design/usage-and-budget#The budget governor|budget governor]].

## Claude Code upgrades

Claude Code updates itself, and bridle relies on behaviour it doesn't
document: mid-turn folding, verbatim replay echoes, the interrupt receipt,
exit codes, cumulative cost, the budget result. Bridle doesn't pin the
version. Instead:

- **The daemon notices a new version.** It records `claude_code_version` from
  each process's first `system/init`, shows it in `bridle status`, and when it
  differs from the last one seen, emits `claude.version`
  (`{version, previous}`) and logs a warning.
- **The contract suite decides.** `just test-contract` runs three live tests
  against real `claude` (Haiku, about $0.10;
  `crates/bridle-claude/tests/contract_test.rs`). If they pass, the version is
  accepted and recorded in `crates/bridle-claude/tests/contract-verified.txt`.
  If not, bridle is fixed forward, not rolled back.

## States

```
 starting ─► idle ◄──────► working ──► (interrupt) ──► idle
     │         │   send/turn end │
     │         └──── stop ───────┴──► stopping ──► stopped      (we closed it)
     └──────────────────────────────────────────► exited(code) (it ended on its own)
                                                   crashed     (EOF with no close from us, or startup error)
                                                   lost        (daemon restarted while it ran)
```

- **starting** lasts only until the process is spawned; the agent is then
  `idle` until its first turn.
- **idle → working** on `system/init`, which is re-emitted before *every* turn
  (S2). **working → idle** on `result`. The result's `subtype`, `is_error` and
  `terminal_reason` are stored on the turn.
- **`stopped`, `exited`, `crashed` and `lost` are resumable.** `bridle resume
  <agent>` starts a new process with `--resume <session_id>`. The conversation
  and session id are kept (S7), and so are the cumulative cost counters. The
  model is the one it was spawned with; the role's prompt and flags are re-read
  from the current config. Every `pending` message is then written at once,
  including ones held for `--when idle` when the agent exited.
- **Stall detection**: an agent that is `working` but has emitted nothing for
  `stall_after` (default 10 min, checked every 30 s) gets an `agent.stalled`
  event, once per silent stretch.
- **Exit codes don't mean crash.** After a stdin close the code is 0 or 1 by
  whether the *last turn* succeeded (S5). So:
  - if bridle asked it to stop, it is `stopped`, with reason `stdin_closed`,
    `sigterm` or `sigkill` by the step that ended it, or `budget_exhausted`;
  - otherwise exit code 0 or 1, no signal, after some output, is `exited`;
  - anything else is `crashed`, with the tail of stderr as the reason.

## Stopping

`bridle stop <agent>` snapshots the process tree first, so a child spawned in a
short turn is still seen, then runs the escalation, each step only if the
previous one didn't end the process:

1. **Close stdin.** Claude finishes the current turn and exits in about 0.5 s
   when idle (S5). The wait is bounded by `stop_grace` (default 30 s; `--now`
   skips to step 2).
2. **SIGTERM the process group.** Claude kills its Bash tool trees and exits
   143 (S6).
3. **After 3 s, SIGKILL the process group.**
4. **Sweep** (below). This runs on every exit, stopped or not, because tool
   processes live in *their own* process groups (S4, S6) and SIGKILL orphans
   them.

Stopping an agent that isn't running does nothing and succeeds.

`bridle rm <agent>` stops the agent if it's running, then removes its
worktree:

- It refuses if the worktree has uncommitted changes, unless `--force`. It
  checks before stopping a running agent, and again after, since the agent's
  last turn may leave changes.
- It keeps the branch unless `--delete-branch`. It refuses an unmerged branch
  (one not an ancestor of the clone's `HEAD`) up front, with 409, before
  stopping or removing anything, unless `--force` too.
- It finishes if the worktree directory is already gone.
- It revokes the agent's token, drops its undelivered messages, deletes the
  agent's row and emits `agent.removed`. Its turns (so its usage), events
  and transcript under `.bridle/agents/<id>/` are kept.

## Containment

The implementation today uses the process table. A `Containment` trait
(research 01 §5.2) is defined for a cgroup implementation on Linux, but isn't
wired in yet.

- **Track**: every 2 s, and just before any stop, snapshot the process table
  (`ps -axo pid,ppid,pgid,lstart`). Add every descendant of the agent's pid to
  the agent's *seen* set, keyed by pid + start time so that a reused pid is
  never killed. This catches tool process groups before they re-parent.
- **Sweep**: SIGTERM every seen process still alive with a matching start time,
  wait 2 s, SIGKILL the remainder, and emit `agent.orphans_killed` with the
  count when it killed anything.
- **Not covered**: double-forked daemons that detach between two scans
  ([[process-containment-on-macos-9trg|spike 9trg]]), and Linux specifics such
  as `lstart` locale and cgroups ([[process-containment-on-linux-2mj9|spike 2mj9]]).

## What bridle records per agent

Every raw line in and out goes to `.bridle/agents/<id>/transcript.jsonl`, in
spike 01's format. From the parsed stream, bridle keeps:

- **agent row**: state, pid and start time, session id, turn count, last event
  time, current turn's start time, cumulative cost, worktree and branch;
- **per turn**: start and end times, result subtype, `is_error`,
  `terminal_reason`, the four token counts and the turn's cost (the difference
  between consecutive cumulative counters);
- **account**: the latest `rate_limit_event` info (it arrives once per process,
  S8);
- **events**: `agent.*`, `turn.*`, `message.*`, `permission.denied`, and
  `tool.use` (name and the command, path or pattern, no output)
  ([[docs/design/agent-host/api#Events|events]]).

Transcript lines are tagged `in`, `out`, `err` or `note` with a millisecond
timestamp. The spawn's argv and pid are recorded as notes, and a resume
appends to the same file.

Assistant content arrives **one content block per event** (surprise 6).
Bridle emits `agent.text` per text block and doesn't try to reassemble
messages.
