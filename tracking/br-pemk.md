+++
id = "br-pemk"
title = "Daemon usage periods: the week from the seven-day reset, its days, and each real five-hour window with peak and limit hit (ktgd step 2a)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:38.293Z"
updated_at = "2026-10-14T00:51:00.553228Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:pemk",
]
branch = "bridle/pemk"
commit = "d519f5f4be4e86dc4c2e1a446fffab3678dca5e3"
summary = "Added GET /v1/usage/periods (bridle-api types UsagePeriods/UsageWeek/UsageDay/UsageWindow, client usage_periods, daemon route, pure fns in bridle-daemon/src/usage_periods.rs). Weeks and five-hour windows are distinct resets_at values from rate_limit_history; days are calendar days in [schedule] timezone clipped to the week. hit_limit = utilization >= 1.0 because the history keeps no status. Docs: api.md, usage-and-budget.md, CHANGELOG. No schema change."
ticket = "pemk"
+++

docs/tickets/open/daemon-usage-periods-the-week-from-the-seven-day-reset-its-d-pemk.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:37.864Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ask is the spec. Start: `rate_limit_history` (br-xxw9: store.rs, the /v1/usage/history route and RateLimitPoint in bridle-api types.rs). Add one additive route and types for the periods (week from the seven_day resets_at, its days, each real five_hour window with peak and limit-hit). Day boundaries: use the human's timezone from config if bridle has one, else UTC, and say which in the docs. Derive windows from `resets_at` changes, not fixed 5 h grids; "hit the limit" = status not allowed or utilization >= 1.0 (check what the readings carry). Pure functions over the readings with a table-driven test (two five-hour windows and a seven-day reset). Docs: api.md, usage-and-budget.md, CHANGELOG.md. No schema change. Out of scope: breakdowns (zn6s), gateway (v6cg).

### note · agent:pemk · 2026-10-14T00:16:52.231Z
built: ca92dd20 (branch bridle/pemk)

### note · agent:pemk · 2026-10-14T00:16:52.271Z
done: GET /v1/usage/periods (weeks, days, five-hour windows with peak and limit hit); just check exit 0, 1650 tests run (1650 passed, 6 skipped), run on the pre-merge commit; main merge brought only a ticket-file move; ca92dd20

### note · agent:manager-2 · 2026-10-14T00:49:14.370Z
integrated: d519f5f4be4e86dc4c2e1a446fffab3678dca5e3 (branch bridle/pemk)

### note · agent:manager-2 · 2026-10-14T00:49:56.404Z
cleanup: removed agent pemk, branch bridle/pemk

### note · external:advisor/product-manager · 2026-10-14T00:51:00.553Z
Not resolving yet (PdM): live history has 1476 distinct five_hour resets_at (about 34 real windows) and 955 seven_day (2 weeks); resets_at jitters by ms, so grouping by exact value fragments the periods. Fix filed as br-bm7a; pemk resolves when it lands.
