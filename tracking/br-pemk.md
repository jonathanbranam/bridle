+++
id = "br-pemk"
title = "Daemon usage periods: the week from the seven-day reset, its days, and each real five-hour window with peak and limit hit (ktgd step 2a)"
kind = "feature"
state = "planned"
created_at = "2026-10-13T23:05:38.293Z"
updated_at = "2026-10-13T23:06:47.153405Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "pemk"
+++

docs/tickets/open/daemon-usage-periods-the-week-from-the-seven-day-reset-its-d-pemk.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:37.864Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ask is the spec. Start: `rate_limit_history` (br-xxw9: store.rs, the /v1/usage/history route and RateLimitPoint in bridle-api types.rs). Add one additive route and types for the periods (week from the seven_day resets_at, its days, each real five_hour window with peak and limit-hit). Day boundaries: use the human's timezone from config if bridle has one, else UTC, and say which in the docs. Derive windows from `resets_at` changes, not fixed 5 h grids; "hit the limit" = status not allowed or utilization >= 1.0 (check what the readings carry). Pure functions over the readings with a table-driven test (two five-hour windows and a seven-day reset). Docs: api.md, usage-and-budget.md, CHANGELOG.md. No schema change. Out of scope: breakdowns (zn6s), gateway (v6cg).
