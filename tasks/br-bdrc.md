+++
id = "br-bdrc"
title = "Traceability: tickets record who filed them, and whoever relays an ask watches the tasks made from it"
kind = "feature"
state = "planned"
created_at = "2026-10-04T21:49:07.246Z"
updated_at = "2026-10-04T21:49:35.813723Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "bdrc"
+++

Build docs/tickets/open/traceability-tickets-record-who-filed-them-and-whoever-relay-bdrc.md (read it, and the xxxq task-watchers design in docs/design/ and the code for task created_by/watchers). Two parts. (1) Tickets record who filed them: bridle ticket new (and --from-task) writes a filed_by frontmatter field with the calling principal (e.g. external:aide), taken from the caller's identity the same way tasks get created_by. The field is optional, so existing tickets need no migration (bridle ticket check must accept its absence); document it in the ticket conventions in docs/README.md. (2) Whoever runs bridle ticket task <id> (or files a task from a ticket) is added as a watcher of the task automatically, in addition to the creator; the aide that relays an ask then gets the landing notice with no manual bridle task watch. Files: crates/bridle (ticket subcommands), crates/bridle-daemon (task creation/watchers if the watcher add belongs there), cli.md, docs/README.md. Acceptance: just check passes; tests for filed_by written, ticket check accepting old tickets, and auto-watch on ticket task. Model: Sonnet. Out of scope: backfilling filed_by on existing tickets (git history has no reliable author), commit-author attribution.

## Thread

### note · external:orchestrator · 2026-10-04T21:49:10.896Z
Approval: the human, 2026-10-04 via aide (m-0146): "there needs to be some traceability to which agent was responsible for these things. If you want to be notified when something lands, then you should be watching it. And that should be, I think, largely automatic." Full quote in the ticket (bc0fc9ae).
