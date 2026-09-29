---
id: hx7t
title: Routine notes still reach the human's inbox
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [kp3f, d99e]
---

## What happened

The human's inbox is only for what they must act on (kp3f). On 2026-09-29 it held:

- `done:` reports from one-off workers (`fix-fmt` m-1552, `fix-changelog` m-1657),
  addressed to `human`, not to the manager that spawned them;
- a note from pm-1 with an **empty body** (m-1495);
- a stale `question` (m-1348) answered by later work, never closed.

The orchestrator can't mark them read (only the recipient can), so the human has to.

## The ask

1. A worker's `done:`/progress report goes to its spawner (or the manager), never to
   `human`. The simplest fix may be in the worker skill and br-d99e's stop-check, which
   now insists on a `done:` report: check who it tells the worker to send it to.
2. `bridle send` (and the API) refuses an empty body.
3. Optional, only if cheap: the orchestrator can mark a message to `human` read, since
   it's the human's delegate. Otherwise the human clears these by hand.
