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

## Messages

A message goes to an agent, a task (all current and future claimants), a role
(`driver`) or `human`. Kinds:

| Kind | Effect |
|---|---|
| `note` | informational; appended to the task's thread if addressed to a task |
| `question` | **blocks** the task until answered; to `human` it shows in `bridle inbox --human` |
| `answer` | unblocks; the answer is written durably to the task record |
| `handoff` | "here is where I left it" when releasing a claim |
| `conflict` | opened by the impact registry between two claimants ([impact registry](docs/design/impact-and-conflicts.md)) |
| `system` | from bridle: rebase needed, blocker cleared, lease lapsing |

Messages addressed to a task are durable: they go to the state branch as part of
the task's thread, which gives later claimants the context Gas Town calls
*seance*. Agent-to-agent chatter is kept only in the database and expires.

## Questions do not stop work

```
worker hits a question ─► bridle ask tw-7fa2 --to driver "…"
   task → blocked; worker writes a handoff note, releases or keeps claim
   worker ─► bridle ready ─► claims something else
driver answers, or forwards to human ─► answer lands on the task ─► task ready
   ANY worker may claim it — not necessarily the one that asked
```

## How agents actually hear things (Claude Code integration)

| Mechanism | Used for |
|---|---|
| **SessionStart hook** → `bridle prime` | identity, claimed task, unread messages, facts, role-scoped rules |
| **PostToolUse / UserPromptSubmit hook** → `bridle inbox --inject` | new messages injected into the running agent's context between tool calls; also renews the claim's heartbeat. Must be fast (<20 ms) and silent when there's nothing new |
| **Stop hook** → `bridle stop-check` | refuses to let a worker stop with an unreleased claim and no handoff note |
| **`bridle wait` as background Bash** | the driver or a worker is re-invoked when a task reaches a state or a message arrives |
| **PreToolUse hooks** | enforce locked rules mechanically, e.g. workers can't run `git push` or `bridle accept` |

> **Update 2026-09-27:** [`research/01-agent-runtime.md`](docs/research/01-agent-runtime.md)
> answers most of this. Hooks do fire in subagents and carry `agent_id`. The
> recommendation is headless `claude -p` stream-json workers that bridle owns
> directly: stdin to wake an agent, hooks for mid-turn injection and status,
> and a bridle MCP server for tools.
>
> **Update 2026-09-27 (later):** spike 01 verified the stream-json host, and
> [`agent-host.md`](docs/agent-host.md) designs the daemon, API and agent host built
> on it. Mid-turn delivery there uses stdin rather than hooks.

**To verify before building on it:** whether hooks fire inside `Agent`-tool
subagents with enough identity (session/agent id) to tell them apart, or whether
workers must be separate `claude` sessions started by `bridle spawn` (tmux +
worktree). The design supports both, since everything goes through the store.
Which is the default depends on the answer.
