# Roles and the task lifecycle

> **Superseded in part** (note added when this file was split out of `design.md`):
> `docs/agent-host.md` §1.2 and §2 split the driver into an **external orchestrator** (the human's own agent, optional, anywhere) and a **manager** agent that bridle hosts. Workers stay as described here.

## Roles

Split by what each role may decide (research 09 §4):

| Role | Default model | Decides | Never |
|---|---|---|---|
| **Human** | — | priorities, product questions, **acceptance** | watch agents type |
| **Driver** | Opus/Fable-class, long-lived | decomposition, plans, ordering, conflict arbitration, what to ask the human | implement bulk code; accept |
| **Worker** | Sonnet-class, per task | how to implement a planned task; negotiating conflicts with peers | change design silently; accept; talk to the human directly |
| **Reviewer** | strong model, never the task's implementer | whether a diff matches its plan and specs | fix what it reviews |
| **Integrator** | *not an agent* — bridle itself | merge order, conflict probes, rebase notices | resolve a semantic conflict |

Models and roles are set in `workflow.toml` per layer, so a project can make its
reviewer cheaper or its worker stronger.

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
- **claimed** carries a lease renewed by the heartbeat hook ([[docs/design/coordination#How agents actually hear things (Claude Code integration)|how agents hear things]]). If the lease
  expires, the task returns to ready, and whatever the worker wrote on it goes
  to the next claimant (research 10 §5).
- **Kinds** change the gates and the prime: `feature`, `bug`, `chore`,
  `question`, `research`, `explore` ([[docs/design/explorations|explorations]]), `arch-revision` ([[docs/design/architecture-tier|architecture]]) and
  `re-evaluate` ([[docs/design/traceability|traceability]]).
- **integrated** means merged to the integration branch with tests green.
  **accepted** means the human said yes. They are separate states, so specs fold
  continuously while acceptance stays with the human (research 10 §3.2).

## The loop, from the driver's seat

```
bridle ready --all                      # what can move, across projects
bridle task new "Fix vitest config" -k chore            → tw-c0f1
bridle task new "Watch: ratings filter" -k feature      → tw-7fa2
bridle dep add tw-7fa2 --blocked-by tw-c0f1
bridle plan tw-7fa2                     # driver writes plan + spec edits + impact
bridle spawn worker tw-c0f1             # worktree + session, claims the task
bridle wait tw-c0f1 --until integrated  # run as background Bash; driver is woken
…
bridle spawn worker tw-7fa2             # now ready
bridle review                           # human's batch: diffs, scenarios, verification notes
```

The driver does not have to stay alive for this. The edge, the wait and the
plan are all in the store, so a new driver session can run `bridle prime` and
pick up where the last one stopped.
