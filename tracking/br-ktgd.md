+++
id = "br-ktgd"
title = "Account usage for the dashboard: every project summed by role, name and instance, per week, day and five-hour window, with context per turn"
kind = "feature"
state = "held"
created_at = "2026-10-13T22:52:42.726Z"
updated_at = "2026-10-13T22:55:24.796369Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "ktgd"
held_by = "external:advisor/product-manager"
held_reason = "The human, ~7:15 PM ET: going through the ask step by step with advisor product-manager before it is built"
held_at = "2026-10-13T22:55:24.796369Z"
+++

docs/tickets/open/account-usage-for-the-dashboard-every-project-summed-by-role-ktgd.md

## Thread

### note · agent:pm-1 · 2026-10-13T22:53:43.082Z
Brief (pm-1). Model: Sonnet. Size: L: too big to hold in one context, so build in this order and STOP at the first boundary that gets long: tell the manager, who splits the remainder into a new ticket. Run after br-xfy7/br-njjt (landed) work; it builds on them.

Step 1 (S): context per turn. Add a `context_tokens` column to `turns` from what each turn already reports (never wake an agent), normal DB migration (docs/design/storage.md; old rows NULL, never zero; safe on every project's database). External sessions already have context in `interactive_usage`.

Step 2 (M): the per-project daemon aggregation. One daemon endpoint (reuse the br-xfy7 usage routes and shapes; additive types in crates/bridle-api/src/types.rs, clients and daemon together) that for a period gives, by role, by name (rolled up across sessions and instances) and by instance/session, hosted and external together: cost, tokens and context growth. Periods: the week from the seven_day reset (read `rate_limit_history.resets_at`, do not hardcode Wednesday 3 AM), each day of it, and each real five-hour window from `resets_at`. Include the five_hour and seven_day series with each window's peak and whether it hit the limit. Shares: each group's share of the week's cost as an ESTIMATE of its share of the seven_day percentage, labelled an estimate, plus the measured ratio (seven_day points per dollar).

Step 3 (M): the gateway sums every project it reaches (unreachable ones listed as unreachable, never zero) through one account-wide route. It takes over br-qhsa's merged route: find it first (git log --all --grep=qhsa; crates/bridle-gateway/src routes) and extend or replace it, keeping its callers working until ui-h79b (the UI half, bridle-ui's planner) switches; post the final route path and response shape on this thread as soon as step 2/3 fix them, because ui-h79b waits on it. Add the TS bindings the other views export.

Step 4 (S): `bridle usage` shows the same (`--week`, `--by name`), docs/design/usage-and-budget.md, storage.md, api.md, CHANGELOG.md.

Acceptance (from the ask): a test with two projects, a hosted agent and an external session across two five-hour windows gives the right per-window, per-day and per-week sums by role, name and instance, and the shares add up to the week's seven_day percentage; `just check` passes. Out of scope: cost per ticket, other machines' projects the gateway cannot reach, the UI.

### note · external:advisor/product-manager · 2026-10-13T22:55:24.796Z
held: The human, ~7:15 PM ET: going through the ask step by step with advisor product-manager before it is built
