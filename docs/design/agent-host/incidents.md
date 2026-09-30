# Incidents

Design for ticket [[incident-notices-that-retract-themselves-nc7r|nc7r]], revised to the human's
decision (2026-09-29, in nc7r and ex9q): **an incident is a task of kind `incident`.** **Not
built.** There is no incidents table and no new record type: an incident has a state, an owner,
a body and a thread (comments, updates), which is what a task already is. The only new
machinery is the **notice**: a persistent broadcast to every agent while an incident is active.
The notice is a `system` note in the existing message queue ([[messages]]), withdrawn if it's
still `pending` or `held`.

An agent that already read a notice can't be made to unread it (stdin can't be unsent), so
"retract" means two things: undelivered notices are **dropped**, delivered ones get a short
"resolved".

## 1. The task kind

A task created with `bridle task new "<title>" -k incident --body ...` (`TaskKind` gains
`incident`). Everything else about a task applies: id, body, thread, `bridle task
show|edit|list|search`, the state branch copy, `--json`.

**States reuse the existing ones** (no new `TaskState`):

| Incident | Task state | How |
|---|---|---|
| potential | `open` | anyone files it: `bridle task new -k incident` |
| active | `planned` | the orchestrator promotes it: `bridle task plan <id>` |
| resolved | `integrated` | the orchestrator closes it: `bridle task done <id>`; `--commit` isn't required for this kind (there's no commit), a `--resolution` line goes in the thread instead |
| rejected | `dropped` | the orchestrator drops a potential one: `bridle task drop <id> --reason` |
| recurred | `reopened` | as for any task; the notice goes out again on the `planned` transition |

`claimed` isn't used. Incidents are **kept out of `queue` and `ready`** (those are
for work to build, and an incident is nobody's to claim): the queue filters `kind = incident`.
"Updates" are thread notes plus `task edit` of the body; they are not a state.

- **Owner:** the orchestrator (`external:orchestrator`). The human may do anything the
  orchestrator may.
- **Who may file** (create in `open`): any principal, agents included; workers file a
  suspected incident this way. Before filing they search open incidents (`bridle task list
  -k incident`, `bridle task search`) so the orchestrator isn't handed duplicates; a
  duplicate is dropped with a reason pointing at the original.
- **Who may promote, resolve, drop:** the orchestrator and the human only (403 otherwise, as for
  lifecycle endpoints, [[principals]]). Commenting is open to all.
- **Retention:** incidents are tasks, so they live on the state branch and survive a lost
  database like any task.
- **The daemon files none in v1.** Candidates are a budget hold, a failed push, a red `main`
  (c8qw's CI watcher); each would be a small hook filing a `potential` incident with
  `filed_by = system`. **Budget holds stay as they are**: their wind-down and resume notices
  carry per-agent instructions that a broadcast body can't.
- **Cross-project: not in v1.** Each daemon has its own tasks and agents. The orchestrator
  files the same incident per affected daemon with `bridle --project <p> task new`
  (it holds one credential per project, principals.md).

## 2. Audience

**Every agent, any role.** Role-scoping isn't in v1 (the cost of not doing it: a notice to a
worker that doesn't care; incidents are rare). If it turns out trivial later, `--role <name>`
maps onto the fan-out target `send` already has.

**External principals** (advisor, orchestrator) have an inbox, not a process, so there is
nothing to deliver to or retract from. They read `bridle status` (§4), which lists active
incidents. The orchestrator, who owns them, doesn't need telling.

## 3. The notice

The notice is a message: `from_principal = system`, `kind = note`, `when = idle`, with one new
nullable column `messages.incident_task` (the task id) linking it to its incident (a new schema
version after V14; storage.md's messages table gets the column). Body: `Incident <id>: <title>\n<body>`.
It goes through the existing send path (`Supervisor::send`), so held/pending, budget holds and
acks apply unchanged. `when idle` because a notice mustn't interrupt a tool call.

- **Promote (`open`/`reopened` → `planned`)** fans one notice out to each live-or-resumable
  agent (running or not: a stopped agent's notice sits `pending`, like any message to it).
- **An agent that starts, resumes or renews while it's active** must not miss it. `resume` and
  `renew` already write every `pending` message; they gain one check first: an agent with no
  notice row for an active incident gets one. A new agent gets a notice row at spawn, after its
  first message (or as its first message if it has none). A notice is **not** put in the system
  prompt: `render_system_prompt` is shared and cache-stable ([[agents]]), and a prompt can't be
  retracted.
- **Update** (`task edit` of the body of an active incident) rewrites the body of any notice
  still `pending` or `held` (the agent sees only the latest) and sends a fresh
  `Incident <id> updated: …` note to agents whose notice is already written or delivered. A
  thread comment sends nothing: agents read the thread with `bridle task show`.
- **Resolve or drop-while-active (`planned` → `integrated`, or `dropped`)**, per notice:
  - `pending` or `held` → `dropped` (event `message.dropped`, as `--drop-held` does). The agent
    never hears of it, and is never told late.
  - `written`, `delivered` or `read` → one `Incident <id> resolved: <resolution or title>`
    note, `when idle`; an agent with several notices for it gets one.
  - A resolved note has no `incident_task`, so it can't itself be dropped. If its agent is
    stopped it stays `pending` and is delivered on resume (an agent that saw the incident
    should hear it ended, however late).
- **Reused:** `send`, `pending`/`held`, `dropped`, acks, resume's flush, the budget hold on
  idle agents. **New:** the `incident_task` link, drop-on-resolve and fill-on-start. No
  coalescing.

## 4. Surfaces

No `bridle incident` command group and no `/v1/incidents` endpoints: the task commands and
routes cover it ([[docs/design/cli]], api.md).

- **CLI:** `bridle task new -k incident`, `task plan|done|drop|edit|show|list|search` as
  above. `task list` gains a `-k/--kind KIND` filter (used for "open incidents").
- **`bridle status`** gets an `incidents` list of the active ones (id, title, age). Every
  principal reads it, external ones included; `--json` carries the same list.
- **Events:** the tasks' own events (task created, planned, done, dropped) already fire, and an
  incident is told apart by its `kind`. The orchestrator's watcher wakes on a created task of
  kind `incident` (a filter change in the watcher, not daemon work). New: `incident.notified`
  is not added; delivery uses the existing `message.*` events.

## Build split

One build: the `incident` kind and the queue filter, `task list -k`, the `incident_task`
column, fan-out on promote, the fill check in spawn/resume/renew, drop-and-resolve on close,
and the `status` list. The human's to-do list (ex9q) is the same pattern on a different
assignee and shares only the task kind machinery, not the notice.
