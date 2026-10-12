---
id: br-b264
title: "A restart request sends the orchestrator a message"
kind: feature
created: 2026-10-12
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-up5r]
---

## The ask

The human, 2026-10-11 ~11:20 PM ET (to aide), verbatim: "ok, yeah, that is interesting and something that needs to be investigated; i'd suggest a message should go to the orchestrator when a restart is requested so that they are informed about the request".

Context (incident br-up5r): the human ran `bridle daemon restart` at ~9:45-9:56 PM ET on 2026-10-11. The daemon went into draining, waited on a phantom spawning agent (same failure as br-y455 / br-btdn), and held every message to agents for ~75 min, which stalled the br-2wnn land and v0.6.3. Nothing recorded who asked for the restart, and the orchestrator found the stuck drain only by investigating the stall.

The human's proposal: when a restart is requested (by anyone), the orchestrator gets a message saying so.

## Notes

- Aide's addition, not the human's: the message could name who asked and, if the drain hasn't finished after a while, what it is still waiting on (so a stuck drain shows up in minutes, not when work stalls). The human asked only for the notification.
- The investigation the human mentions (why the drain stuck) is br-up5r / br-btdn.
