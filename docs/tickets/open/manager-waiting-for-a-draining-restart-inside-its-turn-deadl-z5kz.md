---
id: br-z5kz
title: Manager waiting for a draining restart inside its turn deadlocks the restart
kind: bug
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

What the title says; the details are in the sections below and the planner's brief on the
thread (`bridle ticket show`). Readied 2026-10-17 under the human's rule (br-mhjn): bugs, fixes
and chores go through without asking.

Seen 2026-10-14 ~02:59-03:54Z: upgrade e91faf95 drained ~55 min waiting on manager-2, which sat in one turn looping a foreground 'bridle agent wake --timeout 600' because it 'can't spawn bm7a while the daemon drains'. The drain waits for agents to leave their turn; the manager waited for the drain: deadlock until the orchestrator ran 'bridle agent interrupt manager-2'. Fix options: the drain wake/spawn refusal tells the agent to end its turn (the daemon resumes it after the restart); and/or the manager role/skill says so; and/or an agent blocked only in 'bridle agent wake' counts as quiet for the drain. Also seen: the daemon forwards its own wakes to itself as system@dalek mail (each wake arrives twice).
