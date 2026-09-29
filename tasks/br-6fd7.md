+++
id = "br-6fd7"
title = "Reports go to the spawner not the human; bridle send refuses an empty body (hx7t)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T12:04:21.152Z"
updated_at = "2026-09-29T12:04:23.668963Z"
+++

Ticket: docs/questions/open/*hx7t.md (read it). Goal, two parts. 1) A worker's done:/progress report must go to its spawner or manager, never to human: check who workflow/base/skills/worker/SKILL.md and the stop-check block message from br-d99e (crates/bridle/src/stop_check.rs) tell a one-off worker to send to (the <sender> placeholder resolves to human for workers spawned by the human or orchestrator); make the default the spawning principal or the manager, and fix the skill text and the block reason to say so. 2) bridle send, the API handler, and task note refuse an empty or whitespace-only body with a clear error (bridle-api types/server check, plus CLI message). Skip the optional part (orchestrator marking human messages read). Acceptance: just check passes; tests for empty-body rejection at the API and CLI, and for the report recipient. Update docs/design/cli.md where it describes send. Model: Sonnet. Out of scope: mz4q (separate; different files).
