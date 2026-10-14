---
id: br-8swr
title: A drain treats a manager waiting on just check or a landing as idle, and restarts it mid-landing
kind: bug
created: 2026-10-12
created_by: external:advisor/product-manager
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kt25, br-y55w]
---

## The ask

The human, 2026-10-12 ~10:17 AM ET (to advisor product-manager, dictated, in the grilling session on kx5u/kt25, Q5 on the drain), verbatim:

> 2. The manager was landing a ticket and went idle, but he was just waiting for the results of compiling and running tests. The daemon incorrectly interpreted that as an idle period.
>
> Those two issues should be filed, and we need to work on those as part of a later epic, but we're not going to block the project move MVP for those fixes yet. We'll address them manually for now, which is, I think, perfectly fine.

Earlier, on the kt25 design's question 3 (drain timeout), the human: upgrades "interrupt work bc system isn't designed well (e.g. can't tell a manager is waiting for just check)".

## The problem

A drain's quiet point is "no agent mid-turn and no spawn in flight" (docs/design/agent-host/daemon.md, "Restart in place"). A manager that starts `just check` (or `bridle ticket land`'s integration check) as a background command ends its turn while the build and tests run, so the daemon sees it as idle and the drain goes ahead: the restart stops the manager mid-landing and the check's result is lost.

The ask: the daemon can tell that an agent is waiting on work it started (a check, a land, a background command) and treats that as busy, so a drain waits for it. How it knows (the agent declares it, the daemon tracks the agent's child processes, `bridle ticket land` holds a marker, or other) is for the design.

## Scope and order

- Later epic (drain behaviour review). Not on kx5u's path: during MVP moves the orchestrator watches the drain and makes sure no landing is in flight, by hand.
- Related: br-y55w (a landing's integration check has no timeout).
- Needs a design pass first; no build until the human reviews.

## Decision: pull forward the manual guard (2026-10-14)

The orchestrator (m-12730): this ticket killed a running land four times on 2026-10-14 (05:44Z,
07:50Z, 12:44Z, 15:07Z; ~30-45 min each), and the manual guard can't be applied because
nothing cancels or postpones a drain while self-upgrade runs every 3 h.

Aide's options to the human: A, add now only a small command so the orchestrator can postpone or
cancel a pending restart while a land runs (e.g. `bridle daemon restart --cancel`/`--postpone`),
the full fix staying in the later epic; B, self-upgrade less often; C, leave it.

The human, 2026-10-14 ~6:25 PM ET (to aide), verbatim: "Yes A". So: build the postpone/cancel
now; the full fix (the daemon sees an agent's background work as busy) stays in the later epic.
