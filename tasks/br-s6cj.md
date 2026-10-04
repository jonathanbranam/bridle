+++
id = "br-s6cj"
title = "bridle-ui: a Tasks page, every project's open tasks with status and who's working them, mobile-first"
kind = "feature"
state = "open"
created_at = "2026-10-04T22:45:35.578Z"
updated_at = "2026-10-04T22:45:35.751219Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: s6cj
Bridle-repo (gateway) half of docs/tickets/open/*-s6cj.md (read it; it quotes the human). The UI half is a bridle-ui task (see thread).

Approval: the human, via bridle-ui's aide (m-0190, 2026-10-04): "Try to resolve any questions and get that work moving. And queued up for after this other work lands."

Decision (orchestrator, resolving the open question for the human's ask): the gateway may expose READ-ONLY views of tasks, agents and status; still no agent control and no events stream. Update docs/design/human-web-ui.md section 2 to say so (this task does the tasks part; br-7sd9 the agents/status part), and record it there, not only here.

Goal: read-only gateway routes in crates/bridle-gateway (new module or next to items), proxying each project's daemon with the gateway's existing per-project credentials:
- GET /api/v1/projects/{project}/tasks?state=open|closed|all (default open: everything not integrated/dropped); each: id, title, kind, state, priority, claimed_by, updated.
- GET /api/v1/projects/{project}/tasks/{id}: the task with body, thread, watchers, claimed_by, branch, and its edges (blocks / blocked-by, from /v1/edges), for any state, so closed tasks still open from links.
- Who works it: claimed_by already names the manager or worker; include the agent's name and role when claimed by an agent.
Doc: the routes in human-web-ui.md. Tests: list default filters closed out, detail of a closed task, edges included, unknown project 404.
Acceptance: just check green. Model: sonnet.
Ordering: touches the same route registration and doc as br-7sd9; run one after the other (not an edge).
Deferred: live updates (events) until the static page is in use (the human: "static first"); ticket listing, until j28f settles IDs.
