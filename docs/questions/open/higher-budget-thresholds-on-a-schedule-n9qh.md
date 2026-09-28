---
id: n9qh
title: Higher budget thresholds on a schedule, while the human isn't using the account
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

And, a few minutes later:

> Yes time of day is good. Could probably use more budget during week day
> work hours too because I'm not using my Claude much then either

And:

> And also there should be an override

Asked whether that meant switching periods by hand until a time, then back to
the schedule:

> Yes like a thermostat

## What exists

The governor's thresholds are per window, with no time of day: `hold_at` 80,
`wind_down_at` 90, stop 95, `resume_below` 70 (`[budget]`,
`crates/bridle-daemon/src/config.rs`, `BudgetConfig::default`). They're read
from the machine-wide config, and a project config may only lower them.

## To decide

- The schedule. Two periods so far: nights (the human is asleep from about
  11:00 PM to 7:00 AM Eastern) and weekday work hours (hours not given yet;
  9:00 AM to 5:00 PM Eastern is a guess). A list of periods with days and
  hours in the human's timezone, each with its own thresholds, is enough.
- The other thresholds in a period. Holding at 90 needs wind-down and stop above
  it, and the orchestrator's watcher wakes at five_hour ≥ 93%. For example:
  hold 90, wind down 93, stop 95.
- Only the five-hour window. The ask names it; seven_day pacing is unchanged.
- The override. The human can switch periods by hand without editing config
  or restarting: force a period's thresholds (or the defaults) now, until a
  given time, then fall back to the schedule. For example, `bridle budget
  override default --until 5pm` when they need their own Claude during a work
  period, or `bridle budget override night --until 11pm` to boost early.
  Human-only, like `bridle budget hold`, and shown in `bridle budget`.
- Thermostat semantics, then: with no `--until`, an override lasts until the
  next scheduled change (a thermostat's "hold until next"); `--until <time>`
  sets the end; `bridle budget override --clear` goes back to the schedule
  now. Whether a permanent override is needed is open (YAGNI: probably not).
