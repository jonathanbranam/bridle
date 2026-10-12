+++
id = "br-xfy7"
title = "Usage per agent instance over time: series and instance list in the daemon, gateway and CLI (bridle half of the per-agent chart)"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T22:53:04.789Z"
updated_at = "2026-10-12T23:47:59.608268Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wxfy7",
]
ticket = "xfy7"
+++

docs/tickets/open/usage-per-agent-instance-over-time-series-and-instance-list-xfy7.md

## Thread

### note · agent:pm-1 · 2026-10-12T22:53:45.213Z
Brief (pm-1). Model: Sonnet. Size: M-L. The ask above is the spec. Blocked by br-9y7v (costs must be right). Queue it after br-hd28 and br-yydm: all three touch `bridle usage`, the usage routes and the usage docs, so run them one after another, not in parallel.

Build order inside the branch: (1) store queries over the turns ledger: series by bucket (hour, day) and the instance list (removed agents included, from the ledger alone); (2) daemon routes + wire types in crates/bridle-api/src/types.rs (clients and daemon together; additive); (3) CLI `bridle usage --agent <name|id> --since 7d`; (4) gateway proxy of both, following br-yydm's route in crates/bridle-gateway (system.rs pattern, TS bindings). Keep the response shapes plain so the bridle-ui half (ui-mxkj) can chart them; say the final route paths in a ticket comment as soon as they are fixed, because bridle-ui's planner waits on them.

If it runs past one branch's worth of context, stop after (1)-(3), mark the gateway proxy as the remainder, and tell the manager so it can be split. Acceptance as in the ask: a removed agent is listed and charted; buckets add up to `breakdown` totals; `just check` passes. Migration: none expected (read-only queries; add an index only if needed, as a normal DB migration). Docs: usage-and-budget.md, the API doc, storage.md if an index is added, CHANGELOG.md. Out of scope: the UI page, per-project budgets.
