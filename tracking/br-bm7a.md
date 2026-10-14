+++
id = "br-bm7a"
title = "Usage periods: group readings whose resets_at differ by milliseconds into one window or week (pemk splits them)"
kind = "fix"
state = "integrated"
created_at = "2026-10-14T00:50:42.002Z"
updated_at = "2026-10-14T05:15:21.576174Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:bm7a",
]
branch = "bridle/bm7a"
commit = "67d98c53ff8f96afe1ac43e303e4528a84c41ed1"
summary = "usage_periods.rs by_reset now sorts readings by resets_at and starts a new period only when it moves more than RESET_TOLERANCE (10 min, separate from wake.rs's rule); a period reports its latest resets_at and peak. Test with ms jitter and a minute-edge straddle for both windows. Docs: usage-and-budget.md, CHANGELOG. Live history since 2026-10-07 (same algorithm applied to the daemon's raw readings; the running daemon is the old build): five_hour 35 windows (was 1510), seven_day 2 weeks (was 969)."
ticket = "bm7a"
+++

docs/tickets/open/usage-periods-group-readings-whose-resets-at-differ-by-milli-bm7a.md

## Thread

### note · agent:pm-1 · 2026-10-14T00:51:13.909Z
Brief (pm-1). Model: Sonnet. Size: S. The ask is the spec. Fix the grouping in crates/bridle-daemon (usage_periods.rs, landed in pemk d519f5f4): sort the readings by time and start a new period only when `resets_at` moves by more than a tolerance (name it as a constant, 10 minutes for five_hour; for seven_day the same constant is fine, well under a day), then report the period's `resets_at` as one value (the latest). Tests: jittered resets_at (millisecond differences, and a straddle of a minute edge) for both windows give one period each, and a real next window still starts a new one. Check against the live history on this machine (the daemon's /v1/usage/periods or `bridle usage`): about one window per five hours used and two weeks since 2026-10-07; put the counts in the done note. Docs: usage-and-budget.md if it states how windows are derived, CHANGELOG.md. No schema change. Out of scope: breakdowns (zn6s).

### note · agent:bm7a · 2026-10-14T04:31:01.228Z
built: 17e86228 (branch bridle/bm7a)

### note · agent:bm7a · 2026-10-14T04:31:01.295Z
done: jittered resets_at now group into one window/week (10 min tolerance); just check exit 0, 1657 tests passed; live history 35 windows / 2 weeks (was 1510 / 969); 17e86228

### note · agent:manager-2 · 2026-10-14T05:14:58.949Z
integrated: 67d98c53ff8f96afe1ac43e303e4528a84c41ed1 (branch bridle/bm7a)

### note · agent:manager-2 · 2026-10-14T05:15:21.576Z
cleanup: removed agent bm7a, branch bridle/bm7a
