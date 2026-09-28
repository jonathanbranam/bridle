---
id: c424
title: bridle budget: local times, the applied budget and the schedule
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The ask

The human, verbatim (2026-09-28):

> low pri easy ticket: make 'bridle budget' report either always in US Eastern, or use locale
> timezone

## What happens today

`bridle budget` prints reset and hold times as UTC, e.g.:

```
  five_hour        holding      87%, resets 2026-09-28 19:19 UTC
```

The formatting is hardcoded as `"%Y-%m-%d %H:%M UTC"` in `crates/bridle/src/commands.rs` @
65a64c4 (lines 1006, 1275, 1283, 1290; 1006 may be `bridle status`).

## Notes

- The standing rule `workflow/base/rules/human-timezone.md`: times shown to the human are
  US Eastern, written bare ("7:00 AM"), naming the zone only when it isn't Eastern; times
  recorded in bridle, git, tickets and logs stay UTC.
- The human offered either option: always Eastern, or the machine's local timezone.
- `chrono` is already a dependency of the `bridle` crate.
- Low priority, small.

## The human's answer

2026-09-28:

> fine resolve that question and use the local machine timezone

Decided: show times in the machine's local timezone. What's left is implementing it; the
ticket resolves when that lands.

## Also: show the applied budget and the schedule

The human, verbatim (2026-09-28, later):

> there should be a way to see the current applied budget and the next change, it's span,
> and settings; also an option to print the entire scheule out (make it easy so the user
> doesn't have to scan the various config files);

So, in local time as above:

- `bridle budget` shows which schedule entry applies now (its name, its span, and its
  `hold_at`/`wind_down_at`/`stop_at`), and when the next change happens and to what.
- An option (e.g. `bridle budget --schedule`) prints the whole resolved schedule, so the
  human doesn't have to read `~/.bridle/config.toml` and the project config to work it out.
