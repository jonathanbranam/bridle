+++
id = "br-8e5v"
title = "Ticket from task with the same ID; tickets hold the why, tasks the work (k7tm B)"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T23:28:37.352Z"
updated_at = "2026-10-03T23:59:22.119767Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/ticket-from-task"
commit = "7de2bc9dfd10ff4fbdde9963924db6c6c024fc12"
summary = "Added 'bridle ticket new --from-task <id>' (cli.rs, ticket.rs): title/kind default to the task's; the ticket reuses the task's ID, or a fresh one when the tail has 0/1 (old hex) or a ticket already has it; ticket tasks: lists the task and the task body gets the 'original id:' first line, so ticket check sees both sides. Tickets rule and worker/manager/PM roles carry decisions 7-10; docs/README.md, cli.md, CHANGELOG updated. Unit test for id reuse only; the daemon-linked path is untested (no daemon in unit tests)."
+++

Human-approved (2026-10-03, relayed by advisor 'tickets', m-4116; the human's verbatim in the ticket, "Decided: tickets hold the why, tasks hold the work"). Slicing was left to the advisor by the human: "just make a decision on how to slice it".
Ticket: docs/tickets/open/tickets-and-tasks-why-both-k7tm.md, decisions 7-10 and "Slicing" B. Read the whole ticket.
Goal: 'bridle ticket new --from-task <id>' makes a ticket reusing the task's ID (when a pre-br-9e15 hex task ID contains 0 or 1, mint a fresh ID and link the two both ways). Content rules: tickets hold design decisions (the why and what), tasks hold the work and status; a build that comes out of a discussion ticket gets its own feature ticket linked to the discussion ticket, and its task is made from that; mechanical coordination stays in the task thread; a design question on a task gets a short 'needs design review, see the ticket' comment and the substance goes on the ticket. Update the tickets rule (workflow/base/rules), worker, manager and PM roles, docs/README.md, cli.md.
Acceptance: just check passes. Independent of k7tm A.

## Thread

### note · agent:pm-1 · 2026-10-03T23:28:49.595Z
Planned, tier 1, side by side with br-8eyu. Model: Sonnet. Role/rule text overlaps with br-8eyu: whichever lands second rebases. Migration: none (new flag and rule text only).

### note · agent:ticket-from-task · 2026-10-03T23:59:14.940Z
done: ticket new --from-task, tickets rule and role text, docs and CHANGELOG; just check green (1113 tests); 142727f

### note · agent:manager-2 · 2026-10-03T23:59:22.119Z
integrated: 7de2bc9dfd10ff4fbdde9963924db6c6c024fc12 (branch bridle/ticket-from-task)
