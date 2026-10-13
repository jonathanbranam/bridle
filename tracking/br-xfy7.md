+++
id = "br-xfy7"
title = "Usage per agent instance over time: series and instance list in the daemon, gateway and CLI (bridle half of the per-agent chart)"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T22:53:04.789Z"
updated_at = "2026-10-13T00:13:42.504158Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wxfy7",
]
branch = "bridle/wxfy7"
commit = "9a10c919a322d18e8da4131255523ea4ee7d71b1"
summary = "Added per-instance usage over time, all read-only over the turns ledger (no migration). Store: sync::usage_series (UTC hour/day buckets, empty ones omitted) and usage_instances (removed flag = no agents row). Daemon: GET /v1/usage/agents and /v1/usage/agents/{id}/series; types in bridle-api types.rs, client usage_instances/usage_series. CLI: bridle usage --agent <name|id> [--since] [--bucket]. Gateway: /api/v1/projects/{project}/usage/agents[/{id}/series] with flat string-time views and TS bindings. Test: removed agent listed and charted, hourly and daily buckets sum to the breakdown. External sessions (hd28) live in a separate table, not turns, so they are not instances here; noted in usage-and-budget.md. Docs: api.md, usage-and-budget.md, human-web-ui.md, CHANGELOG."
ticket = "xfy7"
+++

docs/tickets/open/usage-per-agent-instance-over-time-series-and-instance-list-xfy7.md

## Thread

### note · agent:pm-1 · 2026-10-12T22:53:45.213Z
Brief (pm-1). Model: Sonnet. Size: M-L. The ask above is the spec. Blocked by br-9y7v (costs must be right). Queue it after br-hd28 and br-yydm: all three touch `bridle usage`, the usage routes and the usage docs, so run them one after another, not in parallel.

Build order inside the branch: (1) store queries over the turns ledger: series by bucket (hour, day) and the instance list (removed agents included, from the ledger alone); (2) daemon routes + wire types in crates/bridle-api/src/types.rs (clients and daemon together; additive); (3) CLI `bridle usage --agent <name|id> --since 7d`; (4) gateway proxy of both, following br-yydm's route in crates/bridle-gateway (system.rs pattern, TS bindings). Keep the response shapes plain so the bridle-ui half (ui-mxkj) can chart them; say the final route paths in a ticket comment as soon as they are fixed, because bridle-ui's planner waits on them.

If it runs past one branch's worth of context, stop after (1)-(3), mark the gateway proxy as the remainder, and tell the manager so it can be split. Acceptance as in the ask: a removed agent is listed and charted; buckets add up to `breakdown` totals; `just check` passes. Migration: none expected (read-only queries; add an index only if needed, as a normal DB migration). Docs: usage-and-budget.md, the API doc, storage.md if an index is added, CHANGELOG.md. Out of scope: the UI page, per-project budgets.

### note · agent:wxfy7 · 2026-10-12T23:55:46.136Z
Routes fixed (additive):
- GET /v1/usage/agents?since=  -> [UsageInstance{agent_id,name,role,model,removed,turns,cost_usd,first_turn_at,last_turn_at}], newest last turn first.
- GET /v1/usage/agents/{agent_id}/series?since=&bucket=hour|day (default day) -> UsageSeries{agent_id,bucket,since,points:[{start,turns,tokens{input,output,cache_read,cache_write},cost_usd,busy_seconds}]}, oldest first, empty buckets omitted, UTC bucket starts.
- Gateway proxies: GET /api/v1/projects/{project}/usage/agents and /api/v1/projects/{project}/usage/agents/{agent_id}/series, same query params.
Types: crates/bridle-api/src/types.rs.

### note · agent:wxfy7 · 2026-10-13T00:13:34.716Z
built: 6458b8eb (branch bridle/wxfy7)

### note · agent:wxfy7 · 2026-10-13T00:13:36.726Z
done: all four steps (store, daemon routes, CLI --agent, gateway proxy + TS bindings), no split needed; just check exit 0, 1630 tests run (1606 last full), main merged; 6458b8eb. Routes are on the ticket thread.

### note · agent:manager-2 · 2026-10-13T00:13:42.504Z
integrated: 9a10c919a322d18e8da4131255523ea4ee7d71b1 (branch bridle/wxfy7)
