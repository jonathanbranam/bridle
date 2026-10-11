---
id: br-aejj
title: "User-facing rename slice 3: gateway exposes tickets under ticket names, old routes kept (q8cs)"
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-sehn]
related: []
parent: br-q8cs
---

## The ask

Child of [[user-facing-rename-bridle-ticket-replaces-bridle-task-hidden-q8cs|q8cs]] (22ab step 7), split off by pm-1 on 2026-10-11 ~5:12 AM ET; made a child ticket by advisor product-manager so the order is a `blocked_by` link (the task dependency is gone). pm-1's brief, as written:

Ticket: docs/tickets/open/user-facing-rename-bridle-ticket-replaces-bridle-task-hidden-q8cs.md (read The ask and 22ab section 9). Slice 3 of 3, after br-q8cs (CLI slice) is integrated; independent of slice 2 but do not run at the same time as it if both touch docs/design. Goal: bridle-gateway (crates/bridle-gateway) exposes tickets under ticket-named routes and JSON, and the old task routes keep working unchanged for one release (bridle-ui reads the gateway; its own change is step 8 in the bridle-ui project, not here). Reuse the same handlers for both route sets. Docs: the gateway section of docs/design (find it via docs/README.md), CHANGELOG. Migration: none; old routes are kept; state it in the done note. Acceptance: just check passes; tests: each new ticket route returns what its old task route returns; old routes unchanged. Model: Sonnet. Out of scope: bridle-ui, code rename, alias removal.
