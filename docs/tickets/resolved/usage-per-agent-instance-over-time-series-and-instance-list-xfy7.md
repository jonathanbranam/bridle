---
id: br-xfy7
title: "Usage per agent instance over time: series and instance list in the daemon, gateway and CLI (bridle half of the per-agent chart)"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: performance
repos: [bridle]
changes: []
specs: []
blocked_by: [br-9y7v]
related: [br-rdkn, br-hd28, br-yydm]
parent: br-rdkn
resolved: 2026-10-13T00:14:13Z
---

## The ask

The human, 2026-10-12 ~6:50 PM ET (to advisor product-manager), verbatim: "When this is done I want to be able to choose a specific agent instance and chart its usage". In epic br-rdkn (usage you can trust); the bridle-ui half is a ui- ticket filed with this one.

## What's there

The turns ledger keeps one row per turn per agent (tokens, cost, busy time; `turns(agent_id, agent_name, role, model, ...)`, no FK, so turns outlive `bridle rm`). `GET /v1/usage/breakdown?by=agent&since=` gives totals per agent, not a series over time. The gateway proxies neither (br-yydm adds only the limit history).

## The ask (bridle half)

1. Daemon: one agent instance's usage as a time series: `GET /v1/usage/agents/{agent_id}/series?since=&bucket=` (or the planner's better shape), buckets of tokens (input, output, cache read, cache write), cost, turns and busy time; bucket sizes at least hour and day. Works for removed agents (from the ledger alone).
2. A list of agent instances that have usage in a period, with name, role, model, first and last turn, so the UI can offer the choice (live and removed).
3. Once br-hd28 lands, the external and interactive sessions are instances too, chartable the same way.
4. Gateway: proxy both per project, as br-yydm does for the history. CLI: `bridle usage --agent <name|id> --since 7d` prints the same series as a table.

Acceptance: tests on the series and the list (a removed agent still listed and charted; buckets add up to `breakdown` totals for the same period); docs/design (usage-and-budget.md, api.md, the gateway's routes) and CHANGELOG.

Blocked by br-9y7v (costs must be right first). br-hd28 adds the external sessions; not a blocker.
