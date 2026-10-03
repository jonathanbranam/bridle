+++
id = "br-6fd7"
title = "Reports go to the spawner not the human; bridle send refuses an empty body (hx7t)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T12:04:21.152Z"
updated_at = "2026-09-29T12:19:23.111852Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/send-empty"
commit = "47ec3b5"
summary = "bridle send, task note (CLI require_body in commands.rs) and both daemon send routes (server.rs require_body, 400) refuse empty/whitespace bodies. Worker skill, worker role and the stop-check unreported-finish reason now say to report to <manager> (spawner or manager agent, never human) instead of <sender>. Tests: CLI require_body unit, daemon empty-body integration, stop-check reason. docs/design/cli.md and CHANGELOG updated."
+++

Ticket: docs/questions/open/*hx7t.md (read it). Goal, two parts. 1) A worker's done:/progress report must go to its spawner or manager, never to human: check who workflow/base/skills/worker/SKILL.md and the stop-check block message from br-d99e (crates/bridle/src/stop_check.rs) tell a one-off worker to send to (the <sender> placeholder resolves to human for workers spawned by the human or orchestrator); make the default the spawning principal or the manager, and fix the skill text and the block reason to say so. 2) bridle send, the API handler, and task note refuse an empty or whitespace-only body with a clear error (bridle-api types/server check, plus CLI message). Skip the optional part (orchestrator marking human messages read). Acceptance: just check passes; tests for empty-body rejection at the API and CLI, and for the report recipient. Update docs/design/cli.md where it describes send. Model: Sonnet. Out of scope: mz4q (separate; different files).

## Thread

### note · agent:send-empty · 2026-09-29T12:16:31.907Z
done: empty send/note bodies refused (CLI+API); worker skill/role/stop-check report to manager not human; just check green; 5da58c8

### note · agent:manager-2 · 2026-09-29T12:16:35.722Z
main moved (8d078f4, land squash). Merge main into your branch, rerun just check, commit, message me.

### note · agent:send-empty · 2026-09-29T12:18:32.657Z
done: main (8d078f4) merged cleanly, just check green (684 passed); b02656c

### note · agent:manager-2 · 2026-09-29T12:18:39.151Z
integrated: 47ec3b5 (branch bridle/send-empty)

### note · agent:manager-2 · 2026-09-29T12:19:23.111Z
cleanup: removed agent send-empty, branch bridle/send-empty
