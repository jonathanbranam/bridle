+++
id = "br-7sd9"
title = "bridle-ui: a System page, each project's daemon, agents and (later) running servers"
kind = "feature"
state = "open"
created_at = "2026-10-04T22:45:35.661Z"
updated_at = "2026-10-04T22:45:35.771898Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 7sd9
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
