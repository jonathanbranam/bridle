+++
id = "br-y25n"
title = "The human's to-do list has an order agents can set: priorities with enough levels to put things at the top"
kind = "feature"
state = "planned"
created_at = "2026-10-05T10:40:43.926Z"
updated_at = "2026-10-05T10:41:41.421876Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: y25n
Ticket (the ask, with the human's words; read first): docs/tickets/open/the-human-s-to-do-list-has-an-order-agents-can-set-prioritie-y25n.md

Goal: the human's to-dos (tasks claimed by human) have an order agents can set, so an agent can say "put these two at the top" and the human sees them first; and "top" stays meaningful when every agent does it.
Recommended design (keep it simple; change only with a reason): extend today's priority (low/normal/high, set with `bridle task priority`) with more levels, e.g. low, normal, high, urgent, critical, ordered; within a level, the most recently set goes first (newest on top), so a newer urgent thing outranks older ones at the same level, and only critical/urgent beat high. Record each change in the thread as today. Ordering lives in the daemon's task listing so CLI and UI agree.
Show it: `bridle task list --claimed-by human` (add the flag if missing) and the bridle-ui to-do list (crates/bridle-gateway items.rs/tasks.rs, bindings/*.ts, and the UI source it serves) sort by this order and show the level.
Files likely: crates/bridle-daemon/src/tasks.rs and server.rs, crates/bridle-api/src/types.rs (wire change: update all clients together), crates/bridle/src/cli.rs, crates/bridle-gateway, docs/design/cli.md and storage.md. Check how the stored priority column and old values migrate: existing low/normal/high rows must keep working (additive levels, no data migration expected; say so in the done note).
Acceptance: just check passes; tests for ordering (levels, newest-first within a level) and the CLI listing; docs updated.
Model: Sonnet. Out of scope: drag-and-drop reordering, per-agent quotas on top slots.
