# Roles and the task lifecycle

> **Status (checked 2026-10-03):** Built and in use: the human, orchestrator, manager, worker, product-manager and integrator (`bridle task land`) roles; task states `open`, `planned`, `claimed`, `dropped`, `integrated`, `reopened` with `ready` derived; claim leases; the queue (`bridle queue`, `queue set|add-tier`, `bridle ready`); task kinds, including `re-evaluate` tasks opened when an `arch-revision` is done · Planned: the reviewer role (no role file or config), `in_review` and `accepted` states, `needs-input`, per-kind gates, roles and models from `workflow.toml`, `bridle review`

## Roles

Split by what each role may decide (research 09 §4):

| Role | Default model | Decides | Never |
|---|---|---|---|
| **Human** | — | priorities, product questions, **acceptance** | watch agents type |
| **Orchestrator** | the human's own agent, *not part of bridle*, optional | whatever the human delegates to it: relaying, summarising, steering | act as the human (it has its own `external` identity) |
| **Manager** | Opus/Fable-class, long-lived, hosted by bridle | decomposition, plans, ordering, conflict arbitration, what to ask the human | implement bulk code; accept |
| **Worker** | Sonnet-class, per task | how to implement a planned task; negotiating conflicts with peers | change design silently; accept; talk to the human directly |
| **Reviewer** (planned) | strong model, never the task's implementer | whether a diff matches its plan and specs | fix what it reviews |
| **Integrator** | *not an agent* — bridle itself (`bridle land`, [[docs/design/agent-host/roles-and-config|roles and config]]) | conflict probes, the merge gate, `main moved` notices | resolve a semantic conflict |

- **The orchestrator is the human's interface**, running anywhere (laptop,
  the workforce host, or hosted by bridle as a role like any other). Bridle
  must run without it. It drives bridle through the same CLI and API as
  everything else ([[docs/design/agent-host/operating-model|operating model]]).
- **The manager holds the judgement.** Bridle is mechanism: it spawns,
  delivers, supervises, records and integrates. Deciding what to work on is
  the manager's job. "Start working" means starting the manager.

Models and roles are to be set in `workflow.toml` per layer (planned), so a project can make its
reviewer cheaper or its worker stronger. Today they are set
in `[roles.*]` in `<repo>/.bridle/config.toml`
([[docs/design/agent-host/roles-and-config|roles and config]]).

**Which branch the manager and worker merge into, push to, and branch new
work from is also a project setting**, `[branches]` in the same file — one
knob for two patterns (trunk, or dev+release), plus the branch a project
trial uses instead of its real `main`/`dev`:
[[docs/design/agent-host/operating-model|operating model]], "Branch
pattern".

## Task lifecycle

```
          ┌────────── question posted ─────────┐
          ▼                                     │
 open ─► planned ─► ready ─► claimed ─► in_review ─► integrated ─► accepted
   │                  ▲         │           │             │
   │                  └─ lease lapses / released           └─► reopened
   └──────────────────────────────► dropped (reason required)

 blocked        derived: an open `blocks` edge, or an unanswered question
 needs-input    derived: a question addressed to the human
```

Built states: `open`, `planned`, `claimed`, `dropped`, `integrated`, `reopened`
(`TaskState`, `bridle-api/src/types.rs`); `ready` and `blocked` are derived.
`in_review`, `accepted` and `needs-input` are planned.

- **ready** is computed: planned, no open blockers, no unanswered questions.
  A blocker counts as open unless it's `dropped` or `integrated` (`accepted`
  doesn't exist yet; see coordination.md, Edges). `bridle task plan <id>` makes the `open ->
  planned` transition (`TaskManager::plan_task`); `open` is the only state it
  accepts, so a task already planned, claimed, dropped or reopened is a
  conflict.
- **The queue is a separate record, not a field on the task.** Tasks stay
  the *what* (kind, body, real `blocks` edges for actual dependencies —
  never for ordering); the queue is the *when*, an ordered list of tiers,
  each tier a set of equally-ranked task ids (tier 1 before tier 2). A task
  not listed in any tier is backlog. It's PM-owned (the human may override, and so may
  `external:orchestrator`, acting PM on a small project with none): `bridle queue set --tier <task,task> --tier <task,task>`
  replaces the whole queue (reorder/add/remove are all "resend the tiers in
  the shape they should be"), and `bridle queue add-tier <task>...` appends
  one tier at the back. Every other principal, the manager included (and a visitor orchestrator), only
  reads it (`server.rs::require_pm_or_human`, gating `POST /v1/queue` and
  `POST /v1/queue/tiers`). Durable on the state branch's `queue.toml`
  (storage.md, "The state branch"), since there's no SQLite table for it at
  all — the in-memory cache is hydrated straight from that file at
  `TaskManager::open`/`rebuild_from_state_branch`.
- **The manager is mechanical** about the queue: it takes from the highest
  tier with a startable task (`TaskManager::highest_startable_tier`:
  `ready`, and therefore unclaimed too, since `is_ready` already requires
  `planned`), picking within a tier by load/free worker slots. It never
  moves a task between tiers; if the top tier's only task is blocked on an
  unmet dependency, it takes from the next tier down rather than idling.
  `bridle ready` (no `--all`) now returns exactly this — the highest
  startable tier's ready tasks, `GET /v1/tasks?top_tier=true` — not every
  ready task project-wide; a task outside the queue is never returned here,
  even if it happens to be ready. `bridle queue` is the read-only view:
  claimed tasks with their worker first, then the tiers in rank order, each
  task marked startable or blocked.
- **claimed** carries a lease renewed by the agent's activity, which the
  daemon already sees on every agent's stream, so no heartbeat hook is needed
  for bridle-hosted agents. If the lease expires (`[daemon] claim_lease_after`, default 10 min), the task returns to ready,
  and whatever the worker wrote on it goes to the next claimant (research 10
  §5). Claims are durable: mirrored to the state branch alongside the task
  and edge state, so `bridle rebuild` restores who's working what
  (storage.md, "claims").
- **Kinds** are to change the gates (planned) and the prime (today only `explore`, via
  `bridle prime worker --task`, which no spawned worker runs): `feature`, `bug`, `chore`,
  `question`, `research`, `explore` ([[docs/design/explorations|explorations]]), `arch-revision` ([[docs/design/architecture-tier|architecture]]) and
  `re-evaluate` ([[docs/design/traceability|traceability]]).
- **The product manager** (`product-manager`) is a project-defined role, not a built-in: it
  triages `open` tasks (`plan`) and owns the queue. Bridle's own `.bridle/config.toml`
  defines it with `autostart = true`. While none runs, the daemon tells the manager when a task
  is filed (coordination.md, "Waking the manager").
- **integrated** means merged to the integration branch with tests green.
  **accepted** means the human said yes. They are separate states, so specs fold
  continuously while acceptance stays with the human (research 10 §3.2).

## The loop, from the manager's seat

```
bridle ready --all                      # what can move, across projects
bridle task new "Fix vitest config" -k chore            → tw-c0f1
bridle task new "Watch: ratings filter" -k feature      → tw-7fa2
bridle dep add tw-7fa2 --blocked-by tw-c0f1
bridle task plan tw-7fa2                # PM: open -> planned, ready to build
bridle queue add-tier tw-c0f1 tw-7fa2   # PM: queues them, tw-c0f1 first
bridle spawn worker --prompt "Claim tw-c0f1 …"   # worktree + session; the worker claims it
bridle wait tw-c0f1 --until integrated  # run as background Bash; manager is woken
…
bridle spawn worker --prompt "Claim tw-7fa2 …"   # now ready
bridle review                           # planned: human's batch: diffs, scenarios, verification notes
```

The manager does not have to stay alive for this. The edge, the wait and the
plan are all in the store, so a new manager session can read them back
(`bridle queue`, `bridle task list`; there is no `bridle prime manager`) and
pick up where the last one stopped.
