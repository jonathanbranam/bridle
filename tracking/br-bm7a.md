+++
id = "br-bm7a"
title = "Usage periods: group readings whose resets_at differ by milliseconds into one window or week (pemk splits them)"
kind = "fix"
state = "claimed"
created_at = "2026-10-14T00:50:42.002Z"
updated_at = "2026-10-14T03:54:45.595693Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:bm7a",
]
ticket = "bm7a"
+++

docs/tickets/open/usage-periods-group-readings-whose-resets-at-differ-by-milli-bm7a.md

## Thread

### note · agent:pm-1 · 2026-10-14T00:51:13.909Z
Brief (pm-1). Model: Sonnet. Size: S. The ask is the spec. Fix the grouping in crates/bridle-daemon (usage_periods.rs, landed in pemk d519f5f4): sort the readings by time and start a new period only when `resets_at` moves by more than a tolerance (name it as a constant, 10 minutes for five_hour; for seven_day the same constant is fine, well under a day), then report the period's `resets_at` as one value (the latest). Tests: jittered resets_at (millisecond differences, and a straddle of a minute edge) for both windows give one period each, and a real next window still starts a new one. Check against the live history on this machine (the daemon's /v1/usage/periods or `bridle usage`): about one window per five hours used and two weeks since 2026-10-07; put the counts in the done note. Docs: usage-and-budget.md if it states how windows are derived, CHANGELOG.md. No schema change. Out of scope: breakdowns (zn6s).
