---
id: ghys
title: A project's aide messages an orchestrator that isn't watching that project's daemon; the roles don't say the orchestrator is per machine
kind: bug
opened: 2026-10-08
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [ma8e, hj4g, s5ah, kuw2, r8kv]
tasks: [br-ghys]
closed: 2026-10-09T23:11:05Z
---

## The ask

The human, 2026-10-07 ~10:10 PM ET, via bridle's aide: "the bridle-ui aide has been messaging a non-existent bridle-ui orchestrator; file this as a bug (not incident) and investigate the role rules; this again is a project vs. machine level issue; but the roles need to be clarified."

## What happened (checked on bridle-ui's daemon, aide's token)

bridle-ui's aide sends to `external:orchestrator` on bridle-ui's daemon, as its role says. The one orchestrator (bridle's session) reads bridle-ui only while it runs a waiter there, and often didn't:

- m-0445 (11:32 PM ET 10-05, "manager-1 looks stuck") and m-0448 (1:02 AM, "still stalled, no reply yet") were read at 21:29Z 10-06: about 18 hours later.
- m-0606 (the human's ~6:40 PM ET 10-07 answers: ui-2kmw, ui-m2pz) was read at 01:02Z: about 2.5 hours later. The orchestrator: "Sorry for the delay: I wasn't running a waiter on bridle-ui's daemon."
- bridle-ui's daemon sent the human no "no waiter" note in that time (no `system` messages to `human` there).

## The role rules (investigated)

- `workflow/base/roles/aide.md`: "One aide session runs per project"; relay with `bridle send external:orchestrator ...`. It doesn't say the orchestrator is one per machine, living in another project's session, nor how to tell whether it is watching this daemon.
- `workflow/base/roles/orchestrator.md` (Watch): "One waiter watches one daemon. Run one per project you hold an orchestrator token for ... or that project's messages to you are never seen (the human, 2026-10-01)." So the duty is on the orchestrator alone, as a rule to remember; nothing checks it. It holds tokens for bridle, bridle-ui, meta-notes and track-web on dalek.
- The working rule is "one orchestrator per box" (ticket [[one-orchestrator-and-advisor-or-one-per-project-ma8e|ma8e]], the human 2026-09-30), but the identity `external:orchestrator` exists on every project's daemon, so a message to it is accepted whether or not anyone is listening.

## The ask

1. **Clarify the roles**: aide (per project) and orchestrator (per machine) say plainly who is where, and how a project aide reaches the machine's orchestrator.
2. **Make a missed route visible**: a message to `external:orchestrator` on a daemon with no orchestrator waiter should not sit silently for hours; e.g. the daemon's existing "no waiter for 15 minutes" note reaches that project's aide, or the sender is told at send time.
3. Fits the machine-level work in [[a-machine-daemon-one-per-machine-doing-machine-wide-work-onc-kuw2|kuw2]] (one orchestrator watching one machine daemon would remove the per-project waiter duty); a role clarification and the visibility fix needn't wait for it.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
