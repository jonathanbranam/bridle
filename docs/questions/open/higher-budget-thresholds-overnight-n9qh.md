---
id: n9qh
title: Higher budget thresholds overnight, while the human isn't using the account
opened: 2026-09-28
repos: [bridle]
changes: []
specs: [docs/design/usage-and-budget.md, docs/design/agent-host/roles-and-config.md]
needs: []
see: []
---

## The ask

The human's words, 2026-09-28 (about 11:45 PM Eastern), after the governor
held at 81% of the five-hour window overnight:

> Budget can go up to 90% of the five hour window at night in the future
> since I'm not doing anything myself and don't need those tokens.

## What exists

The governor's thresholds are per window, with no time of day: `hold_at` 80,
`wind_down_at` 90, stop 95, `resume_below` 70 (`[budget]`,
`crates/bridle-daemon/src/config.rs`, `BudgetConfig::default`). They're read
from the machine-wide config, and a project config may only lower them.

## To decide

- Night hours. The human is asleep from about 11:00 PM to 7:00 AM Eastern.
  A configurable window in the human's timezone is enough.
- The other thresholds at night. Holding at 90 needs wind-down and stop above
  it, and the orchestrator's watcher wakes at five_hour ≥ 93%. For example:
  hold 90, wind down 93, stop 95.
- Only the five-hour window. The ask names it; seven_day pacing is unchanged.
