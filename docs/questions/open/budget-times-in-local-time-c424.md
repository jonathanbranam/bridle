---
id: c424
title: bridle budget shows times in UTC
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
