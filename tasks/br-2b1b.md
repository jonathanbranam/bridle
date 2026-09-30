+++
id = "br-2b1b"
title = "wait-for-wake wakes the orchestrator on a created incident task"
kind = "chore"
state = "integrated"
created_at = "2026-09-30T01:57:27.116Z"
updated_at = "2026-09-30T02:26:18.643231Z"
branch = "bridle/incident-wake"
commit = "727184b9073bf7e84168960403981942c0f4755e"
summary = "wait-for-wake wakes the orchestrator on a created incident task: added incident_created filter to wake_for_event, documented in orchestrator.md, and updated CHANGELOG"
+++

GOAL: 'bridle wait-for-wake' (the orchestrator's watcher, crates/bridle/src/commands.rs; design: docs/design/agent-host/incidents.md ~line 105) should wake when a task of kind 'incident' is created (event task.created with kind incident; see crates/bridle-api/src/types.rs and docs/design/agent-host/api.md for the event shape). A filter change in the watcher, not daemon work; if the event lacks the kind, the smallest fix is to add it to the event payload (change types.rs, daemon and clients together). Also mention it in the wake list in workflow/base/roles/orchestrator.md (line ~55) and CHANGELOG. Acceptance: just check passes; a test that a created incident task wakes and a created feature task does not. Model: Haiku. Out of scope: server-side 'task list -k' filtering, waking other roles.

## Thread

### note · agent:manager-2 · 2026-09-30T02:15:36.409Z
You were interrupted by a daemon restart. Check git status/log on your branch, finish what's missing, merge main, run the check per the worker skill, make sure the summary exists, commit, message me.

### note · agent:incident-wake · 2026-09-30T02:25:41.819Z
done: wait-for-wake wakes on created incident tasks (test passes); e5f8a53

### note · agent:manager-2 · 2026-09-30T02:25:51.029Z
main moved (br-a618 landed). Merge main, confirm ancestor, message me the sha.

### note · agent:incident-wake · 2026-09-30T02:25:57.436Z
main merged; ancestor confirmed; 2cfbcf2

### note · agent:manager-2 · 2026-09-30T02:26:06.231Z
integrated: 727184b9073bf7e84168960403981942c0f4755e (branch bridle/incident-wake)

### note · agent:manager-2 · 2026-09-30T02:26:18.643Z
cleanup: removed agent incident-wake, branch bridle/incident-wake
