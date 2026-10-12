---
id: br-yydm
title: "bridle-ui: a usage page with week-to-week charts of the five-hour and seven-day limits (xxw9, UI side)"
kind: feature
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-xxw9, br-qhsa]
parent: br-rdkn
theme: performance
---

## The ask

The UI side of br-xxw9. The human, 2026-10-05 ~2:55 PM ET (to the aide), in xxw9: "I'd like to see some charts about how often I go up to my five-hour limit" and the seven-day limit, week to week. Approved again 2026-10-12 ~6:35 PM ET (to advisor product-manager, item 3 of the usage epic br-rdkn): "Agree on 1-3. Let's do it".

The data exists: br-xxw9 (landed 2026-10-06) keeps the history of usage readings, served by the daemon at `GET /v1/usage/history` (CLI: `bridle usage --history five_hour|seven_day`). The account is one budget shared by every daemon, so the history is the account's.

1. bridle: the gateway serves that history (it proxies no usage routes today; br-qhsa covers the rest of usage and is not part of this).
2. bridle-ui: a usage page with week-to-week charts: for each week, how often and how close the human came to the five-hour limit, and the seven-day utilization over the week. The planner files the bridle-ui half as a ui- ticket, as with br-gd43/ui-vnuu.
