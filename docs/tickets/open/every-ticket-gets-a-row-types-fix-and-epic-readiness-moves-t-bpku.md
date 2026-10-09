---
id: bpku
title: Every ticket gets a row; types fix and epic; readiness moves to the ticket (ticket ready)
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: [syqn]
see: [22ab]
tasks: []
---

## The ask

Parent: br-22ab (step 2 of its plan; the human approved the plan 2026-10-09). Blocked by step 1 (field names). Read 22ab sections 2 (every ticket is indexed; its type decides its workflow) and 3 (readiness and who may start work).

Scope:

- Every ticket gets a row (the human: "row for every ticket is fine"): `bridle ticket new` makes the row, same ID; `bridle ticket task` and `ticket new --no-task` go away.
- Types: new `fix` (the change that fixes a `bug` report) and `epic` (a large initiative split into child tickets via `parent`, the human's choice); a change flag on `feature`, `fix`, `docs`, `chore`, `arch-revision`.
- Readiness moves to the ticket: `bridle ticket ready` replaces `task ready`, taken only with the human's approval; it refuses a ticket with an empty `## The ask` or one not committed on the local integration branch (k7tm's race fix moves here).
- No `idea` kind (the human, 2026-10-09): whether a ticket is scheduled is its state, not its kind.
- Migration: `bug` tasks that are fixes become `fix`.
- Docs in step: `docs/design/storage.md`, `docs/design/cli.md`, rule `tickets`, the roles that file tickets.

Out of scope: states beyond today's (stx8, reviewed separately); links (step 3); the thread (step 4).

Acceptance: `just check`.
