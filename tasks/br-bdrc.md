+++
id = "br-bdrc"
title = "Traceability: tickets record who filed them, and whoever relays an ask watches the tasks made from it"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T21:49:07.246Z"
updated_at = "2026-10-07T05:41:37.752984Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/traceability-bdrc"
commit = "45f2b54630dac877466c76d986370a0a2880d409"
summary = "Tickets get an optional filed_by front-matter field: ticket new (and --from-task) writes the caller's principal from /v1/status, omitted if no daemon answers; ticket check needs no change (old tickets pass; tested). Part 2 already held: the daemon makes a task's creator a watcher, and ticket task creates the task as the caller, so the relayer is watching; added a daemon test for the ticket path rather than new code. Docs: docs/README.md, cli.md, CHANGELOG. Not done: backfill, commit-author attribution (out of scope)."
ticket = "bdrc"
+++

Build docs/tickets/open/traceability-tickets-record-who-filed-them-and-whoever-relay-bdrc.md (read it, and the xxxq task-watchers design in docs/design/ and the code for task created_by/watchers). Two parts. (1) Tickets record who filed them: bridle ticket new (and --from-task) writes a filed_by frontmatter field with the calling principal (e.g. external:aide), taken from the caller's identity the same way tasks get created_by. The field is optional, so existing tickets need no migration (bridle ticket check must accept its absence); document it in the ticket conventions in docs/README.md. (2) Whoever runs bridle ticket task <id> (or files a task from a ticket) is added as a watcher of the task automatically, in addition to the creator; the aide that relays an ask then gets the landing notice with no manual bridle task watch. Files: crates/bridle (ticket subcommands), crates/bridle-daemon (task creation/watchers if the watcher add belongs there), cli.md, docs/README.md. Acceptance: just check passes; tests for filed_by written, ticket check accepting old tickets, and auto-watch on ticket task. Model: Sonnet. Out of scope: backfilling filed_by on existing tickets (git history has no reliable author), commit-author attribution.

## Thread

### note · external:orchestrator · 2026-10-04T21:49:10.896Z
Approval: the human, 2026-10-04 via aide (m-0146): "there needs to be some traceability to which agent was responsible for these things. If you want to be notified when something lands, then you should be watching it. And that should be, I think, largely automatic." Full quote in the ticket (bc0fc9ae).

### note · agent:traceability-bdrc · 2026-10-07T05:41:14.299Z
done: filed_by on tickets + auto-watch test (part 2 already held via creator-watcher); check exit 0, 1291 tests; 13206ede

### note · agent:traceability-bdrc · 2026-10-07T05:41:15.826Z
done: just check exit 0, 1291 tests, commit 13206ede (main already merged). Part 2 needed no code: creator is already auto-watcher; added test.

### note · agent:manager-2 · 2026-10-07T05:41:21.205Z
integrated: 45f2b54630dac877466c76d986370a0a2880d409 (branch bridle/traceability-bdrc)

### note · agent:manager-2 · 2026-10-07T05:41:37.752Z
cleanup: removed agent traceability-bdrc, branch bridle/traceability-bdrc
