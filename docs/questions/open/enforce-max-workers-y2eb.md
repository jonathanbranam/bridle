---
id: y2eb
title: Enforce max_workers, and let the human scale it down live
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [6t29, nbkj, mt7r]
---

## The ask

The human, verbatim (2026-09-28):

> no, it's ok for now, but yes, a ticket for actually controlling max workers is really good;
> I'll have to scale the workers down if i need my laptop for literally anything.

Earlier the same day, from [[budget-presets-and-max-workers-6t29|6t29]]:

> my laptop is barely usable when bridle is cranking and there are only 2 works I think in
> addition to the manager and orchestrator.

## What happens today

- `[budget] max_workers` (default 2) is only read in `maybe_resume`
  (`crates/bridle-daemon/src/governor.rs` @ da9d1ec), to limit how many budget-paused
  agents resume at once. Nothing checks it at `bridle spawn` or `bridle resume`.
- The only cap on concurrent workers is prose in `workflow/base/roles/manager.md` @ da9d1ec:
  "One task per worker, at most two workers at a time." Three workers were `working` at
  once on 2026-09-28 (smaller-debug-builds, python-pack, branch-rules).
- Config is read only at daemon start, so changing `max_workers` needs a restart.

## Notes

- The human's need is to scale down on demand, when they need the laptop, without a
  restart. [[budget-presets-and-max-workers-6t29|6t29]] (presets that can set
  `max_workers`, applied by `bridle budget override`) needs this ticket first.
- Open: what happens to running workers above a lowered cap (let them finish, or wind the
  extras down), and whether managers, the PM and the orchestrator count toward it.
