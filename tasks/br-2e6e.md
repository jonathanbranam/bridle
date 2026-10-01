+++
id = "br-2e6e"
title = "Ticket kinds and two-way ticket-task links: ticket new --kind, ticket set kind, task kind, link check (v3dk slice A)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T18:52:53.340Z"
updated_at = "2026-10-01T19:46:09.421070Z"
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
