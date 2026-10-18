---
id: br-ewhy
title: "Stages MVP 4: bridle spawns a designer for each unblocked stage; timer to the product manager"
kind: feature
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: [br-quvh, br-ussh]
related: []
parent: br-tj7q
theme: product-process
resolved: 2026-10-18T05:10:16Z
---

## The ask

Step 4 of 7 of the MVP of epic br-tj7q. The human, 2026-10-17: "If a ticket exists and it's ready to
move forward, then the system, Bridle, should spin up a background agent"; "all the artifacts, all
the stages are written by a background agent. That's MVP"; "Just send it straight back to the PDM if
the agent doesn't finish it. No reason to try again"; and on the timer: "If it's not been done for
an hour or 30 minutes, then set a message to the PDM".

---

## Proposal

### Why

Without a human gate, a version 2 ticket should go from filed to prepared with no agent or human
asking for it.

### What changes

- The daemon watches version 2 tickets. When a ticket's next stage is unblocked and the ticket is
  unclaimed, it spawns a `designer` whose prompt is `bridle ticket next <id> --json` (step 2) and,
  for a redo after `changes`, the human's comments with "the human added these comments; please
  address them". The designer claims the ticket, writes, runs `ticket stage-done` (step 3), ends.
- Then the daemon works out what is next: another writer, the human's claim for a signature, or
  ready.
- A stage not done 60 minutes after its writer started (`[stages] timeout`, one setting), or whose
  writer ended without `stage-done`, sends one message to `external:advisor/product-manager`
  ("br-oooo: design not done by designer-3, timed out"); no retry; the PdM may `ticket restage`.
- The `designer` role (`workflow/base/roles/designer.md` and its config prompt) widens from "a
  proposal into the ticket" to writing any stage it is given; still builds nothing. The stage
  rules (step 2) carry the specifics.
- Spawns go through the existing supervisor path, its caps and the budget governor.

### What does not change

Version 1 tickets; the manager's planning and the worker queue.

### Risk and impact

The first time the daemon spawns agents by itself for ticket work: watch cost; the caps bound it.

### Out of scope

Interactive writers (V1); a designer sending a stage back as blocked (V1).

### Open questions

None.

---
