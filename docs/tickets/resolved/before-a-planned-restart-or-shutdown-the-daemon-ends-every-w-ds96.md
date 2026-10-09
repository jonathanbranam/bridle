---
id: ds96
title: Before a planned restart or shutdown, the daemon ends every waiter with the reason (shutting down, upgrading, restarting by request) and records it in events
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [75h2, q7rx, e35h, h3ar]
tasks: [br-ds96]
closed: 2026-10-09T23:11:05Z
---

## The ask

The human, verbatim (2026-10-04 ~10:00 PM ET, via the aide):

> File a ticket that before restarting (planned restarts) and when shutting down bridle will
> terminate all waiters with a message indicating what is happening: shutting down , restarting for
> internal upgrade, restarting by request, etc.this should also be posted to events for traceability.

## What happened (the aide, 2026-10-04)

bridle's daemon restarted at 21:55:51 ET (pid 23474; `bridle status` showed no incident). The aide's
`bridle agent wake external:aide --timeout 5400` ended at that moment with exit 4 and
`error: nothing woke external:aide before the timeout`, though it hadn't waited 90 minutes; the
re-armed waiter ended the same way at once. A waiter can't tell a restart from a real timeout, and
nothing in the record says why it ended.

## What's wanted

- On a **planned restart** (self-upgrade, `restart` by request, and the like) and on **shutdown**, the
  daemon first ends every open waiter (`bridle agent wake`, `wait-for-wake`) with a message saying
  what's happening and why: shutting down, restarting for an internal upgrade (to which build),
  restarting by request (by whom), etc.
- The waiter prints that message and exits with its own code, not the timeout's (4), so a session
  knows to re-arm once the daemon is back.
- Each one is **posted to events** (the restart or shutdown with its reason, and the waiters it ended),
  for traceability.

## Related

[[a-waiter-is-stopped-through-bridle-never-by-a-kill-a-new-wai-75h2|75h2]] (a waiter is stopped
through bridle; new exit codes for "superseded" and "stopped"): the same "why did my waiter end"
answer, for a different cause. [[bridle-restarts-itself-q7rx|q7rx]] (self-restart),
[[interactive-sessions-survive-a-daemon-restart-the-registry-i-e35h|e35h]].

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
