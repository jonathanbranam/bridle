+++
id = "br-cy4q"
title = "Human to-dos are normal tickets: ticket new --for-human, to-do commands and the gateway list under ticket (22ab step 7b)"
kind = "feature"
state = "planned"
created_at = "2026-10-11T13:35:46.520Z"
updated_at = "2026-10-11T13:36:28.008894Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
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
