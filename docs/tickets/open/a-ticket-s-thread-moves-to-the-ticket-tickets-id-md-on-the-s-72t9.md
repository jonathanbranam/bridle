---
id: 72t9
title: A ticket's thread moves to the ticket (tickets/<id>.md on the state branch); ticket comment/ask/answer/show; thread migration
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: [3v75]
see: [22ab]
tasks: [br-72t9]
---

## The ask

Parent: br-22ab (step 4 of its plan; the human approved the plan 2026-10-09). Blocked by step 3. Read 22ab section 1 (the model: option A, the thread on the state branch, accepted by the human) and section 11 (migration, steps 2-4).

Scope:

- A ticket's thread lives in `tickets/<id>.md` on the state branch (the human accepted the split: "I don't _prefer_ the split, but it's an ok compromise for now and is less change/churn"); the row's body is dropped.
- `bridle ticket comment/ask/answer/show` replace the task forms; `ticket show` shows the file and the thread together.
- Migration: each task's thread moves to its ticket's thread file; tasks without a ticket (about 40 active) get a stub ticket from `ticket new --from-task`; finished ticketless tasks stay read-only history; tickets with more than one task get one child ticket per extra task, with `parent`.
- Docs in step: `docs/design/storage.md`, `docs/design/cli.md`, rule `talk-on-the-task`.

Out of scope: the user-facing rename of "task" (step 7) and the code rename (step 9).

Acceptance: `just check`; after the migration every active piece of work has a ticket, and its old thread shows on it.
