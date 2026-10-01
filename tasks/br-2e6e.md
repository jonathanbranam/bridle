+++
id = "br-2e6e"
title = "Ticket kinds and two-way ticket-task links: ticket new --kind, ticket set kind, task kind, link check (v3dk slice A)"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T18:52:53.340Z"
updated_at = "2026-10-01T20:01:32.118917Z"
branch = "bridle/ticket-kinds"
commit = "fcee4cb24fb047e198bf9f508528cac8cea638a7"
summary = """Slice A of v3dk. `ticket new` requires --kind (no default); ticket gets `kind:` and `tasks: []` frontmatter, the task gets the same kind and its id is written into `tasks:`. `ticket set <id> kind <kind>` (validated) and `tasks` are editable. `bridle task kind <id> <kind>`: new SetKindRequest, POST /v1/tasks/{id}/kind, TaskManager::set_kind (open only, else 409), thread note and `task.kind` event. `ticket check` returns errors and warnings: unknown kind and one-sided links are errors (task side checked via the daemon's task bodies "original id: <id>", skipped when the daemon is unreachable); missing kind/tasks (and a task naming a ticket with no tasks field) are warnings behind MISSING_KIND_OR_LINK_IS_ERROR in ticket.rs, which slice B flips. Only `ticket new` writes the link; no other task-from-ticket path exists yet. Docs: cli.md, api.md, docs/README.md, the tickets rule, CHANGELOG."""
+++

Ticket: docs/tickets/open/tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk.md (read it all: 'Decided by the human' and '5, as approved'). Code: crates/bridle/src/ticket.rs (line ~86 hardcodes TaskKind::Question), the 'bridle task priority' command and its daemon route as the pattern for 'task kind', crates/bridle-api/src/types.rs, docs/design/cli.md, docs/design/storage.md, docs/README.md (ticket conventions).

Goal (slice A: the code; the backfill of existing tickets is slice B, a migration):
1. 'bridle ticket new' requires --kind (the task kinds); no default. The ticket gets 'kind:' frontmatter and its task gets the same kind.
2. 'bridle ticket set <id> kind <kind>' edits a ticket's kind any time. Ticket and task kinds are independent; no check compares them.
3. 'bridle task kind <id> <kind>': allowed only while the task is 'open'; refused in planned, claimed, dropped, integrated and reopened; recorded in the thread and as an event; like 'task priority' (wire type, route, CLI, docs together).
4. Two-way link: the ticket records every task made from it ('tasks: [br-xxxx]' frontmatter), written when 'ticket new' or a task-from-ticket is created; the task already records its ticket (original id / path). 'bridle ticket check' fails an unknown kind or a link present on one side only.
5. STRICTNESS IS SLICE B'S: existing tickets have no kind and no tasks field. Until the migration runs, 'ticket check' must report a missing kind or missing link as a warning, not an error, behind one clearly named constant or flag, so 'just check' and every existing project keep passing. Slice B flips it to an error.
6. Update the ticket conventions in docs/README.md, cli.md, the workflow ticket rules that mention ticket new, CHANGELOG.
Tests: ticket new without --kind fails; kind in frontmatter and task; ticket set kind; task kind allowed open and refused in each other state with the event; link written both ways; check flags a one-sided link and an unknown kind, warns on missing kind.
Acceptance: just check passes. Model: Sonnet.
Migration plan: slice B (the backfill migration, blocked on this task and on the automatic run). Nothing in this slice may make an existing project's checks fail.
Out of scope: the backfill; making missing kind an error.

## Thread

### note · agent:ticket-kinds · 2026-10-01T20:01:11.219Z
done: ticket kinds, task kind, two-way link check (missing kind/link only warns, MISSING_KIND_OR_LINK_IS_ERROR); just check exit 0, 966 tests; 2370858

### note · agent:manager-2 · 2026-10-01T20:01:20.988Z
integrated: fcee4cb24fb047e198bf9f508528cac8cea638a7 (branch bridle/ticket-kinds)

### note · agent:manager-2 · 2026-10-01T20:01:32.118Z
cleanup: removed agent ticket-kinds, branch bridle/ticket-kinds
