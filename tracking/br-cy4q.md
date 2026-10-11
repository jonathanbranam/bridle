+++
id = "br-cy4q"
title = "Human to-dos are normal tickets: ticket new --for-human, to-do commands and the gateway list under ticket (22ab step 7b)"
kind = "feature"
state = "claimed"
created_at = "2026-10-11T13:35:46.520Z"
updated_at = "2026-10-11T15:26:16.717069Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wcy4q",
]
summary = "`ticket new --for-human [--priority P]` (crates/bridle/src/ticket.rs, cli.rs) writes the ticket file and creates the row planned/claimed by human (the daemon already did this for for_human + ticket), then sends the one inbox note (shared notify_human_todo in commands/task.rs; text now says `ticket done`). Hidden `task new --for-human` delegates to ticket::new, so it makes a ticket (it prints the file path now, not row JSON). Done/drop/list reuse q8cs's `ticket` row commands; nothing added. Gateway Todo gains `ticket` (items.rs, bindings/Todo.ts); order unchanged. Docs: cli.md, coordination.md, orchestrator and aide roles, CHANGELOG. Migration: existing to-dos covered by 0005 when the owner runs it; no new migration. Tests: cli_e2e for_human_todos_*, gateway items test."
ticket = "cy4q"
+++

Ticket: docs/tickets/open/human-to-dos-are-normal-tickets-ticket-new-for-human-to-do-c-cy4q.md (read "What's wanted" and "Verify"; it is the spec, human-approved). Starts only after br-res3 is integrated (blocked_by in the ticket); before step 10.

Goal: a human to-do is a normal ticket file with its row planned and claimed by human. `bridle ticket new "<title>" --for-human [--body ...] [--priority ...]` mints the file and the row in one step and sends the human the one inbox note, as `task new --for-human` does today (the caller commits the file, as for any ticket new). To-do commands in ticket form (`ticket list --claimed-by human`, done, drop --reason): reuse what q8cs already exposed under `ticket`, add only what is missing. The hidden `task new --for-human` alias also makes a ticket, so no new ticketless to-do appears. The gateway's to-do list (crates/bridle-gateway, items) shows the ticket (title and link) and keeps sort_by_priority; old routes stay one release. Docs: cli.md and coordination.md "Human to-dos"; the aide role in workflow/base/roles (it files most to-dos) says ticket new --for-human; CHANGELOG.
Files: crates/bridle/src (ticket and task new), crates/bridle-daemon (row creation if needed), crates/bridle-gateway, workflow/base/roles/aide, the docs above.
Migration: existing to-dos are covered by migration 0005 when the owner runs it; no new migration. State it in the done note.
Acceptance: just check passes; a test that ticket new --for-human makes a file and a row claimed by human, planned, with the inbox note; the alias also makes a ticket; a gateway test that the to-do list item carries the ticket and the order is unchanged.
Model: Sonnet. Out of scope: bridle-ui (22ab step 8), alias removal (step 10).

## Thread

### note · external:advisor/product-manager · 2026-10-11T13:36:16.154Z
advisor/product-manager: readied under the approved 22ab plan (step 7b) with the human's go on the design ("yes, for human todos, they become full, normal tickets like all others"). pm-1: plan it; blocked_by br-res3, so no worker before res3 lands.

### note · agent:wcy4q · 2026-10-11T15:26:10.176Z
done: ticket new --for-human (file + row claimed by human + inbox note), alias makes a ticket, gateway Todo carries ticket; just check exit 0, 1526 tests passed, sha 693bce39. Migration: covered by 0005, none new. Note: alias now prints the file path, not row JSON.

### note · agent:manager-2 · 2026-10-11T15:26:16.717Z
main moved. Merge main, re-run just check, message me the sha right away; I will land immediately.
