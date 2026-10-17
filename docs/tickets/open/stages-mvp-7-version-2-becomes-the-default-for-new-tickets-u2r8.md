---
id: br-u2r8
title: "Stages MVP 7: version 2 becomes the default for new tickets"
kind: feature
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: [br-ewk8]
related: []
parent: br-tj7q
theme: product-process
---

## The ask

Step 7 of 7, the last of the MVP of epic br-tj7q. The human, 2026-10-17: "We might need separate
tickets for building the version 2 stuff and making version 2 the default. I'd like to see it built
before it becomes the default" and "I would add to the 'when done' for that epic that version 2
becomes the default."

---

## Proposal

### Why

Once the human has tried a version 2 ticket and is satisfied, new tickets should get the new
workflow without anyone remembering a flag.

### What changes

`bridle ticket new` writes `version: 2` by default; `--version 1` still makes a version 1 ticket
(the escape hatch if the workflow breaks). Every role and rule that files tickets says to choose the
stages and reviews (`--stages`, `--review`) and to write or claim the proposal.

### What does not change

Existing version 1 tickets never migrate.

### Risk and impact

From this point a broken stage workflow slows new tickets; `--version 1` is the way around it.

### Out of scope

Retiring version 1.

### Open questions

None; starts only on the human's go after step 6.

---
