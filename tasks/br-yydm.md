+++
id = "br-yydm"
title = "bridle-ui: a usage page with week-to-week charts of the five-hour and seven-day limits (xxw9, UI side)"
kind = "feature"
state = "pending"
created_at = "2026-10-05T21:06:46.794Z"
updated_at = "2026-10-06T00:45:01.174107Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

original id: xxw9
Ticket: docs/tickets/open/week-to-week-usage-charts-in-the-ui-how-often-the-five-hour-xxw9.md. Depends on br-xxw9 (the usage history API); start only after it merges.
Goal: a usage page in bridle-ui (via the gateway: crates/bridle-gateway plus the UI source it serves; follow where br-s6cj and the other UI pages live) with week-to-week charts: how often, and how close, the human gets to the five-hour limit and to the seven-day limit, week by week (e.g. per week: peak utilization, number of times at or above 90%/100%, a utilization-over-time line). Account-wide, not per project. Mobile-first like the other pages. Keep the chart code small and dependency-light.
Files likely: crates/bridle-gateway (proxy the history endpoint, bindings/*.ts), the UI source, docs for the UI.
Migration: none.
Acceptance: just check passes; the gateway endpoint is tested; the page renders from fixture history (describe how you checked it in the done note).
Model: Sonnet. Out of scope: per-project and per-agent token views (br-gztq's findings feed those).
