# Coordination and communication

## Edges

| Edge | Meaning | Affects ready? |
|---|---|---|
| `blocks` | B cannot start until A is integrated (or a named state) | yes |
| `parent` | decomposition | parent closes when children close |
| `discovered-from` | found while working on another task | no |
| `related`, `supersedes`, `duplicates` | provenance | no |

Edges can cross projects (`hx-19ab blocked-by tw-7fa2`), which turns "engine
first, host second" into something the tool enforces.

**Built so far:** the edges table (`from`, `to`, `kind`) and `bridle dep
add|rm`, durable the same way a task is: a SQLite fast index plus a copy on
the state branch, written in the same logical operation
([[docs/design/agent-host/storage|storage.md]]). Only `blocks` is acted on:
`ready` (below) treats any `blocks` edge whose `from` task isn't `dropped` or
`integrated` as still blocking the `to` task (`accepted` doesn't exist yet). The other four
kinds are recorded but not yet acted on (`parent` doesn't yet close a parent
when its children close). `ready [--all] [--role]` is built too; `--role` is
presently a no-op, since tasks don't carry a role field yet.

**Incidents are a task kind** (`incident`; potential = `open`, active = `planned`, resolved =
`integrated`), not a separate record; the only extra is a broadcast notice while one is active
([[docs/design/agent-host/incidents|incidents]], not built).

## Messages

A message goes to an agent, a task (all current and future claimants), a role
(`manager`) or `human`. Kinds:

| Kind | Effect |
|---|---|
| `note` | informational; appended to the task's thread if addressed to a task |
| `question` | **blocks** the task until answered; to `human` it shows in the human's `bridle inbox`. A question is not a separate record: it lives inline in the thread of the task it blocks, on the state branch. The daemon also indexes open questions in SQLite so `bridle inbox` can show them without walking the state branch ([[where-questions-live-on-the-state-branch-c5a8|decided]]) |
| `answer` | unblocks; the answer is written durably to the task record, inline in the same thread |
| `handoff` | "here is where I left it" when releasing a claim |
| `conflict` | opened by the impact registry between two claimants ([impact registry](docs/design/impact-and-conflicts.md)) |
| `system` | from bridle: rebase needed, blocker cleared, lease lapsing |

Messages addressed to a task are durable: they go to the state branch as part of
the task's thread, which gives later claimants the context Gas Town calls
*seance*. Agent-to-agent chatter is kept only in the database and expires.

**Built so far:** messages to an agent or to `human`, of kind `note` or
`question`, with replies linked by `reply_to` and delivery tracked
([[docs/design/agent-host/messages|messages and delivery]]). `bridle send
role:<name>` fans a message out to every live agent currently holding that
role — the same delivery path as sending to one agent, just one message row
per matching agent (`POST /v1/messages` returns the list); sending to a role
with no live agents is a 404. `bridle ask` also sends a pointer message (kind `question`) to its `--to`
(default: the caller's spawner, else `human`), and `bridle answer` one back to the asker; the
thread stays the record. Messages to a
task: `TaskManager::ask_question`/`answer_question`
(`crates/bridle-daemon/src/tasks.rs`) insert a `question`/`answer` message
addressed to the task (`to_kind = task`, [[docs/design/storage#The daemon's
database|storage.md]]), append the matching thread entry, and keep an
`open_questions` SQLite index so `is_ready` excludes a task with one open —
without walking the state branch, per
[[where-questions-live-on-the-state-branch-c5a8|decided]]. `TaskManager::note_task`
is the same shape without the open-question bookkeeping: it inserts a `note`
message addressed to the task, appends a `note` thread entry, and has no
effect on readiness (`bridle task comment`). `handoff`/`conflict`/`system`
message kinds and send-to-task from `bridle send` are not built: bridle's own notices are
`note`s from the `system` principal.

### Waking the manager

When a task is created (`task.created`) and no `product-manager` agent is running, nobody
triages it, so the daemon sends the running `manager` agent a `note` from `system`: "task
<id> filed: <title>; open tasks: N. Plan it or queue it." (`AgentManager::note_task_filed`).
It is coalesced to at most one message per minute: the first goes out at once, and tasks
filed inside the window are listed together in one message when it ends. Nothing is sent
when a PM is running. With no manager running (`autostart = false`), the note goes to
`external:orchestrator` instead, which wakes it to start one.

### Telling workers main moved

When a task lands (`bridle land`, or `bridle task done`, which `land` calls), the daemon sends every other running `worker` that has a
worktree branch or a claimed task a `note` from `system`: "main moved: task <id> (<title>)
landed at <sha>; files changed: <up to 15 paths, then +N more>. Rebase or merge main into your
branch when at a safe point, before your next commit." (`AgentManager::note_main_moved`). The
file list comes from `git diff-tree` on the commit and is left out if git fails. The agent whose
task (claim or branch) landed gets nothing, nor do the manager, PM or orchestrator. At most one
message per agent per minute; landings inside the window are joined into one. A claimant
whose declared impact overlaps what the landing changed also gets `spec changed under you: …`
appended ([[docs/design/impact-and-conflicts|impact and conflicts]]); the notice still goes to
every busy worker, the overlap only adds the detail.

## Questions do not stop work

```
worker hits a question ─► bridle ask tw-7fa2 --to manager "…"
   task → blocked; worker writes a handoff note, releases or keeps claim
   worker ─► bridle ready ─► claims something else
manager answers, or forwards to human ─► answer lands on the task ─► task ready
   ANY worker may claim it — not necessarily the one that asked
```

## Human to-dos

A to-do for the human is a task claimed by the `human` principal (`bridle task new
--for-human`, which plans and claims it in one step and sends one inbox message pointing at
it). Unlike an agent's claim it has no lease: the lease check measures an agent's activity,
so a claim by `human` is never released. The human finishes it with `bridle task done <id>`,
which needs no commit for a task the human claimed. The orchestrator lists
`bridle task list --claimed-by human` at every start and tells the human first.

A to-do has a priority (`high`, `normal` (default), `low`; `task new --priority`, changed with
`bridle task priority`), and the list is sorted by it. Whoever asked can rescind it with
`bridle task drop --reason`; the human gets an inbox note with the reason and the to-do leaves
their list. The audit trail is the task's thread and the event log, each with who and when:
created (with its priority), re-prioritized (`task.priority` event, `high -> low`), rescinded
(`dropped: <reason>`) and done. Priority ranks only the human's to-dos; agent work is ranked by
the queue's tiers.

## How agents actually hear things (Claude Code integration)

Workers are separate headless `claude -p` stream-json sessions that the daemon
owns ([[docs/design/agent-host/agents|agents]]), so bridle holds each agent's
stdin and stdout directly:

| Mechanism | Used for | State |
|---|---|---|
| **stdin user message** | a message to the agent. If the agent is working, it is folded into the running turn at the next tool boundary; if idle, it starts a turn. Acked by `--replay-user-messages` | built |
| **stdout stream** | status, turn boundaries, tool use, usage, and liveness for leases | built |
| **the first user message** | task-specific context at spawn (today: the spawn prompt; later: `bridle prime` output) | built (prompt only) |
| **`--append-system-prompt-file`** | role-scoped rules and guides, identical for every agent in a role so the prompt cache holds | built (role prompt + bridle preamble) |
| **Stop hook** → `bridle stop-check` | refuses to let a worker stop with an unreleased claim and no handoff note, a finished tree whose HEAD hasn't passed `commands.check_worker` (the hook runs it), or a finished tree with no task summary and `done:` report | built |
| **`bridle wait` as background Bash** | an agent is re-invoked when a task reaches a state or a message arrives. For bridle-hosted agents a stdin message does the same | with tasks |
| **PreToolUse hooks** | enforce locked rules mechanically, e.g. workers can't run `git push` or `bridle accept` | with layers |

Hooks cost about 0.9 s each ([spike 01](docs/spikes/01-stream-json-findings.md),
S11), so they're kept for enforcement, not delivery. An agent bridle doesn't
host (the human's orchestrator, an interactive session) reads its inbox with
`bridle inbox` or follows `bridle events --follow`.
