+++
id = "br-w6ww"
title = "A land cut off by a restart leaves the task claimed with no claim row: done, land, claim and release all refuse it"
kind = "bug"
state = "planned"
created_at = "2026-10-11T17:20:34.913Z"
updated_at = "2026-10-11T18:35:37.808072Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "critical"
priority_at = "2026-10-11T18:35:05.073669Z"
ticket = "w6ww"
+++

Ticket: docs/tickets/open/a-land-cut-off-by-a-restart-leaves-the-task-claimed-with-no-w6ww.md (read all of it, including "Update 2026-10-11 18:40Z"; that update is the real scope). CRITICAL: every daemon restart (self-upgrade every ~3 h) drops agents' claim rows, so tasks that land cannot be closed (br-e7e2, br-e9wg stuck; br-dd79 will be). Start at once, ahead of everything.

Do in order, each with a test:
1. A claim survives a restart. Reproduce first with a test: claim a task, reload the TicketManager (same path as daemon start-up), then done/land must work. Find the real cause: the start-up stale-claim sweep (crates/bridle-daemon/src/tasks.rs ~245, "A claim on a task that isn't claimed ... drop the row") probably runs against a cache where an agent-claimed task does not read `claimed` yet (check the state branch file versus the SQLite row; the human's to-dos are created claimed and survive). Fix the sweep or the state load, whichever is wrong; say which on the task thread.
2. Make the claim delete and the state change one transaction (clear_claim then set_task_state are two writes today; crates/bridle-daemon/src/tasks.rs and store.rs).
3. Repair: a task `claimed` with no claims row can be closed by `task done --commit SHA` (or a start-up repair), so rows already stuck can be closed. Then, as the last step, close br-e7e2 (commit 9e6a164b) and br-e9wg (356bc398) with that command and say so in the done note (br-dd79 when it lands). Do not touch other tasks.
Files: crates/bridle-daemon/src/tasks.rs, store.rs, tests (crates/bridle-daemon/tests), docs/design/agent-host/ (claims and start-up, storage.md if schema/semantics change), CHANGELOG.
Migration: no schema change expected; if one is needed, it goes through the daemon's normal migration; state it in the done note. Acceptance: just check passes; tests: claim, reload, done works; an interrupted done leaves either both changes or neither; a claimed task with no claims row can be closed by done --commit. Model: Sonnet. Out of scope: the self-upgrade drain (br-d7r5), other claim behaviour.

## Thread

### note · external:orchestrator · 2026-10-11T18:35:05.073Z
priority: normal -> critical

### note · external:orchestrator · 2026-10-11T18:35:28.262Z
From orchestrator: br-w6ww is now CRITICAL and ready (my call, the human's 'something broken' rule). Not just interrupted lands: every daemon restart drops the claim row of every agent-claimed task (claims table after the 18:14Z upgrade holds only the human's to-dos). br-e9wg landed and stuck; br-dd79 will too. Please plan it now and put it in a tier ahead of everything; brief is in the ticket (update section). Sonnet.
