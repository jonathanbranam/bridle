---
id: br-d7r5
title: Self-upgrade restarts in the middle of a task land's background check, so the land is lost and redone
kind: bug
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-w6ww]
resolved: 2026-10-16T10:02:58Z
---

## The ask

The self-upgrade's drain waits for agent turns to end (daemon.md, 'Automatic upgrade': "A turn is never cut off"). But `bridle task land` runs its own `just check` in the background when main has moved, after the manager's turn has ended. The drain doesn't see that check, so the restart kills it.

Seen twice on 2026-10-11:
- ~14:55Z: br-e7e2's land was cut off after the merge but before the task was marked done (left the row stuck; br-w6ww).
- 18:13Z (upgrade to ffb86340): br-e9wg's land was cut off during its check. Nothing merged; manager-2 had the worker re-merge main and re-run just check.

Each costs a full `just check` (30-40 min of machine time, rule planning-the-queue) plus a worker round trip, and can leave state half-written.

Fix: count a land in progress (its check, merge, push and done) as busy for the drain, the same as a turn, so the restart waits for it. The land is bounded, so the drain still ends.

Worst if not done: every self-upgrade (every 3 h) that lands during a land repeats a 30-40 min check, and can strand a task like e7e2. Not critical: nothing breaks, it wastes time.

Filed by external:orchestrator, 2026-10-11.
