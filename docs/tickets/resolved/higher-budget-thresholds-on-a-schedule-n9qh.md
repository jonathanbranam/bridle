---
id: n9qh
title: Higher budget thresholds on a schedule, while the human isn't using the account
opened: 2026-09-28
repos: [bridle]
changes: []
specs: [docs/design/usage-and-budget.md, docs/design/agent-host/roles-and-config.md]
needs: []
see: []
closed: 2026-09-30T05:12:44Z
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

## Resolution

Both parts have been implemented and merged into main:

**Part 1** (commit cdb4ed0): Schedule-based five-hour budget thresholds by
time of day. `[[budget.schedule]]` periods list named periods with days and
hours in the daemon's local timezone, each with their own `hold_at`,
`wind_down_at`, and `stop_at` thresholds for the `five_hour` window only.
Periods are walked in order and the first match (matching days and time range)
becomes the effective thresholds; no match falls back to plain `[budget]`
defaults. The schedule itself is never affected by the human's hold or
override — only the five-hour window respects it.

**Part 2** (commit aab6ee6): Thermostat-style bridle budget override over the
schedule. `bridle budget override <period-name|default> [--until HH:MM]` forces
a period's thresholds for the five-hour window (or the defaults if `default` is
named), bypassing the schedule entirely until the override expires. With no
`--until`, the override lasts until the schedule (unforced) would naturally
transition to a different period; `--until` sets an explicit end in the daemon's
local time, rolling to tomorrow if already past. `bridle budget override --clear`
cancels an active override immediately. Like the hold, an override is human-only
and shown in `bridle budget` output. Schedule periods live under `[budget]`, so
a project config may only lower a period's thresholds, never raise them.

Both are documented in `docs/design/usage-and-budget.md` (schedule at §195–239,
override at §240–275).

Resolved 2026-09-28.
