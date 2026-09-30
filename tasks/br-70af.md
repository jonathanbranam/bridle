+++
id = "br-70af"
title = "Incidents as a task kind: build kind, states and the broadcast notice"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T01:29:57.368Z"
updated_at = "2026-09-30T01:57:18.217698Z"
branch = "bridle/incident-build"
commit = "5feb1e97502ce56dfe85f481085c6ff17ffec6ab"
summary = 'Incidents built as a task kind. `TaskKind::Incident`; plan/done/drop/reopen of one is orchestrator/human only (403 otherwise, server.rs `require_incident_owner`); incidents are never ready or queueable; `done` needs no commit and takes `--resolution`; `plan` also accepts `reopened` for incidents. The notice: schema V18 adds indexed `messages.incident_task`; AgentManager (supervisor.rs) gets `set_tasks` (OnceLock, wired in lib.rs) and `incident_activated` (fan-out to running/resumable agents), `fill_incident_notices` (spawn/resume/renew), `incident_updated` (rewrite unwritten notice, else an "updated" note) and `incident_resolved` (drop pending/held, one "resolved" note to those who saw it, then unlink all rows so a recurrence starts clean). Wire: `Message.incident_task`, `Status.incidents`, `TaskQuery.kind`, `DoneTaskRequest.resolution` (now `Default`). CLI: `task list -k`, `task done --resolution`, incidents line in `bridle status`. Tests in tests/incidents_test.rs. Docs: cli.md, storage.md, incidents.md, README, CHANGELOG. Not done: orchestrator watcher filter for created incident tasks (a watcher change per the design, not touched); `task list -k` filters client-side.'
+++

GOAL: implement the design in docs/design/agent-host/incidents.md as revised by the preceding task (incident = a task kind owned by the orchestrator; potential -> active -> resolved; persistent broadcast notice to every agent while active, including agents that start, resume or renew; resolved note to those who saw it; undelivered notices dropped). Read that doc first, plus crates/bridle-daemon/src/tasks.rs (task state machine), the message queue and agent start/resume/renew paths in crates/bridle-daemon, crates/bridle-api/src/types.rs (wire contract: change clients and daemon together), and crates/bridle/src/cli.rs. Acceptance: just check passes; tests for filing a potential incident, promote/drop/resolve permissions (orchestrator/human only), notice delivered to a running agent, to an agent started while active, and dropped-if-undelivered on resolve; docs in step (cli.md, storage.md, CHANGELOG). Model: Sonnet. Out of scope: cross-project incidents, daemon-detected incidents, budget holds becoming incidents, TUI display. Touches the task code and types.rs like br-c83e (human to-dos): run after it, not in parallel.

## Thread

### note · agent:incident-build · 2026-09-30T01:56:46.982Z
done: incidents as a task kind (kind, owner-only transitions, notice fan-out/fill/update/resolve, status list, task list -k, done --resolution); just check green, 772 tests; main merged; commit 2d577e9. Not done: orchestrator watcher filter for created incident tasks (a watcher change per design).

### note · agent:manager-2 · 2026-09-30T01:56:53.441Z
integrated: 5feb1e97502ce56dfe85f481085c6ff17ffec6ab (branch bridle/incident-build)

### note · agent:manager-2 · 2026-09-30T01:57:18.217Z
cleanup: removed agent incident-build, branch bridle/incident-build
