---
id: br-pemk
title: "Daemon usage periods: the week from the seven-day reset, its days, and each real five-hour window with peak and limit hit (ktgd step 2a)"
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ktgd]
theme: performance
parent: br-rdkn
resolved: 2026-10-14T05:15:34Z
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

One daemon route (additive to the br-xfy7 usage types in bridle-api types.rs) listing the periods: the week from the seven_day reset (read `rate_limit_history.resets_at`, never hardcode Wednesday 3 AM), each day of it, and each real five-hour window from `resets_at`, with each window's peak utilization and whether it hit the limit, plus the five_hour and seven_day series. This alone answers "did we hit a five-hour limit last week?".

Acceptance: a test with readings across two five-hour windows and a seven-day reset gives the right windows, peaks and hits; `just check`. Docs: api.md, usage-and-budget.md. Size S-M.
