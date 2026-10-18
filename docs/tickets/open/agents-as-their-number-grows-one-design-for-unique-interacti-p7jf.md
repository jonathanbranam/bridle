---
id: br-p7jf
title: "Agents as their number grows: one design for unique interactive roles and background agents"
kind: epic
created: 2026-10-18
created_by: external:advisor/product-manager
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-z485, br-ae9f, br-gtzx, br-gcvj, br-j6d3, br-gb97, br-pdmd, br-r9vh]
---

## The ask

The human, 2026-10-18 ~8:45 AM ET, to advisor (product-manager), verbatim:

> That is the right idea for the admin, but I don't want more interactive sessions on RC. Let's
> move the orch to background before pursuing the admin. Sequence that work.
>
> We need an epic and a clear design for managing the increase in unique interactive agents like
> the aide, pdm, advisors, etc. and also for background agents as well. I don't have the design
> yet in my head.
>
> I think we would be ok to move the orch to background as a first step.

(RC: Claude Code's Remote Control, how the human reaches interactive sessions from the phone.)

## The epic

- **Outcome:** bridle manages its growing set of agents one way: the unique interactive roles
  (orchestrator, aide, PdM, advisors, and new ones like the admin, gb97) and the background agents
  (managers, PM, workers, designers). The human isn't left keeping a pile of RC sessions alive.
- **Done when (PdM draft, for the human to confirm):** a design the human approved says which roles
  run in the background, how the human reaches each, how every agent is named, tracked and retired,
  and in what order they move; the orchestrator runs in the background (z485); no new role
  adds an RC session.
- **Theme:** `agents-and-cli`. Absorbs the roadmap's epic `background-agents` (ae9f).

## Proposal

- **Problem:** each new unique role (aide per project, PdM, named advisors, soon admin) is another
  interactive session on RC, with its own waiter, handover and restart path. Background agents are
  run differently again. There's no single model for either, and the human has none in mind yet.
- **Approach:** in order:
  1. z485: move the orchestrator to the background (design pass, the human's review, build). The
     human's chosen first step.
  2. Grill the human (skill bridle-grill) on the whole picture, then a designer pass with options:
     which roles stay interactive, how the human talks to background ones (gcvj), seats and names
     (gtzx), delivery without waiters (ae9f), lifetimes (start on demand, settle and stop: w2hj,
     r9vh).
  3. Builds as child tickets from the approved design.
  4. The machine admin role (epic j6d3, gb97) after step 1, as a background agent.
- **Scope:** the design and the order. Builds beyond z485 are their own tickets.

## Child and related tickets

- z485: orchestrator to the background (first).
- ae9f: interactive agents to the background (was epic `background-agents`).
- gtzx: seats, every role a named tracked seat (held for the human's review of P1-P10, Q1-Q4).
- gcvj: research, replacing RC with bridle's own way to reach agents.
- Related: j6d3 / gb97 (admin, after z485), pdmd, py93, ts6b, r9vh, w2hj, kuw2.
