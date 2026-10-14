---
id: br-bm7a
title: "Usage periods: group readings whose resets_at differ by milliseconds into one window or week (pemk splits them)"
kind: fix
created: 2026-10-14
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-pemk]
parent: br-rdkn
theme: performance
resolved: 2026-10-14T05:15:34Z
---

Found verifying br-pemk (landed d519f5f4) against the live history. Epic br-rdkn; blocks the steps that read the periods (zn6s, v6cg).

## The problem

`usage_periods.rs` makes one period per distinct `resets_at`. The value Claude Code reports jitters by milliseconds from reading to reading (and drifts across a minute boundary), so on dalek's history since 2026-10-07:

- five_hour: 1476 distinct `resets_at` values, about 34 real windows (68 distinct to the minute: each window splits over a minute edge, e.g. 2026-10-13T22:49 and 22:50).
- seven_day: 955 distinct values, 2 real weeks (4 to the minute).

So GET /v1/usage/periods lists hundreds of "windows" and "weeks". The pemk test passed because its fixtures used exact times.

## The ask

Group readings into one period when their `resets_at` values are close (a tolerance well under the period, e.g. 10 minutes; say which in the code's comment), and report the period's `resets_at` as one value (the latest, or the median). Add a test with jittered `resets_at` (ms differences and a minute-edge straddle) for both windows. Check the result against the live history: about one window per five hours used, two weeks. `just check`. Size S.
