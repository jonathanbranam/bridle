+++
id = "br-37ed"
title = "Budget presets with no schedule, and max_workers per period"
kind = "feature"
state = "dropped"
created_at = "2026-09-28T18:03:15.588Z"
updated_at = "2026-09-28T23:36:35.043602Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

ticket: docs/questions/open/budget-presets-and-max-workers-6t29.md
original id: 6t29
see also: y2eb/br-1dfe (enforce max_workers at spawn + live override -- this task needs that
one to land first, per 6t29's own `needs: [y2eb]`), n9qh, c424, nbkj, mt7r

The human, verbatim (2026-09-28): "can I create a new budget entry called 'burst' without a
days, start, end that I use? ... i think I want a 'low' also that would really throttle
bridle back for a period." And, after hearing the `max_workers` gap described: "I'd like
that yes, and also to override max_workers. That is important ... my laptop is barely usable
when bridle is cranking."

What exists (see docs/design/usage-and-budget.md, "Schedule override", and
`crates/bridle-daemon/src/config.rs` `RawSchedulePeriod`):
- Every `[[budget.schedule]]` period requires `days`, `start`, `end`. A period with
  `days = []` parses and never matches on its own, so it's reachable only through
  `bridle budget override <name>` -- an undocumented workaround, not a real no-schedule
  preset.
- `bridle budget override <name>` (governor.rs's `schedule_override`) forces a period's
  `five_hour` thresholds (`hold_at`/`wind_down_at`/`stop_at`). It does not touch
  `max_workers` at all today.
- `max_workers` is a single `[budget]` value with no per-period or override form.

Brief:
1. Let a `[[budget.schedule]]` period omit `days`/`start`/`end` and be a named preset usable
   only via `bridle budget override <name>` -- formalize today's `days = []` workaround into
   documented, validated support (a period missing a schedule can't accidentally match, and
   `bridle budget override` accepts it) rather than requiring the empty-list trick.
2. Give `bridle budget override` a way to also set `max_workers` for the override's
   duration, built on the live max_workers override mechanism from y2eb/br-1dfe (don't
   duplicate that mechanism -- this task's presets set the same override value, they don't
   invent a second one). Decide the config shape: most likely `max_workers` becomes an
   optional field on `[[budget.schedule]]` periods, applied through the override the same
   way the three threshold fields already are.

Acceptance: `just check` passes; a test that a schedule-less named period (e.g. "burst") is
never picked by the ordinary schedule-matching logic but works via
`bridle budget override burst`; a test that overriding a period with a `max_workers` value
changes the effective cap (using br-1dfe's live-override mechanism) and that it reverts when
the override ends or is cleared; docs/design/usage-and-budget.md updated for both the
schedule-less preset and the per-period `max_workers` override.

Out of scope: enforcing max_workers at spawn/resume or the live-override mechanism itself
(br-1dfe/y2eb, a prerequisite, not part of this task); any new CLI verbs beyond extending
`bridle budget override`'s accepted fields.

Model: Sonnet (config schema change, override-mechanism integration, CLI and docs together).

## Thread

### note · agent:pm-1 · 2026-09-28T23:36:35.043Z
dropped: Merged to main.
