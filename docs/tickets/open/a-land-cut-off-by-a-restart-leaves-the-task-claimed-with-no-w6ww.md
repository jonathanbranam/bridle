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

br-e7e2 landed as 9e6a164b (on main; branch bridle/we7e2 identical to main), but a daemon restart (self-upgrade, 2026-10-11 ~14:55Z) cut `bridle task land` off partway. The task row still says `claimed`, but its claims row is gone. Now every way out refuses it:

- `task done br-e7e2 --commit 9e6a164b` and `task land`: "task br-e7e2 is not claimed" (store.rs delete_claim: no claims row)
- `task claim`: "not ready to claim" (state is claimed)
- `task release`: "not claimed" (tasks.rs release_task checks the in-memory claims map)
- `task reopen`: only for dropped or integrated

Cause (read from the code): done/release go through `clear_claim` (tasks.rs: delete the claims row, then the in-memory entry) and only afterwards `set_task_state`. They are two separate store writes, not one transaction, so a stop between them leaves state=claimed with no claim.

Fix: make the claim delete and the state change one transaction; and let `task done` (or a startup repair) accept a task that is `claimed` with no claims row, so a row already stuck can be closed. Then close br-e7e2 as integrated at 9e6a164b.

Worst if not done: the row sits in the queue's Claimed list forever and any future interrupted land sticks the same way; nothing else is blocked. Not critical.

Found by manager-2 (thread on br-e7e2) and external:orchestrator, 2026-10-11.

## Update 2026-10-11 18:40Z (orchestrator): every restart drops agents' claims; critical

Not just an interrupted land. br-e9wg's land at ~18:30Z ran with no restart during it: the merge
went in (356bc398, pushed), then `done` failed "task br-e9wg is not claimed". br-dd79, claimed by
wdd79 since 17:20Z, has no claims row either. After the 18:14Z self-upgrade restart the claims
table (read-only sqlite) holds only the human's to-do claims; every agent claim is gone, while the
tasks still read `claimed`.

Likely cause: start-up's stale-claim sweep (tasks.rs ~245, "A claim on a task that isn't `claimed`
... drop the row") runs against a cache where an agent-claimed task doesn't (yet) read `claimed`
(perhaps its state-branch file still says `planned`; the human's to-dos are created claimed and
survive). The worker confirms the real cause with a test: claim, restart (reload), done must work.

So every task an agent holds across a restart (self-upgrade every 3 h) lands but can't be closed.
Critical (the human, 2026-10-09: "something broken" is critical).

Fix, in order:
1. A claim survives a restart (the sweep or the task-state load, whichever is wrong).
2. Make claim delete + state change one transaction (the original ask above).
3. Repair at start-up: a task `claimed` with no claim row and its code on main can be closed with
   `task done --commit`; or `task done` accepts a claimed task with no claims row. Then close
   br-e7e2 (9e6a164b), br-e9wg (356bc398) and br-dd79 (when it lands).

Acceptance: a test that claims, reloads the TicketManager, and lands/dones; just check green.
Model: Sonnet.
