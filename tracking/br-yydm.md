+++
id = "br-yydm"
title = "Gateway route for usage history: GET /projects/{project}/usage/history (gateway half of the week-to-week charts, xxw9)"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T21:06:46.794Z"
updated_at = "2026-10-12T23:10:24.568518Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
    "agent:wyydm",
]
branch = "bridle/wyydm"
commit = "f9eae4bf1f75527c96e434865c783a9d02675ae4"
summary = "Gateway half only: GET /api/v1/projects/{project}/usage/history?window=&since= (system.rs, lib.rs) proxies the daemon's /v1/usage/history as UsageHistory{project,window,points[utilization,resets_at,observed_at]}, oldest first; TS bindings added; human-web-ui.md and CHANGELOG updated. Tests: fake daemon returns time-ordered series and passes since; unknown project is 404. The page is bridle-ui ticket ui-u6uz."
ticket = "yydm"
+++

Ticket: docs/tickets/open/week-to-week-usage-charts-in-the-ui-how-often-the-five-hour-xxw9.md. Depends on br-xxw9 (the usage history API); start only after it merges.
Goal: a usage page in bridle-ui (via the gateway: crates/bridle-gateway plus the UI source it serves; follow where br-s6cj and the other UI pages live) with week-to-week charts: how often, and how close, the human gets to the five-hour limit and to the seven-day limit, week by week (e.g. per week: peak utilization, number of times at or above 90%/100%, a utilization-over-time line). Account-wide, not per project. Mobile-first like the other pages. Keep the chart code small and dependency-light.
Files likely: crates/bridle-gateway (proxy the history endpoint, bindings/*.ts), the UI source, docs for the UI.
Migration: none.
Acceptance: just check passes; the gateway endpoint is tested; the page renders from fixture history (describe how you checked it in the done note).
Model: Sonnet. Out of scope: per-project and per-agent token views (br-gztq's findings feed those).

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.016Z
watching the task

### note · agent:manager-2 · 2026-10-12T22:41:15.022Z
manager-2: not starting. It has no build brief, and the ask is two halves (gateway proxy of /v1/usage/history here; the bridle-ui page is to be filed as a separate ui- ticket). Please file the bridle-ui ticket and narrow this one to the gateway route with an acceptance check, or send a brief; I will then run it.

### note · agent:pm-1 · 2026-10-12T22:41:25.903Z
Narrowed to the GATEWAY HALF (pm-1, answering manager-2). Model: Sonnet. Size: S.

Goal: add `GET /projects/{project}/usage/history?window=&since=` to the gateway, proxying the daemon's `/v1/usage/history` (br-xxw9, integrated: RateLimitPoint / UsageHistoryQuery in crates/bridle-api/src/types.rs, client `usage_history`). Follow the existing per-project routes: crates/bridle-gateway/src/lib.rs (route table, ~line 67) and system.rs (`system_route`, its tests with a fake daemon, ~line 279-325). Export the TS binding like the other views (bindings/*.ts; see how RateLimitView is exported). Docs: the gateway API doc under docs/design/ (find where /projects/{project}/system is described), CHANGELOG.md.

Acceptance: a gateway test with a fake daemon returns a window's series in time order and passes `since` through; an unknown project errors like the other routes; `just check` passes. Migration: none.

Out of scope: the chart page itself. That is a bridle-ui change, in another project (existing-projects rule): the PdM files it as a ui- ticket against bridle-ui once this route exists. Per-project/per-agent views stay out (br-gztq).

### note · external:advisor/product-manager · 2026-10-12T22:42:12.151Z
From advisor (product-manager): narrowed per manager-2: this ticket is now the gateway route only (scope and acceptance in the body). The bridle-ui page is ui-u6uz, readied on bridle-ui, blocked by this one. pm-1: brief please.

### note · agent:wyydm · 2026-10-12T23:09:52.062Z
built: 6525d387 (branch bridle/wyydm)

### note · agent:wyydm · 2026-10-12T23:09:52.161Z
done: gateway usage/history route; just check exit 0, 1623 tests passed, sha 6525d387 (main merged, no new changes after the check)

### note · agent:manager-2 · 2026-10-12T23:09:57.249Z
integrated: f9eae4bf1f75527c96e434865c783a9d02675ae4 (branch bridle/wyydm)

### note · agent:manager-2 · 2026-10-12T23:10:24.568Z
cleanup: removed agent wyydm, branch bridle/wyydm
