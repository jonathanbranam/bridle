---
id: br-6c8x
title: "Stages MVP 5: a worker builds a version 2 ticket from tasks.md (OpenSpec apply)"
kind: feature
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle]
changes: []
specs: []
blocked_by: [br-quvh]
related: []
parent: br-tj7q
theme: product-process
resolved: 2026-10-18T06:16:13Z
---

## The ask

Step 5 of 7 of the MVP of epic br-tj7q. The human, 2026-10-17: one worker builds the whole ticket
from tasks.md, like OpenSpec's apply: "We should liberally steal and copy the OpenSpec apply skill".

---

## Proposal

### Why

A prepared version 2 ticket must reach a worker with its design, specs and task list, and the specs
must end in the project's bridle specs.

### What changes

- A worker on a version 2 ticket is told to read the proposal, design.md and specs.md, and to work
  through tasks.md in order, ticking each item (`- [x]`) as it is done, pausing on a blocker
  (copied from OpenSpec's apply skill, `/Volumes/Data/work/bridle/OpenSpec/skills/
  openspec-apply-change/SKILL.md`). The first task merges specs.md into `design/specs/`.
- The land check fails a version 2 ticket whose tasks.md has unticked items.

### What does not change

Workers on version 1 tickets; landing otherwise.

### Risk and impact

Low; prompt text and one land check.

### Out of scope

Rules injected into the apply step (V2).

### Open questions

None.

---
