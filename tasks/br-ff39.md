+++
id = "br-ff39"
title = "Human to-dos: priority, rescind, full audit trail (ex9q follow-up)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T03:47:17.742Z"
updated_at = "2026-09-30T04:03:43.534720Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/todo-priority"
commit = "1224b2cb2ae52b5a28d2da4514fef608d660dccd"
summary = "Tasks have a priority (high/normal/low, default normal; TaskPriority in bridle-api types.rs), stored in the task file frontmatter (omitted when normal, so rebuild keeps it and old files load as normal). 'task new --priority' and 'bridle task priority <id> <p>' (POST /v1/tasks/{id}/priority) set it; a change writes a thread note (who/when) and a task.priority event. Human to-dos get a 'created' thread entry. task list/show display it and list sorts high first; the orchestrator startup steps point at 'task list --claimed-by human'. task drop on a human-claimed task sends the human a system inbox note with the reason, unless the human dropped it themself. Done already had thread+event. Priority is not set for agent work. Docs: cli.md, storage.md, coordination.md, api.md, CHANGELOG."
+++

Follow-up to human to-dos (br-c83e, landed). Spec: the last section of docs/questions/open/the-humans-to-do-list-and-restart-checklist-ex9q.md (the human's words: 'prioritized, able to be rescinded, with a full audit trail'). Do: (1) a priority on tasks (small set, e.g. high/normal/low; default normal; shown in bridle task list/show, the human's list sorted by it, and in the orchestrator startup list), set at 'bridle task new --for-human --priority' and changed with a command that records who and when in the thread and as an event; (2) rescind: whoever asked withdraws it with 'bridle task drop --reason' (exists): make sure the human sees it leave the list with the reason (an inbox note to the human on rescind of a human-assigned task); (3) audit: created, re-prioritized, rescinded, done are each in the thread and events with who and when. Reuse existing task fields/thread machinery; priority is stored in the task record and the state branch (check storage.md and rebuild from the state branch keeps it). API types in bridle-api/src/types.rs, clients and daemon together. Tests, docs (cli.md, storage.md, coordination.md), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: priority for agent work (the queue tiers rank that).

## Thread

### note · agent:todo-priority · 2026-09-30T04:03:20.190Z
done: task priority (high/normal/low) with set command, rescind inbox note to the human, audit in thread+events, survives rebuild; docs+CHANGELOG; just check green (801 tests), main merged; 38077cf

### note · agent:manager-2 · 2026-09-30T04:03:25.858Z
integrated: 1224b2cb2ae52b5a28d2da4514fef608d660dccd (branch bridle/todo-priority)

### note · agent:manager-2 · 2026-09-30T04:03:43.534Z
cleanup: removed agent todo-priority, branch bridle/todo-priority
