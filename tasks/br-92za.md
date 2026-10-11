+++
id = "br-92za"
title = "Code rename slice 2: wire types say ticket and accept both names for one release (95t9)"
kind = "arch-revision"
state = "pending"
created_at = "2026-10-11T11:05:19.144Z"
updated_at = "2026-10-11T11:06:07.515218Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
ticket = "92za"
parent = "br-95t9"
+++

docs/tickets/open/code-rename-slice-2-wire-types-say-ticket-and-accept-both-na-92za.md

Ticket: docs/tickets/open/code-rename-no-type-is-called-task-types-wire-tasks-table-st-95t9.md (read The ask and 22ab section 9 Code second pass). Slice 2 of 3, after slice 1 (br-95t9, storage) is integrated. Goal: the wire contract in crates/bridle-api/src/types.rs, the HTTP client, the daemon server, the CLI, the TUI, the gateway and the mail bridge move from task names to ticket names (types, JSON field names, endpoint paths) while still ACCEPTING the old names for one release: serde aliases on the way in, old endpoint paths kept as routes to the same handlers, and the daemon answering an old client in the old shape if its fields differ. The lesson of incident 2ax5: a breaking wire change broke cross-project messaging, so wire changes go to all clients and the daemon together, and an old client must still talk to the new daemon. bridle-ui reads the gateway: the old gateway routes from br-aejj stay. Docs: docs/design/agent-host/api.md, CHANGELOG. Migration: none for project files; compatibility is the migration; state it in the done note. Acceptance: just check passes; a wire compatibility test: an old-shape request and an old-shape JSON body are accepted by the new daemon and give a correct answer; new names work. Model: Sonnet. If context nears 150K tokens land a coherent part (types and server first, clients second) and say what remains. Out of scope: SQL table and state folder (slice 1), pure-internal identifiers (slice 3), alias removal (step 10), bridle-ui.
