+++
id = "br-ff39"
title = "Human to-dos: priority, rescind, full audit trail (ex9q follow-up)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T03:47:17.742Z"
updated_at = "2026-09-30T03:47:22.806359Z"
+++

Follow-up to human to-dos (br-c83e, landed). Spec: the last section of docs/questions/open/the-humans-to-do-list-and-restart-checklist-ex9q.md (the human's words: 'prioritized, able to be rescinded, with a full audit trail'). Do: (1) a priority on tasks (small set, e.g. high/normal/low; default normal; shown in bridle task list/show, the human's list sorted by it, and in the orchestrator startup list), set at 'bridle task new --for-human --priority' and changed with a command that records who and when in the thread and as an event; (2) rescind: whoever asked withdraws it with 'bridle task drop --reason' (exists): make sure the human sees it leave the list with the reason (an inbox note to the human on rescind of a human-assigned task); (3) audit: created, re-prioritized, rescinded, done are each in the thread and events with who and when. Reuse existing task fields/thread machinery; priority is stored in the task record and the state branch (check storage.md and rebuild from the state branch keeps it). API types in bridle-api/src/types.rs, clients and daemon together. Tests, docs (cli.md, storage.md, coordination.md), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: priority for agent work (the queue tiers rank that).
