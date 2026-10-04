+++
id = "br-a3yd"
title = "bridle-ui: render front matter, and auto-link URLs, file paths and ticket and task IDs everywhere"
kind = "feature"
state = "open"
created_at = "2026-10-04T22:32:01.072Z"
updated_at = "2026-10-04T22:32:01.130130Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: a3yd
Bridle-repo (gateway) half of docs/tickets/open/*-a3yd.md (read it; it extends bnhn). The UI half is bridle-ui task (see thread).

Approval: the human, via bridle-ui's aide (m-0176, 2026-10-04): "IDs or something that map to a ticket or a task, those should all be links ... everywhere."

Goal: extend br-bnhn's POST /api/v1/projects/{project}/links/resolve (crates/bridle-gateway/src/documents.rs) so a target may also be:
- a bare ticket ID (4 chars of abcdefghjkmnpqrstuvwxyz23456789): resolve to the ticket file whose name ends -<id>.md, open/ then resolved/;
- a task ID <prefix>-<id> whose <id> is a ticket ID: resolve to that ticket (a ticket's first task takes its id). Other task IDs resolve to none (bridle-ui has no task view yet).
Update the route's section in docs/design/human-web-ui.md; tests: ticket ID open and resolved, task ID to ticket, unknown ID none.
Acceptance: just check green. Model: haiku (small, follows bnhn's code). Files: documents.rs only, plus the doc.
Deferred: cross-project IDs and project identifiers on tickets, until question j28f decides the scheme.
