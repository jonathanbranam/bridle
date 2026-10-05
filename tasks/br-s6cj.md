+++
id = "br-s6cj"
title = "bridle-ui: a Tasks page, every project's open tasks with status and who's working them, mobile-first"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T22:45:35.578Z"
updated_at = "2026-10-05T07:45:52.093719Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
branch = "bridle/tasks-page"
commit = "f69b9afadcca28a9edc898d80166b45a9a83f93c"
summary = "Gateway read-only Tasks routes (crates/bridle-gateway/src/tasks.rs): GET /api/v1/projects/{project}/tasks?state=open|closed|all (default open = not integrated/dropped; id, title, kind, state, priority, claimed_by, agent {name, role} when claimed by agent:<name>, updated) and GET .../tasks/{id} (any state; body, thread, watchers, branch, blocks and blocked_by from /v1/edges, Blocks kind only). Own wire types with ts-rs bindings regenerated. Unknown project 404. Tests cover default filter, closed/all, closed detail with edges, agent lookup, unknown project and task. human-web-ui.md section 2 now records that read-only views of tasks (and agents/status, br-7sd9) are allowed; Tasks section added; CHANGELOG line added."
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

## Thread

### note · external:orchestrator · 2026-10-04T22:45:54.800Z
UI half: bridle-ui ui-umaq. Gateway decision: read-only views allowed (see brief).

### note · external:aide · 2026-10-04T22:46:11.165Z
watching the task

### note · agent:tasks-page · 2026-10-05T07:37:46.167Z
done: read-only gateway Tasks routes (list + detail with edges/agent), docs, changelog; just check exit 0, 1209 tests passed, sha 083856699b2d6b5c3706a51544c845b3f6fecb97. Merged main as of c2ac03d; main has since moved by docs-only br-tc7t (b80f427), not merged to keep the checked sha.

### note · agent:manager-2 · 2026-10-05T07:45:52.093Z
integrated: f69b9afadcca28a9edc898d80166b45a9a83f93c (branch bridle/tasks-page)
