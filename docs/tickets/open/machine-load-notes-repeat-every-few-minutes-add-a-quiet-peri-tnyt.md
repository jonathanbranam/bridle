---
id: tnyt
title: "Machine load notes repeat every few minutes: add a quiet period, and send one note per machine, not one per daemon"
kind: bug
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [d48r, v7ug]
tasks: [br-tnyt]
---

## The ask

The human, 2026-10-09 ~6:15 PM ET, verbatim (to the orchestrator, for the aide):

> tell aide to file a ticket to deal with that; seems like there isn't a quiet period for those notifications; they shouldn't come over and over and over.

## The facts (the orchestrator, 2026-10-09)

- `crates/bridle-daemon/src/load.rs` sends one note ("Machine load is high: ... Don't add work:
  wait.") to `external:orchestrator` on every upward crossing of `[machine] load_per_core` (2.5).
  It samples every 30 s with no hysteresis and no minimum gap, so while builds hold the load near
  the threshold it fires every 1-2 minutes.
- Every daemon on the machine (on dalek: bridle, bridle-ui, track-web) sends its own copy of the
  same machine-wide note. On the evening of 2026-10-09: ~10 notes in 10 minutes across the three.
- Background: [[machine-load-notes-go-to-aide-not-the-orchestrator-d48r|d48r]] (withdrawn; the
  recipient stays the orchestrator).

## The ask, as understood (aide)

1. A quiet period: once a note is sent, no new note until the load has stayed below the threshold
   for a while (hysteresis), and at most one note per interval in any case. The values are config,
   with defaults the design picks.
2. One note per machine, not per daemon: a machine-wide condition is reported once.
3. Unchanged: the recipient and what the daemon does under load (hold new spawns and resume them
   itself).
