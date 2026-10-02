---
id: jqx3
title: Windows that cross midnight match on the current day, not the day they started
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq]
closed: 2026-10-02T00:43:36.330002Z
---

## The ask


The human, 2026-09-30, after adding a `[[focus]]` "sleep" period
(`days = ["sun", "mon", "tue", "wed", "thu"]`, `start = "21:30"`, `end = "06:00"`) and being
told it doesn't cover Thursday night into Friday but does cover Saturday night into Sunday:

> oh... IDK how that should work. that isn't very good for overnight blocks. Sure, I guess
> document the behavior clearly.

## What's there now

`in_window` (`crates/bridle-daemon/src/config.rs`, at 88cc95f) tests `days` against the
**current** weekday, then the time range (`t >= start || t < end` when it wraps). So for a
window that crosses midnight, the part after midnight belongs to the next day's entry. To cover
Sunday through Thursday nights you need two blocks: `sun..thu 21:30–00:00` and
`mon..fri 00:00–06:00`. `[[focus]]` and `[[budget.schedule]]` share `in_window`.

Both docs say only that a range "may cross midnight":

- `docs/design/agent-host/roles-and-config.md`, Focus hours: "host-local `HH:MM`, a range
  that may cross midnight"
- `docs/design/usage-and-budget.md`: "may cross midnight, e.g. `23:00..07:00`"

The human asked for the behaviour to be documented clearly, with the two-block example. They
didn't settle whether matching should change so a window belongs to the day it started.

Also in the same Focus hours section: the example's `mode` comment reads
`# default; "locked" parses but is not acted on yet`, and the `FocusMode` doc comment in
`config.rs` says the same. The section's **Locked** paragraph and `bridle_daemon::focus`
show that locked mode is built.

## Resolution

Resolved by: br-905b (ce396e6)
