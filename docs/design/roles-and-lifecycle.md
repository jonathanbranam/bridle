# Roles and the task lifecycle

## Roles

Split by what each role may decide (research 09 §4):

| Role | Default model | Decides | Never |
|---|---|---|---|
| **Human** | — | priorities, product questions, **acceptance** | watch agents type |
| **Orchestrator** | the human's own agent, *not part of bridle*, optional | whatever the human delegates to it: relaying, summarising, steering | act as the human (it has its own `external` identity) |
| **Manager** | Opus/Fable-class, long-lived, hosted by bridle | decomposition, plans, ordering, conflict arbitration, what to ask the human | implement bulk code; accept |
| **Worker** | Sonnet-class, per task | how to implement a planned task; negotiating conflicts with peers | change design silently; accept; talk to the human directly |
| **Reviewer** | strong model, never the task's implementer | whether a diff matches its plan and specs | fix what it reviews |
| **Integrator** | *not an agent* — bridle itself | merge order, conflict probes, rebase notices | resolve a semantic conflict |

- **The orchestrator is the human's interface**, running anywhere (laptop,
  the workforce host, or hosted by bridle as a role like any other). Bridle
  must run without it. It drives bridle through the same CLI and API as
  everything else ([[docs/design/agent-host/operating-model|operating model]]).
- **The manager holds the judgement.** Bridle is mechanism: it spawns,
  delivers, supervises, records and integrates. Deciding what to work on is
  the manager's job. "Start working" means starting the manager.

Models and roles are set in `workflow.toml` per layer, so a project can make its
reviewer cheaper or its worker stronger. Until the layers exist, they are set
in `[roles.*]` in `<repo>/.bridle/config.toml`
([[docs/design/agent-host/roles-and-config|roles and config]]).

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

- **ready** is computed: planned, no open blockers, no unanswered questions.
  `bridle ready` is the dispatch primitive.
- **claimed** carries a lease renewed by the agent's activity, which the
  daemon already sees on every agent's stream, so no heartbeat hook is needed
  for bridle-hosted agents. If the lease expires, the task returns to ready,
  and whatever the worker wrote on it goes to the next claimant (research 10
  §5).
- **Kinds** change the gates and the prime: `feature`, `bug`, `chore`,
  `question`, `research`, `explore` ([[docs/design/explorations|explorations]]), `arch-revision` ([[docs/design/architecture-tier|architecture]]) and
  `re-evaluate` ([[docs/design/traceability|traceability]]).
- **integrated** means merged to the integration branch with tests green.
  **accepted** means the human said yes. They are separate states, so specs fold
  continuously while acceptance stays with the human (research 10 §3.2).

## The loop, from the manager's seat

```
bridle ready --all                      # what can move, across projects
bridle task new "Fix vitest config" -k chore            → tw-c0f1
bridle task new "Watch: ratings filter" -k feature      → tw-7fa2
bridle dep add tw-7fa2 --blocked-by tw-c0f1
bridle plan tw-7fa2                     # manager writes plan + spec edits + impact
bridle spawn worker tw-c0f1             # worktree + session, claims the task
bridle wait tw-c0f1 --until integrated  # run as background Bash; manager is woken
…
bridle spawn worker tw-7fa2             # now ready
bridle review                           # human's batch: diffs, scenarios, verification notes
```

The manager does not have to stay alive for this. The edge, the wait and the
plan are all in the store, so a new manager session can run `bridle prime` and
pick up where the last one stopped.
