---
id: br-ussh
title: "Stages MVP 3: stage-done, approve (signed and traced), changes, restage; ready means prepared"
kind: feature
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: [br-9mqh]
related: []
parent: br-tj7q
theme: product-process
resolved: 2026-10-18T03:10:48Z
---

## The ask

Step 3 of 7 of the MVP of epic br-tj7q. The human, 2026-10-17: "whoever owns the gate, will sign it
as approved, and then the rest of the command can't go forward unless it's been signed"; "I want to
know where that came from"; "Every watcher is informed on every stage change"; and ready means
prepared: "Agents should fill out everything, and then the ticket's ready."

---

## Proposal

### Why

Stages need to be marked written, signed by their gate's owner, sent back for changes, and the
ticket must become ready by itself when prepared, with a trace of who did what.

### What changes

- `bridle ticket stage-done <id> <stage>`: checks the file exists (proposal: the delimited section
  is present and non-empty), records it written; releases the writer's claim.
- `bridle ticket approve <id> <stage>`: signs a stage listed for review. Records signer (the
  human), who ran it (the caller's principal from its token: the human, or "human via <agent>")
  and the time, as a thread entry and on the ticket's signed list. UI and CLI are both the human.
- `bridle ticket changes <id> <stage> [text]`: sends a stage back (it becomes unwritten and
  unclaimed; the text goes on the thread as the human's comments for the next writer).
- `bridle ticket restage <id> <stage>`: the product manager puts a failed or timed-out stage back
  for bridle (unclaimed, unwritten).
- While a written stage waits for the human's signature, the ticket is claimed by `human`.
- Ready means prepared: when every listed stage is written and every gate signed, the daemon moves
  a version 2 ticket `pending` -> `open` itself, recorded as by bridle. Before that, `ticket ready`
  refuses a version 2 ticket. `plan` and the rest are as today (when it is built is the PM's call).
- Watchers (`ticket watch`) are told on every stage change: written, waiting for signature, signed,
  sent back, timed out.
- Editing an approved stage does not withdraw its signature (the human, 2026-10-17: "No").

### What does not change

Version 1 tickets and `ticket ready` for them; cryptographic signing (v2va).

### Risk and impact

Adds states to the ticket index; version 1 must be untouched (tests).

### Out of scope

Spawning writers and the timer (step 4); buttons in the UI (V1).

### Open questions

None.

---
