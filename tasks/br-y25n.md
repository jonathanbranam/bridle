+++
id = "br-y25n"
title = "The human's to-do list has an order agents can set: priorities with enough levels to put things at the top"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T10:40:43.926Z"
updated_at = "2026-10-05T11:59:16.138261Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/priorities"
commit = "6c72d8a702bd13a0cd3df3e76ade610d3d1d65b8"
summary = "Added critical and urgent levels above high (TaskPriority, CLI args, gateway Priority + regenerated bindings) and a priority_at timestamp (set on task new/task priority, stored in frontmatter, optional). One shared sort_by_priority in bridle-api orders CLI 'task list' and the gateway to-do list: level first; newest-set first at high and above, oldest first at normal/low. No data migration: old low/normal/high rows and files load unchanged; priority_at absent falls back to created_at. Dropped the ticket's 'ordered' level (YAGNI). Docs: cli.md, storage.md, CHANGELOG. UI source is not in this repo; it consumes the regenerated Priority.ts binding and the already-sorted list, so it needs to show the two new level names."
+++

original id: y25n
Ticket (the ask, with the human's words; read first): docs/tickets/open/the-human-s-to-do-list-has-an-order-agents-can-set-prioritie-y25n.md

Goal: the human's to-dos (tasks claimed by human) have an order agents can set, so an agent can say "put these two at the top" and the human sees them first; and "top" stays meaningful when every agent does it.
Recommended design (keep it simple; change only with a reason): extend today's priority (low/normal/high, set with `bridle task priority`) with more levels, e.g. low, normal, high, urgent, critical, ordered; within a level, the most recently set goes first (newest on top), so a newer urgent thing outranks older ones at the same level, and only critical/urgent beat high. Record each change in the thread as today. Ordering lives in the daemon's task listing so CLI and UI agree.
Show it: `bridle task list --claimed-by human` (add the flag if missing) and the bridle-ui to-do list (crates/bridle-gateway items.rs/tasks.rs, bindings/*.ts, and the UI source it serves) sort by this order and show the level.
Files likely: crates/bridle-daemon/src/tasks.rs and server.rs, crates/bridle-api/src/types.rs (wire change: update all clients together), crates/bridle/src/cli.rs, crates/bridle-gateway, docs/design/cli.md and storage.md. Check how the stored priority column and old values migrate: existing low/normal/high rows must keep working (additive levels, no data migration expected; say so in the done note).
Acceptance: just check passes; tests for ordering (levels, newest-first within a level) and the CLI listing; docs updated.
Model: Sonnet. Out of scope: drag-and-drop reordering, per-agent quotas on top slots.

## Thread

### note · agent:priorities · 2026-10-05T11:59:11.418Z
done: critical/urgent priority levels + newest-set-first ordering shared by CLI and gateway; just check exit 0, 1226 tests passed; 6455fdf1

### note · agent:manager-2 · 2026-10-05T11:59:16.138Z
integrated: 6c72d8a702bd13a0cd3df3e76ade610d3d1d65b8 (branch bridle/priorities)
