---
id: k6jd
title: Managers and the orchestrator fetch origin; divergence from origin is warned (N ahead, M behind)
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [j7r4, 2mtr, 8z7j]
tasks: [br-k6jd]
closed: 2026-10-09T23:11:01Z
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 2: make divergence from origin visible.
The human, 2026-10-09 14:03 EDT, verbatim (comment c2 on j7r4): "Yes, definitely. This is
critical to allow for moving a project from one machine to another. Manager and orchestrator are
trusted to do this."

The ask:
1. Managers and the orchestrator may run `git fetch origin` (read-only), including managers in
   don't-ask mode: allow it in their permissions. Workers stay as they are.
2. The daemon (or `bridle doctor`, and on a timer) fetches and warns when `origin/<integration>`
   is not an ancestor of the local integration branch: "N ahead, M behind". A plain
   `[ahead 23]` hid the incident for hours.
3. The warning reaches the orchestrator (an event and a message), not only a log.

Related: 2mtr (workers report failed fetches plainly), 8z7j (one pusher).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
