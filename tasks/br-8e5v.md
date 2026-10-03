+++
id = "br-8e5v"
title = "Ticket from task with the same ID; tickets hold the why, tasks the work (k7tm B)"
kind = "feature"
state = "planned"
created_at = "2026-10-03T23:28:37.352Z"
updated_at = "2026-10-03T23:28:49.595091Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Human-approved (2026-10-03, relayed by advisor 'tickets', m-4116; the human's verbatim in the ticket, "Decided: tickets hold the why, tasks hold the work"). Slicing was left to the advisor by the human: "just make a decision on how to slice it".
Ticket: docs/tickets/open/tickets-and-tasks-why-both-k7tm.md, decisions 7-10 and "Slicing" B. Read the whole ticket.
Goal: 'bridle ticket new --from-task <id>' makes a ticket reusing the task's ID (when a pre-br-9e15 hex task ID contains 0 or 1, mint a fresh ID and link the two both ways). Content rules: tickets hold design decisions (the why and what), tasks hold the work and status; a build that comes out of a discussion ticket gets its own feature ticket linked to the discussion ticket, and its task is made from that; mechanical coordination stays in the task thread; a design question on a task gets a short 'needs design review, see the ticket' comment and the substance goes on the ticket. Update the tickets rule (workflow/base/rules), worker, manager and PM roles, docs/README.md, cli.md.
Acceptance: just check passes. Independent of k7tm A.

## Thread

### note · agent:pm-1 · 2026-10-03T23:28:49.595Z
Planned, tier 1, side by side with br-8eyu. Model: Sonnet. Role/rule text overlaps with br-8eyu: whichever lands second rebases. Migration: none (new flag and rule text only).
