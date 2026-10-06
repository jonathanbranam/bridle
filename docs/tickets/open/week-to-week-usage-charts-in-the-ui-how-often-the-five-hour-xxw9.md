---
id: xxw9
title: "Week-to-week usage charts in the UI: how often the five-hour and seven-day limits are reached"
kind: feature
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [xypj, puaf]
tasks: [br-xxw9, br-yydm]
---

## The ask

The human, 2026-10-05 ~2:55 PM ET, verbatim (to the aide, by voice): "can you schedule a few tickets in for the UI to add some ability to to show like a week, you know, week to week usage? I'd like to see like. I'd like to see some charts about how often I go up to my five-hour limit, and then, like, how often I go up to my seven-day limit."

What's there today: `bridle usage --json` reports the current `rate_limits` (five_hour, seven_day: utilization, resets_at), but the store keeps only the latest reading per window (`rate_limits(window PK, ...)`, docs/design/storage.md), so there is no history to chart.

The ask:
1. bridle: keep a history of usage readings (each window's utilization and resets_at over time, for weeks), and serve it over the API.
2. bridle-ui: a usage page with week-to-week charts: how often, and how close, the human gets to the five-hour limit, and to the seven-day limit, week by week.

The account is one budget shared by every project daemon (xypj), so the history should be the account's, not one project's.
