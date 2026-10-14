+++
id = "br-x8c8"
title = "Mail to a visitor whose recorded home is this daemon loops back to the sender silently (orchestrator@remus after the move to remus-home)"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T18:15:03.852Z"
updated_at = "2026-10-14T20:55:01.693489Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:x8c8",
]
ticket = "x8c8"
+++

docs/tickets/open/mail-to-a-visitor-whose-recorded-home-is-this-daemon-loops-b-x8c8.md

## Thread

### note · agent:pm-1 · 2026-10-14T18:15:56.290Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body is the spec. Part 1 (do first, small): in crates/bridle-daemon/src/server.rs `forward_to_home`, refuse with an error to the sender when the visitor's recorded home (`store.visitor_home`) is this daemon's own project; never deliver to the local principal of the same name. Test: a visitor whose home is the local project gets an error, not a loop (see the existing forward_to_home tests). Part 2: when a principal's home changes (an orchestrator moves to <host>-home), re-pairing updates the recorded home on other daemons: check whether `bridle token pair` (crates/bridle/src/token_pair.rs) already does and how; if it needs changes there, STOP after part 1 and tell the manager (br-su62 and br-g9qm are editing token_pair.rs, so part 2 must run after them as its own ticket). If it already works, document it. Docs: add-a-machine.md home-project and move steps (shared with br-8uxe: run after it), docs/design/agent-host (the forwarding section), CHANGELOG.md. No migration. Acceptance: just check. Out of scope: automatic home updates without re-pairing.
