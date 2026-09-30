---
id: k7nr
title: After a budget pause, the governor may never resume the manager
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [c70b971]
specs: []
needs: []
see: [y2eb, 6t29, incidents]
---

## What happened

On 2026-09-28 a wind-down stopped pm-1 (18:14:49), python-pack-2 and manager-2 (18:14:59),
all with `budget_paused`. When the five_hour window reset, at 19:20:36, the governor resumed
pm-1 and python-pack-2 only. manager-2 stayed stopped until the orchestrator resumed it by
hand at 21:27. By 19:27 both resumed agents had finished and gone idle. python-pack-2's
"done" report (m-1084) to manager-2 sat undelivered, so bridle did nothing for two hours.
(The orchestrator's session was down too; see `docs/context/incidents.md`.) The human:
"really, bridle should keep running without an orchestrator".

## Why

`maybe_resume` (`crates/bridle-daemon/src/governor.rs:480` @ d2eac43):

- It sorts paused agents oldest first and resumes `max_workers - running` of them (2 here).
  manager-2 was paused last, so it lost out.
- It counts every agent, managers included, against `max_workers`.
- `running` counts live processes, idle ones included. So once pm-1 and python-pack-2 were
  back, even though idle, `slots` stayed 0 on every later tick, and manager-2 was never
  resumed.

## Fix

- Resume managers (any role with `resume_on_restart`, or any non-worker) unconditionally;
  `max_workers` limits workers only.
- Among workers, count `working` agents or workers only, not idle managers. y2eb (br-1dfe) is
  redefining what `max_workers` counts at spawn and resume, so the two must agree: do this
  with y2eb, or straight after it.
- A test: three agents paused, `max_workers = 2`, a manager among them paused last; after
  recovery the manager is resumed.

## Resolution

Fixed in c70b971 (y2eb, k7nr): `maybe_resume` in `crates/bridle-daemon/src/governor.rs` resumes
every non-worker role unconditionally and applies `max_workers` to workers only, counting
running workers. Documented in `docs/design/usage-and-budget.md` (Resuming). Test:
`resume_brings_back_a_manager_even_when_workers_fill_max_workers`.
