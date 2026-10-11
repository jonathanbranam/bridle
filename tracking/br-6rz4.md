+++
id = "br-6rz4"
title = "ticket new: a body with its own '## The ask' gets a second, empty one, and ticket task refuses"
kind = "fix"
state = "integrated"
created_at = "2026-10-11T06:05:26.191Z"
updated_at = "2026-10-11T19:50:03.711669Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w6rz4",
]
priority_at = "2026-10-11T17:17:58.129126Z"
branch = "bridle/w6rz4"
commit = "097c063754b756878c112f29118efcb21353b55e"
summary = "ticket new no longer adds the template's empty '## The ask' when the body has its own; task_source (ready check) accepts any non-empty '## The ask' section, so old files with a leading empty heading pass, while all-empty is still refused. crates/bridle/src/ticket.rs, 2 new tests, CHANGELOG."
ticket = "6rz4"
+++

Ticket: docs/tickets/open/ticket-new-a-body-with-its-own-the-ask-gets-a-second-empty-o-6rz4.md (read it; human approved 2026-10-11, hit five times). Small.

Goal: when the body given to `bridle ticket new --body/--body-file` already has a `## The ask` heading, the ticket must not also get the template's empty one; and `bridle ticket ready` (the old `ticket task` refusal text may still exist in the alias; check) must read the first NON-EMPTY `## The ask` section so an old file with a leading empty heading is not refused. Fix both ends, small. A body without the heading goes under the template heading as today.
Files: crates/bridle/src/ticket.rs (new and the ask check), tests, CHANGELOG.
Migration: none. Acceptance: just check passes; tests: body with its own heading gives exactly one; body without gets the template one; a file with an empty first and a filled second heading passes the readiness check; a file with only empty ones is still refused. Model: Haiku. Out of scope: any other ticket template change.

## Thread

### note · external:advisor/product-manager · 2026-10-11T06:05:38.524Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-11T06:05:38.574Z
advisor (product-manager): kept pending at low; goes to the human in the morning to ready (small, Haiku).

### note · external:advisor/product-manager · 2026-10-11T07:22:56.948Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-11T17:17:58.129Z
priority: low -> normal

### note · external:advisor/product-manager · 2026-10-11T17:18:05.368Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied, raised low -> normal (hit 5 times, latest ft3b).

### note · agent:w6rz4 · 2026-10-11T19:33:18.746Z
done: ticket new keeps one 'The ask'; ready reads first non-empty one; just check exit 0, 1535 tests passed, main merged (up to date); 7af02434

### note · agent:manager-2 · 2026-10-11T19:49:49.766Z
integrated: 097c063754b756878c112f29118efcb21353b55e (branch bridle/w6rz4)

### note · agent:manager-2 · 2026-10-11T19:50:03.711Z
cleanup: removed agent w6rz4, branch bridle/w6rz4
