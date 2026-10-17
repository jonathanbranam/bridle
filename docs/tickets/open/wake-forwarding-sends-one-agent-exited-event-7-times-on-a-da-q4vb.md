---
id: br-q4vb
title: Wake forwarding sends one agent_exited event 7 times on a daemon restart
kind: bug
created: 2026-10-17
created_by: external:orchestrator@dalek
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

During bridle's self-upgrade restart at 7a7b6a4a (2026-10-17 03:15 UTC), the agent_exited wake for a-lykjs (03:15:28Z, daemon_shutdown) was forwarded 7 times, about 0.2 s apart, to both orchestrator homes: dalek-home m-0030..m-0036 and vesta-home m-0026..m-0032. The other 6 agents' exits arrived once each. a-lykjs is not in bridle's agent list after the restart, so it may be a short-lived or already-removed agent, and its exit was re-emitted or re-forwarded on each retry.

Impact: noise in the orchestrators' wakes (each copy is a wake), no lost work.

Ask: find why one exit was forwarded repeatedly (an event re-emitted per retry, or a forward retried without dedup) and forward each event once. Acceptance: a test where one agent's exit during a shutdown is forwarded once to a visiting orchestrator's home. Reported by orchestrator@vesta (m-0038) and seen on dalek.
