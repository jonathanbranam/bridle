+++
id = "br-2b1b"
title = "wait-for-wake wakes the orchestrator on a created incident task"
kind = "chore"
state = "planned"
created_at = "2026-09-30T01:57:27.116Z"
updated_at = "2026-09-30T02:11:01.961273Z"
summary = "wait-for-wake wakes the orchestrator on a created incident task: added incident_created filter to wake_for_event, documented in orchestrator.md, and updated CHANGELOG"
+++

GOAL: 'bridle wait-for-wake' (the orchestrator's watcher, crates/bridle/src/commands.rs; design: docs/design/agent-host/incidents.md ~line 105) should wake when a task of kind 'incident' is created (event task.created with kind incident; see crates/bridle-api/src/types.rs and docs/design/agent-host/api.md for the event shape). A filter change in the watcher, not daemon work; if the event lacks the kind, the smallest fix is to add it to the event payload (change types.rs, daemon and clients together). Also mention it in the wake list in workflow/base/roles/orchestrator.md (line ~55) and CHANGELOG. Acceptance: just check passes; a test that a created incident task wakes and a created feature task does not. Model: Haiku. Out of scope: server-side 'task list -k' filtering, waking other roles.
