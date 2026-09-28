---
id: 6t29
title: Budget presets with no schedule, and max_workers per period
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: [y2eb]
see: [n9qh, c424, nbkj, mt7r]
---

## The ask

The human, verbatim (2026-09-28):

> ok, ok, so can I create a new bduget entry called "burst" without a days, start, end that I
> use? Otherwise, I can't override the budget to something different; like I think I want a
> "low" also that would really throttel bridle back for a period.
>
> that sounds handy, or the CLI needs to accept all fo the options as arguments but the
> presets are nice; i'm concerned it will reject them if they don't have an actual schedule.

and, after the advisor described the workaround below:

> oh interesting; I'd like that yes, and also to override max_workers. That is important, idk
> but my laptop is barely usable when bridle is cranking and there are only 2 works I think in
> addition to the manager and orchestrator.

## What exists

From [[docs/design/usage-and-budget|usage and budget]] @ 873af46, `### Schedule override`:

> `<period-name>` forces that `[[budget.schedule]]` period's
> `hold_at`/`wind_down_at`/`stop_at` for `five_hour`, whether or not the
> schedule itself would currently pick it.

- `days`, `start` and `end` are required on every `[[budget.schedule]]` period
  (`RawSchedulePeriod` in `crates/bridle-daemon/src/config.rs` @ 873af46).
- **Workaround today:** `days = []` parses and never matches, so a period with it is only
  reachable through `bridle budget override <name>`. Undocumented.
- **`max_workers` doesn't cap spawns.** The daemon only uses it in `maybe_resume`
  (`crates/bridle-daemon/src/governor.rs` @ b874b0f), to limit how many budget-paused agents
  resume at once. The only cap on concurrent workers is prose in the manager's role prompt,
  `workflow/base/roles/manager.md`: "One task per worker, at most two workers at a time."
  Three workers were `working` at once on 2026-09-28 (smaller-debug-builds, python-pack,
  branch-rules). So the human's `max_workers` ask needs the daemon to enforce it at spawn
  (and resume) first.
- A period sets only the three `five_hour` thresholds. `max_workers` is a single `[budget]`
  value (default 2), with no per-period or override form.

## Notes

- The laptop being "barely usable" with 2 workers plus the manager, PM and orchestrator may be
  mostly build and test load (each worker's worktree compiles and runs the full suite) rather
  than `claude` itself; not measured. Related: [[smaller-debug-builds-nbkj|nbkj]],
  [[run-only-the-tests-a-change-can-affect-mt7r|mt7r]].
- Config is read only at daemon start, so today any change needs a restart.
