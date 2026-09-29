# Brief: how the bridle task system works

As of 2026-09-29. Status words: **built** (in the code and CHANGELOG), **partly built**,
**planned** (design docs only). Companion to [[docs/briefs/specs|the specs brief]].

## What a task is

A task is one piece of work a worker can finish on one branch. It has an id (`br-7fa2`:
the project's prefix plus four hex characters), a title, a **kind** (`feature`, `bug`,
`chore`, `question`, `research`, `explore`, `arch-revision`, `re-evaluate`), an optional
**size** (S, M, L, meant for picking small work when budget is short), a **body** (the
brief: goal, files, acceptance, out of scope), a **thread** (notes, questions, answers,
the merge commit) and a **summary** (how it was implemented, written by the worker at the
end and used as the landing commit's body). **Built.** Two kinds do something: only an
`arch-revision` task may change `design/architecture/**` (a hook blocks the edit, and
landing refuses it otherwise), and landing one opens `re-evaluate` tasks. The kind
`question` is a label only. Size is stored and shown; nothing acts on it yet.

**States:** `open` (idea, being written up) -> `planned` (ready to build) -> `claimed` (a
worker holds it) -> `integrated` (merged). `dropped` (needs a reason) and `reopened` (brings
a dropped or integrated task back) are the side exits. **Built.**

## Creating and planning

`bridle task new "title" --kind feature [--size s] --body-file f` creates an `open` task.
`task edit` changes title or body, `task note` adds a plain note, `task list` and
`task search <words>` find tasks, `task show` prints one in full. `bridle task plan <id>`
moves `open` -> `planned`; the code lets any principal do it, so "the PM plans" is a role
convention, not a rule the daemon enforces. **Built.**

## The queue: who ranks what

Planned tasks are not automatically startable. The **queue** is a separate ordered list of
*tiers*; tasks in one tier rank equally, tier 1 first. A planned task in no tier is
*backlog*: `bridle ready` never shows it. `bridle queue` prints claimed tasks with their
worker, then each tier with each task marked startable or blocked. `bridle queue set` (the
whole queue, resent in the shape you want) and `queue add-tier` may be run only by the PM or
the human; the daemon enforces it. `bridle ready` returns the highest tier that has a
startable task, skipping a tier stuck on a dependency. **Built.**

## Dependencies, ready, questions

`bridle dep add <task> --blocked-by <other>` adds a `blocks` edge. Other kinds (`parent`,
`discovered-from`, `related`, `supersedes`, `duplicates`) are stored and shown but don't
affect anything. A task is **ready** when it is `planned`, every task blocking it is
integrated (or dropped), and it has no open question. `dep rm` removes an edge. **Built.**

`bridle ask <task> "text"` records a question on the task and takes it out of `ready` until
`bridle answer <task> "text"`. A task has at most one open question. `ask` also sends a
pointer message (kind `question`) to `--to` (an agent, `role:NAME`, `external:NAME` or
`human`), by default the caller's spawner or, for a human caller, the human; `answer` sends
the asker a pointer back. The thread stays the record. **Built.**

## Claims and leases

`bridle claim <task>` moves a ready task `planned` -> `claimed` for the caller; one claimant
at a time. `bridle release <task>` puts it back. There's no renewal call: the daemon
watches the claiming agent's own activity and releases the claim after 10 minutes
(`claim_lease_after`) without any. Dropping or integrating a claimed task clears the claim.
**Built.** The `stop-check` hook stops a worker from finishing while its claimed task has
no thread entry, no summary or `done:` report, or (when configured) no recorded passing
check. **Built.**

## Messages tied to tasks

`bridle send <agent> --task <id> "text"` puts the text on the task's thread and sends the
recipient a short pointer; `task note <id> --notify <agent>` does the same. Briefs, done
reports and findings travel this way, so the thread is the task's record. `bridle wait
<task> [--until state] [--or-message]` blocks until the task changes or a message arrives.
**Built.**

## Roles and the trip from idea to landed

- **Human**: sets direction, answers questions, approves what touches their projects.
- **Orchestrator**: the human's assistant (bridle's supervision of it is **planned**).
- **PM** (`product-manager`): triages tickets, writes task briefs, plans them, owns the queue.
- **Manager**: takes from `bridle ready`, spawns a worker per task, reviews, lands. Cannot
  reorder the queue.
- **Worker**: claims the task in its own worktree and branch, builds, writes the summary,
  reports to the manager.

Path: PM creates and plans the task and queues it -> manager sees it in `bridle ready`,
spawns a worker -> worker claims, commits on its branch, merges the integration branch in,
runs the check, `bridle task summary`, reports -> manager reviews, runs `bridle land <task>`
-> the daemon refuses architecture edits by a non-arch task, probes for conflicts, squashes
the branch into **one commit** on the integration branch (`<task id>: <title>`, summary as
body, `Task:` and `Branch:` trailers), runs the project's check on that result, and advances
the branch only if the check passes and the branch hasn't moved, then marks the task done
(`task done --commit C --branch B`, which also removes the branch's agents and worktree)
and notifies running workers ("spec changed under you" when their declared impact was
touched). **Built.** The roles' split of duties is prompts and `require_not_worker`-style
checks on agent lifecycle, not a general permission system. Landing is refused if the
check fails; the check is skipped (with a note) if none is configured.

## Storage

SQLite (`bridle.db`) is the fast index: the task row (id, title, kind, state, size), edges,
open questions, claims, messages. The **state branch** in git holds the durable copy:
task files (body, thread), `edges.toml`, `claims.toml`, `queue.toml` and a monthly event
log. Writes go to SQLite at once and to the branch in batches. `bridle rebuild` recreates
the tables from the branch on a fresh clone and refuses if the database has rows.
Messages, conflicts and ports are SQLite only and are lost in a rebuild (claims are restored). **Built.**

## The human's touchpoints

Today: `bridle inbox` and `bridle status` (what's waiting), `bridle task show|list|search`,
`bridle queue` (read; the human may also edit it), `bridle ready`, answering with
`bridle answer` or `bridle send`, `bridle task drop|reopen`, `bridle tui`, and the budget
`hold`/`release`. The human is not asked to approve a plan: a plan gate for protected
requirements is **planned** (the marker parses; nothing enforces it).

## Gaps and known rough edges

- `bridle ready --role` is accepted and ignored (tasks carry no role).
- Any principal can `plan`, `done` or `drop`; only the queue is gated.
- A crash between a write and the next state-branch flush can lose an edit to a task's body
  or thread (not its state).
- Claim leases judge liveness by the agent's activity: a long single tool call or a stopped
  agent can lose its claim.
- Size, `parent`/`related`-style edges and the `question` kind are inert labels.

## Where design docs disagree with the code

- `docs/design/roles-and-lifecycle.md` and `coordination.md` describe roles and checks
  (plan gates, protected requirements, role-filtered `ready`) beyond what is enforced; see
  above.

## Decisions for you

1. Should the daemon restrict `plan`/`drop`/`done` by role, or is convention enough?
2. Should a question on a task message the manager (or you) automatically?
3. Is the 10-minute claim lease right for long-running work?
