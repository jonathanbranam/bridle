---
id: br-92za
title: "Code rename slice 2: wire types say ticket and accept both names for one release (95t9)"
kind: arch-revision
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-95t9]
related: []
parent: br-95t9
---

## The ask

Child of [[code-rename-no-type-is-called-task-types-wire-tasks-table-st-95t9|95t9]] (22ab step 9), split off by pm-1 on 2026-10-11 ~7:05 AM ET; made a child ticket by advisor product-manager so the order is a `blocked_by` link. pm-1's brief, as written:

Ticket: docs/tickets/open/code-rename-no-type-is-called-task-types-wire-tasks-table-st-95t9.md (read The ask and 22ab section 9 Code second pass). Slice 2 of 3, after slice 1 (br-95t9, storage) is integrated. Goal: the wire contract in crates/bridle-api/src/types.rs, the HTTP client, the daemon server, the CLI, the TUI, the gateway and the mail bridge move from task names to ticket names (types, JSON field names, endpoint paths) while still ACCEPTING the old names for one release: serde aliases on the way in, old endpoint paths kept as routes to the same handlers, and the daemon answering an old client in the old shape if its fields differ. The lesson of incident 2ax5: a breaking wire change broke cross-project messaging, so wire changes go to all clients and the daemon together, and an old client must still talk to the new daemon. bridle-ui reads the gateway: the old gateway routes from br-aejj stay. Docs: docs/design/agent-host/api.md, CHANGELOG. Migration: none for project files; compatibility is the migration; state it in the done note. Acceptance: just check passes; a wire compatibility test: an old-shape request and an old-shape JSON body are accepted by the new daemon and give a correct answer; new names work. Model: Sonnet. If context nears 150K tokens land a coherent part (types and server first, clients second) and say what remains. Out of scope: SQL table and state folder (slice 1), pure-internal identifiers (slice 3), alias removal (step 10), bridle-ui.
