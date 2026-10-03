+++
id = "br-eb50"
title = "Task watchers 2/3: a watchers list on tasks (no delivery change)"
kind = "feature"
state = "planned"
created_at = "2026-10-03T02:07:11.774Z"
updated_at = "2026-10-03T02:30:25.960570Z"
size = "M"
+++

Ticket: docs/tickets/open/task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq.md (decided items 2 and 5; the notification design changed 2026-10-03: task changes will be MESSAGES, built in the next task br-7605; this task is the list only). Code: the task store/API (crates/bridle-api types.rs), CLI in crates/bridle, docs/design/storage.md (task records, state branch, rebuild).
Goal: (1) a watchers list on the task (principals), also in the task record on the state branch (rebuild restores it; old records without it parse as empty). (2) The creator is added at creation (created_by from br-c078); claiming a task adds the claimer as a watcher (removable). (3) 'bridle task watch <id>' and 'bridle task unwatch <id>' add/remove the CALLING principal; matching API endpoints; recorded in the thread and as an event. 'task show' and JSON list watchers. (4) One function in the daemon, 'who is notified of this task', that br-7605 will use. (5) Backfill existing tasks: watchers = created_by plus the current claimer; automatic, idempotent, never blocks daemon start-up (a failure logs and leaves the list empty). DO NOT change wakes or add any delivery (br-7605 does that and deletes the old 'created or claimed' scan in principal_wake.rs). Tests: creator watches at creation; claim adds the claimer; watch/unwatch work and are recorded; old tasks backfilled; rebuild keeps watchers. Docs: storage.md, api.md, cli.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: the automatic backfill above. Depends on br-c078.

## Thread

### note · agent:pm-1 · 2026-10-03T02:20:15.613Z
PM 2026-10-03: keep DELIVERY neutral (xxxq advisor correction 2026515; wake vs message is open, br-5e4a). Build the watchers list, watch/unwatch, auto-add, backfill, and one function that answers 'who watches this task / who is notified of this event' that the wake code calls; do not change what a wake carries and do not add a delivery mechanism, so delivery can switch to messages later without redoing this.
