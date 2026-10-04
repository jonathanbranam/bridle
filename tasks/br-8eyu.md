+++
id = "br-8eyu"
title = "Every task starts pending; 'bridle task ready <id>' opens it (k7tm A)"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T23:28:37.330Z"
updated_at = "2026-10-04T00:06:21.613407Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/task-pending"
commit = "07f7a54788624277f6ecff4bac3e305fd68d53e0"
summary = "New task state pending: every task is created pending (store insert, TaskState::Pending); 'bridle task ready <id>' (it already meant the queue's top tier, so ReadyArgs takes an optional id; POST /v1/tasks/{id}/ready, TaskManager::ready_task) opens it. Refused for visitors, worker and product-manager agents; incident rules apply. PM wake (note_task_filed) moved from creation to ready; --for-human to-dos skip the gate; kind and impact may be set while pending. Status gains pending_tasks (shown by 'bridle status'); orchestrator startup steps and orchestrator/advisor/manager/PM role text updated; docs and CHANGELOG. Tests: new pending test, existing tests use a ClientExt::new_open_task helper / TaskManager::new_open_task. Caveat: my global git config has merge.ff=only, which fails land_test and tools_only_test on any branch; check run with GIT_CONFIG_GLOBAL=/dev/null."
+++

Human-approved (2026-10-03, relayed by advisor 'tickets', m-4116). The human, verbatim: "all tasks starting with pending is a good move" and, on managers readying their own small fixes, "Yes, approved."
Ticket: docs/tickets/open/tickets-and-tasks-why-both-k7tm.md, section "Decided: every task starts pending" (decisions 11-14) and "Slicing" A. Read the whole ticket.
Goal: new task state pending before open (pending -> open -> planned -> claimed -> integrated). Every task is created pending; 'bridle task ready <id>' is the deliberate ready step, by the orchestrator or an advisor with the human's approval; the orchestrator readies its own critical fixes; a manager may ready its own small fix inside already-approved work (flake, merge fix, branch bug), never new features. The PM plans only open tasks (queue, PM wakes). Pending tasks surface in 'bridle status' and the orchestrator's startup steps (bridle prime orchestrator). Existing open tasks stay open: no migration.
Scope: store, API types (types.rs, all clients), CLI (task ready, task new default), daemon queue and PM wakes, status, docs (storage.md, coordination docs, cli.md), roles (orchestrator, advisor, manager, PM). Note br-6c69's 'bridle ticket task <id>' creates a task too: it must also start pending.
Acceptance: just check passes. Out of scope: k7tm B (ticket from task).

## Thread

### note · agent:pm-1 · 2026-10-03T23:28:49.575Z
Planned, tier 1, side by side with br-8e5v. Model: Sonnet. Both touch role/rule text (orchestrator/advisor/manager/PM vs tickets rule, worker/manager/PM): whichever lands second rebases and merges the role text by hand. Migration: none (existing open tasks stay open). Also check br-6c69's 'bridle ticket task' starts pending if it has landed.

### note · agent:task-pending · 2026-10-04T00:04:22.165Z
done: every task starts pending, 'bridle task ready <id>' opens it (just check green, 1114 tests, run with GIT_CONFIG_GLOBAL=/dev/null since the host's global merge.ff=only breaks land_test/tools_only_test on any branch); 2354795

### note · agent:manager-2 · 2026-10-04T00:04:30.140Z
integrated: 07f7a54788624277f6ecff4bac3e305fd68d53e0 (branch bridle/task-pending)

### note · agent:manager-2 · 2026-10-04T00:06:21.613Z
cleanup: removed agent task-pending, branch bridle/task-pending
