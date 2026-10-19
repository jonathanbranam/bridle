---
id: br-cgw3
title: "Each project's usage shares the whole account's weekly gain over its own spend (data-contracts: $0.10 = 60 points)"
kind: bug
created: 2026-10-19
created_by: external:advisor/usage
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-a65d, br-xypj]
---

## The ask

The human, 2026-10-19 (to advisor usage): "Please note any issues like this and investigate and file bugs to address them".

Every project daemon's `bridle usage --by name` shares the account's whole weekly gain out over that one project's spend. So each project claims all of it. On dalek, week of 2026-10-14 (60 points gained):

- bridle: $477, SHARE adds up to 60.
- data-contracts: $0.10 (one manager turn), SHARE `manager 60.0`, "583.45 points per dollar".
- dalek-home: $1.07, `manager 60.0`; track-web: $10.35, adds up to 61.

The seven_day window is account-wide, but `share_points` (crates/bridle-daemon/src/usage_groups.rs, from `GET /v1/usage/groups`) assumes the daemon's own spend is all of the account's. The bridle daemon's numbers are close only because bridle is 97% of the spend this week. In a small project they're nonsense, and "points per dollar", meant as a week-to-week check of the estimate, is wrong anywhere other than the biggest spender.

The ask: a project's share is never shown as the whole account's gain. Smallest fix: each daemon divides by the cost of every live daemon on the machine (the registry already lists them; `bridle usage` can sum their `/v1/usage/groups`), and says that other machines are not included (a65d). Or, failing that, the CLI and gateway label the column "share if this project were the only spender" and hide points per dollar. Found by advisor usage, 2026-10-19; it also wrote `scripts/usage-report.py`, which does the machine-wide sum for a one-off analysis.
