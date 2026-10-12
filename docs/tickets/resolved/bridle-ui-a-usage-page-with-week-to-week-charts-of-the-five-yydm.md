---
id: br-yydm
title: Gateway serves the account's usage history (/v1/usage/history) for the UI's limit charts (bridle half; UI is ui-u6uz)
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
resolved: 2026-10-12T23:10:39Z
---

## The ask

The UI side of br-xxw9. The human, 2026-10-05 ~2:55 PM ET (to the aide), in xxw9: "I'd like to see some charts about how often I go up to my five-hour limit" and the seven-day limit, week to week. Approved again 2026-10-12 ~6:35 PM ET (to advisor product-manager, item 3 of the usage epic br-rdkn): "Agree on 1-3. Let's do it".

The data exists: br-xxw9 (landed 2026-10-06) keeps the history of usage readings, served by the daemon at `GET /v1/usage/history` (CLI: `bridle usage --history five_hour|seven_day`). The account is one budget shared by every daemon, so the history is the account's.

## Scope (narrowed 2026-10-12, after manager-2's review)

This ticket is the **bridle half only: the gateway route**. The bridle-ui page is ui-u6uz (blocked by this one).

The gateway serves the account's usage history by proxying the daemon's `GET /v1/usage/history?window=&since=` (returns `Vec<RateLimitPoint>`; `crates/bridle-daemon/src/server.rs` `usage_history`), the way it proxies other per-project routes. The account is one budget across daemons, so one project's daemon is enough: per project under the usual project prefix; no merged-across-projects route (that's br-qhsa, not this).

Acceptance:
- A gateway test: the route returns the daemon's history for `window=five_hour` and `window=seven_day`, with `since` passed through.
- An unreachable daemon gives an error the UI can show as unreachable (as other proxied routes do), never an empty list.
- Read-only, same auth as the other gateway read routes; docs/design (the gateway's route list) and CHANGELOG updated.
