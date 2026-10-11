---
id: br-w6ww
title: "A land cut off by a restart leaves the task claimed with no claim row: done, land, claim and release all refuse it"
kind: bug
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

## The ask

br-e7e2 landed as 9e6a164b (on main; branch bridle/we7e2 identical to main), but a daemon restart (self-upgrade, 2026-10-11 ~14:55Z) cut `bridle task land` off partway. The task row still says `claimed`, but its claims row is gone. Now every way out refuses it:

- `task done br-e7e2 --commit 9e6a164b` and `task land`: "task br-e7e2 is not claimed" (store.rs delete_claim: no claims row)
- `task claim`: "not ready to claim" (state is claimed)
- `task release`: "not claimed" (tasks.rs release_task checks the in-memory claims map)
- `task reopen`: only for dropped or integrated

Cause (read from the code): done/release go through `clear_claim` (tasks.rs: delete the claims row, then the in-memory entry) and only afterwards `set_task_state`. They are two separate store writes, not one transaction, so a stop between them leaves state=claimed with no claim.

Fix: make the claim delete and the state change one transaction; and let `task done` (or a startup repair) accept a task that is `claimed` with no claims row, so a row already stuck can be closed. Then close br-e7e2 as integrated at 9e6a164b.

Worst if not done: the row sits in the queue's Claimed list forever and any future interrupted land sticks the same way; nothing else is blocked. Not critical.

Found by manager-2 (thread on br-e7e2) and external:orchestrator, 2026-10-11.
