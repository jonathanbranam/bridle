+++
id = "br-aejj"
title = "User-facing rename slice 3: gateway exposes tickets under ticket names, old routes kept (q8cs)"
kind = "feature"
state = "planned"
created_at = "2026-10-11T09:12:01.655Z"
updated_at = "2026-10-11T11:03:39.200467Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
summary = "Gateway now answers /projects/{project}/tickets, /tickets/{id} and POST /tickets/{id}/{action} with the same handlers as the tasks routes (lib.rs); the list is keyed `tickets` via new TicketList (tasks.rs, binding TicketList.ts). Task routes unchanged for one release. Test compares old and new route status and body. Docs: human-web-ui.md, CHANGELOG. Migration: none."
ticket = "aejj"
parent = "br-q8cs"
+++

docs/tickets/open/user-facing-rename-slice-3-gateway-exposes-tickets-under-tic-aejj.md

Ticket: docs/tickets/open/user-facing-rename-bridle-ticket-replaces-bridle-task-hidden-q8cs.md (read The ask and 22ab section 9). Slice 3 of 3, after br-q8cs (CLI slice) is integrated; independent of slice 2 but do not run at the same time as it if both touch docs/design. Goal: bridle-gateway (crates/bridle-gateway) exposes tickets under ticket-named routes and JSON, and the old task routes keep working unchanged for one release (bridle-ui reads the gateway; its own change is step 8 in the bridle-ui project, not here). Reuse the same handlers for both route sets. Docs: the gateway section of docs/design (find it via docs/README.md), CHANGELOG. Migration: none; old routes are kept; state it in the done note. Acceptance: just check passes; tests: each new ticket route returns what its old task route returns; old routes unchanged. Model: Sonnet. Out of scope: bridle-ui, code rename, alias removal.

## Thread

### note · agent:waejj · 2026-10-11T11:03:39.200Z
done: gateway /tickets routes share the tasks handlers, list keyed tickets, old routes unchanged, no migration; just check exit 0, 1514 tests run (1514 passed, 6 skipped), sha dacdea79
