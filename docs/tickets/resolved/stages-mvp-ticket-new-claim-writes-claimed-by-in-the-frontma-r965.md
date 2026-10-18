---
id: br-r965
title: "Stages MVP: ticket new --claim writes claimed_by in the frontmatter, not a row claim (pending v2 tickets can't be row-claimed)"
kind: fix
created: 2026-10-18
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-9mqh, br-ussh]
parent: br-tj7q
resolved: 2026-10-18T06:33:54Z
---

## The ask

Found by wussh building Stages MVP step 3 (br-ussh), raised by the orchestrator 2026-10-17 ~11:10 PM
ET. Decided by advisor product-manager under the human's overnight mandate (br-tj7q thread,
~10:30 PM ET: "Make decisions as necessary").

---

## Proposal

### Why

`bridle ticket new --version 2 --claim` (step 1, br-9mqh) claims the new ticket's row
(`claim_task`, crates/bridle/src/ticket.rs ~line 247). A row claim needs the task `planned`, but a
version 2 ticket stays `pending` while its stages are prepared (docs/design/stages.md). So `--claim`
fails after the file is written, and the filer is never recorded as the proposal's writer. Step 3
(br-ussh) already records "claimed by" as a `claimed_by:` frontmatter line for the same reason.

### What changes

- `ticket new --version 2 --claim` writes `claimed_by: <filer>` into the new ticket's frontmatter
  (the same line and helper step 3 uses, crates/bridle/src/stage_cmds.rs), and makes no row claim.
- `stage-done` on the proposal ends it as step 3 already does; check that path covers a filer's
  claim, not only `human`.
- A test: `ticket new --version 2 --claim` against the test daemon succeeds and the file has the
  line; the row stays `pending` and unclaimed.
- docs/design/stages.md and cli.md: say `--claim` is the frontmatter line. CHANGELOG line.

### What does not change

Version 1 tickets and row claims for planned tasks.

### Out of scope

Anything else in steps 4-5.

### Open questions

None.

---
