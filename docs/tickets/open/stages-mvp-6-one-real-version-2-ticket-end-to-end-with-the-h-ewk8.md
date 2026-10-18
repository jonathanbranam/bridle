---
id: br-ewk8
title: "Stages MVP 6: one real version 2 ticket end to end, with the human"
kind: chore
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: [br-ewhy, br-6c8x]
related: []
parent: br-tj7q
theme: product-process
---

## The ask

Step 6 of 7 of the MVP of epic br-tj7q. The human, 2026-10-17: "I do want to try a version 2 ticket"
and "I'd like to see it built before it becomes the default". The epic's done-when: proven on one
real ticket end to end.

---

## Proposal

### Why

The MVP is proven only when one real ticket goes from filed to landed through the stages.

### What changes

With the human: file one real ticket as version 2 (the human picks it, or a small real fix from the
queue), with the proposal and specs listed for review. Watch it: designer spawns, the human signs
through an advisor, ready by itself, the PM plans it, a worker builds from tasks.md, specs merged.
Record what broke as tickets (version 1, so a broken workflow cannot block their fixes).

### What does not change

Nothing in code, unless a fix comes out of it.

### Risk and impact

None; one ticket.

### Out of scope

Making version 2 the default (step 7).

### Open questions

Which ticket: the human's choice when steps 1-5 land.

---

## The try (2026-10-18)

The ticket is br-2ae8 (replaces br-4ge4). The human, to advisor tj7q: "I want review each step of the test ticket. Otherwise keep all the other tickets moving. fix things the right way. That's the point of the manual walkthrough". So every stage of 2ae8 needs the human's signature (commit c3bd2bfc). What broke so far, each filed as a version 1 ticket and fixed properly, not worked around: br-upxq (a stage must be on main before review; the human's design), br-fyww (landing a stage branch marked 2ae8 integrated; the human's design), br-65qh (a worker lost its claim; cause br-4n3v, which the daemon only ran from 20:54Z).
