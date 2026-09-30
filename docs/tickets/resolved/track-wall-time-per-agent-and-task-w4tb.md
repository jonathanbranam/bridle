---
id: w4tb
title: Track wall time per agent and task, not only tokens and cost
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [mt7r]
---

## The question

`bridle usage` reports tokens and cost. Nothing reports how long work takes
in wall time, or where that time goes. Should bridle track it, per agent and
per task?

The human's words, 2026-09-28:

> Are we tracking wall time as well for tasks and workers? It would be a very
> interesting metric; things are taking longer in wall time than I expected;
> the token use and budget seems on track though.

## What exists

The raw data is stored: `turns.started_at` / `ended_at` per turn, and
`agent.spawned` / `agent.exited` in `events`. The orchestrator computed, from
the database on 2026-09-28, each worker's wall time (spawn to exit) against
its busy time (the sum of its turns). For example:

| Worker | Wall | Busy |
|---|---|---|
| p0-1-tasks | 99 min | 69 min |
| p0-2-edges | 88 min | 41 min |
| context-measure | 83 min | 16 min |
| deflake | 79 min | 22 min |
| p0-3a-questions | 27 min | 21 min |

The gap is time spent waiting: on review, merges, a red `main`, or the
manager. The busy time includes builds and `just check` slowed by the other
workers' load.

Also found: removing an agent deletes its `agents` row, but its `turns` rows
stay, so the turns lose their agent's name. The names had to be recovered from
`agent.spawned` events' branch field.

## Resolution

Implemented and merged 2026-09-28 in commit 72cc135 ("bridle usage reports per-agent busy and wall time"). The feature is now live: `bridle usage` reports per-agent wall time (agent spawn to exit) and busy time (sum of turns) for every worker and task, matching the data computed and analyzed in the ticket's "What exists" section.
