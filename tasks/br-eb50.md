+++
id = "br-eb50"
title = "Task watchers 3/4: a watchers list on tasks; wakes go to watchers"
kind = "feature"
state = "open"
created_at = "2026-10-03T02:07:11.774Z"
updated_at = "2026-10-03T02:07:11.774Z"
size = "M"
+++

Ticket: docs/tickets/open/task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq.md (decided item 2, and item 5). Code: crates/bridle-daemon/src/principal_wake.rs (task_reasons: today 'created or claimed', with a 1M-event scan), the task store/API (crates/bridle-api types.rs), CLI in crates/bridle, docs/design/storage.md (task records, state branch, rebuild).
Goal: (1) a watchers list on the task (principals), in the task record on the state branch too (rebuild restores it; old records without it parse as empty). (2) The creator is added at creation (uses created_by from task 2); claiming a task adds the claimer as a watcher (removable). (3) 'bridle task watch <id>' and 'bridle task unwatch <id>' add/remove the CALLING principal; the API has the matching endpoints; recorded in the thread/event so it is auditable. 'task show' and JSON list watchers. (4) principal_wake task_reasons uses the watchers list: a principal wakes for changes to tasks it watches, never its own changes; the 'created or claimed' scan and the 1,000,000-event read are deleted. (5) Backfill: existing tasks get watchers = created_by plus the current claimer (automatic, idempotent, never blocks daemon start-up; a failure logs and leaves the list empty). Tests: creator watches at creation; claim adds the claimer; unwatch stops wakes; watch starts them; own changes never wake; old tasks backfilled; rebuild keeps watchers. Docs: storage.md, api.md, cli.md, messages.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: the automatic backfill above. Out of scope: what a wake says (task 4), the human as watcher (9nrt), wake-vs-message. Depends on tasks 1 and 2 (br-a34f, created_by task).
