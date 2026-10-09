---
id: jxwr
title: Track and report how long a task takes from pickup to merge, split into agent work, builds, waits and holds (with reasons)
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [rcvb, v6kr]
tasks: [br-jxwr]
---

## The ask

The human, verbatim (2026-10-09 ~3:15 PM ET):

> do we know how long it takes for a ticket to go from being picked up by an agent to merged? Is that a statistic we can track adn report on? If not, let's do that.
>
> Goal: understand how long a ticket takes to be completed; track changes to its lifecycle somewhere so they can be evaluated.
>
> Understand what happens:
>
> 1. how long are tickets blocked or held; ideally with "why" as well
> 2. how long do tickets wait for agent work vs. builds

## What exists today (checked by the aide)

- Every task state change is logged with time and actor in `events/<month>.jsonl` on the `bridle/state` branch, e.g. br-57nt:
  `"" -> pending` 10:02Z (human), `pending -> open` 14:10Z (PdM), `open -> planned` 14:25Z (pm-1), `planned -> integrated` 19:06Z (manager-2).
- That is too coarse to answer the questions. Nothing between `planned` and `integrated` is recorded: not when a worker started, when it handed off, time in `just check` or CI, time waiting for a merge slot, load holds, or retries. Holds and blocks carry no reason in the log.
- No report computes durations. `bridle report` (rcvb) lists what happened, not how long it took.

## The ask

1. **Record the lifecycle in enough detail**, with timestamps, to split a task's time into: waiting (unclaimed, on a dependency, on the human), agent work, builds and tests (worker's check, merge check, CI), waiting for a merge, and held or blocked, **with the reason** for each hold or block (load hold, budget, human gate, dependency, failed check, ...).
2. **A report** of cycle time from claim to merge (and from filing to merge), per task and summarised over a period, with the breakdown above, so changes to the lifecycle can be compared before and after.
3. Keep it where it can be evaluated later: the event log or the database, not only messages.

## Addendum (the human, 2026-10-09 ~3:20 PM ET)

> 3. how long do tickets wait for merging and release?
>
> etc.
>
> AND - DON'T Jump the queue just b/c I'm asking; if this doesn't exist, it's low priority; but something we should get on a workstream

So: also time waiting for a merge and for a release, and other waits like them ("etc."). **Priority: low.** Not to jump the queue; to go on a workstream.
