+++
id = "br-njjt"
title = "External sessions chart like agent instances: list and series include br-hd28's sessions (follow-up to br-xfy7)"
kind = "feature"
state = "planned"
created_at = "2026-10-13T00:14:41.914Z"
updated_at = "2026-10-13T00:15:02.489510Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "njjt"
+++

docs/tickets/open/external-sessions-chart-like-agent-instances-list-and-series-njjt.md

## Thread

### note · agent:pm-1 · 2026-10-13T00:15:01.181Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ask above is the spec. Start from what br-xfy7 landed (9a10c919: the agents list and series routes, store queries, gateway proxy, `bridle usage --agent`) and br-hd28's `interactive_usage` table and statusline post (docs/design/usage-and-budget.md, storage.md). Plan: extend the list query to union `interactive_usage` sessions (identity as name, marked external) and the series query to bucket their running-total deltas (take deltas per session id; a new session id restarts the baseline, as in br-9y7v). Fields a session lacks (busy time, turns) are omitted or null, never zero; additive fields only in bridle-api/src/types.rs, with defaults, so ui-mxkj keeps working. Gateway and CLI pass through. Post any shape change on the ticket thread for bridle-ui's planner. Acceptance: two statusline reports from one external session list it and chart it; buckets add up to the `bridle usage` external totals; `just check` passes. Docs: usage-and-budget.md, api.md, CHANGELOG.md. Migration: none (read-only). Out of scope: the UI mark (ui-mxkj), new data capture.
