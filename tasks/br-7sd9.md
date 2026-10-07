+++
id = "br-7sd9"
title = "bridle-ui: a System page, each project's daemon, agents and (later) running servers"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T22:45:35.661Z"
updated_at = "2026-10-05T08:41:19.265757Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
branch = "bridle/system-page"
commit = "9fbe2eee51e36a0b4fee7a2f9be8b33f50f547d1"
summary = "Gateway read-only GET /api/v1/projects/{project}/system (SystemView: reachable/error/status; daemon down is a 200 with reachable:false, unknown project 404) and .../agents (AgentList; stopped agents included with a stopped flag, the open task each holds, running first). New crates/bridle-gateway/src/system.rs with allow-listed own wire types, ts-rs bindings regenerated. Dropped: principal, daemon workspace/repo/url, agent session id/pid/cwd/worktree/created_by, session pid/pane/claude session id (listed in human-web-ui.md). Ports and live updates deferred. Tests: status passthrough, unreachable, agents list, secrets stripped, unknown project."
ticket = "7sd9"
+++

Bridle-repo (gateway) half of docs/tickets/open/*-7sd9.md (read it; it quotes the human). The UI half is a bridle-ui task (see thread).

Approval: the human, via bridle-ui's aide (m-0190, 2026-10-04): "think about everything that's in Bridal, what it can do, and how you can render that ... Try to resolve any questions and get that work moving."

Decision (orchestrator): the gateway may expose READ-ONLY views of agents and daemon status; no agent control, no events stream. Update docs/design/human-web-ui.md section 2 accordingly (br-s6cj does the tasks part).

Goal: read-only gateway routes proxying each project's daemon:
- GET /api/v1/projects/{project}/system: the daemon's /v1/status (pid, version, start time, CI, incidents, budget/governor, rate limits, sessions, upgrade state), with the daemon's reachability (down = a clear "unreachable", not a 500).
- GET /api/v1/projects/{project}/agents: /v1/agents (name, role, state, model, task, branch, context, cost); include stopped agents with a flag so the UI can hide them.
Strip anything secret (tokens, credential paths) from what's passed through; list what you dropped in the doc.
Tests: status passthrough, unreachable daemon, agents list, secrets stripped.
Acceptance: just check green. Model: sonnet.
Ordering: after br-s6cj (same route registration and doc; not an edge).
Deferred: servers and ports (/v1/ports), until the first version is in use (the human: "that can come later"); live updates, as for tasks.

## Thread

### note · external:orchestrator · 2026-10-04T22:45:54.819Z
UI half: bridle-ui ui-ng82.

### note · external:aide · 2026-10-04T22:46:11.189Z
watching the task

### note · agent:system-page · 2026-10-05T08:37:09.082Z
done: gateway /system and /agents routes read-only; just check exit 0, 1214 tests passed; a6beee8c

### note · agent:manager-2 · 2026-10-05T08:37:14.079Z
integrated: 9fbe2eee51e36a0b4fee7a2f9be8b33f50f547d1 (branch bridle/system-page)

### note · agent:manager-2 · 2026-10-05T08:41:19.265Z
cleanup: removed agent system-page, branch bridle/system-page
