+++
id = "br-1dfe"
title = "Enforce max_workers at spawn, and let it be scaled down live (y2eb)"
kind = "feature"
state = "dropped"
created_at = "2026-09-28T18:07:52.175Z"
updated_at = "2026-09-28T22:21:37.771324Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

ticket: docs/questions/open/enforce-max-workers-y2eb.md
original id: y2eb
see also: 6t29/br-37ed (budget presets incl. max_workers-per-period; depends on this task
landing first, per 6t29's own `needs: [y2eb]`), nbkj, mt7r

The human, verbatim (2026-09-28): "a ticket for actually controlling max workers is really
good; I'll have to scale the workers down if i need my laptop for literally anything." Their
laptop is "barely usable" when 2+ workers plus the manager/PM/orchestrator are all running.

What exists today: `[budget] max_workers` (default 2, `crates/bridle-daemon/src/config.rs`)
is read only in `Governor::maybe_resume` (`crates/bridle-daemon/src/governor.rs:480`), which
limits how many budget-paused agents resume at once. Nothing checks it when spawning a new
worker. The only cap on concurrent workers today is prose in
`workflow/base/roles/manager.md`: "One task per worker, at most two workers at a time" --
three workers ran concurrently on 2026-09-28, proving the prose isn't enough. Config is read
only at daemon start, so today changing `max_workers` needs a restart.

Brief: two changes.
1. Enforce `max_workers` at spawn time, not just resume: find wherever a new worker agent is
   spawned (likely `crates/bridle-daemon/src/supervisor.rs` and/or the `bridle spawn` command
   path) and refuse/queue the spawn once the number of running workers meets the cap. Match
   `maybe_resume`'s definition of "running" so the two enforcement points agree.
2. Let the human change the effective cap live, without a daemon restart. Governor already
   has exactly this shape for budget thresholds: `schedule_override` in
   `crates/bridle-daemon/src/governor.rs:70-201` is a `Mutex<Option<ScheduleOverrideState>>`
   set by `bridle budget override <name>` and read on the hot path instead of the static
   config value. Add an analogous live override for `max_workers` (a `Mutex<Option<u32>>` or
   similar), with its own CLI setter -- decide the least-surprising verb (e.g. extend
   `bridle budget override` to accept a worker count, or a separate
   `bridle budget set-max-workers <n>`) by looking at how the existing override command is
   structured in the CLI crate (`crates/bridle/src`) and wire types
   (`crates/bridle-api/src/types.rs`) before picking. The override is in-memory only, like
   the schedule override, and falls back to the config value when unset or after a restart.

Design decisions (settle these, don't leave them open):
- Lowering the cap never preempts or kills already-running workers; it only blocks new
  spawns/resumes until the running count drops back under the new cap. No preemption logic
  in this task.
- The cap counts worker agents only, not the manager, PM, or orchestrator roles -- matches
  today's prose ("two workers at a time") and keeps scope tight.

Acceptance: `just check` passes; a test that spawning a new worker is refused once running
workers meet `max_workers` (and that managers/PM/orchestrator don't count against it); a
test that `maybe_resume` and the new spawn-time check agree on what counts as "running"; a
test that the live override changes the effective cap without a restart and that it's lost
on restart (falls back to config); `docs/design/usage-and-budget.md` gets a section on the
max_workers live override, parallel to the existing "Schedule override" section.

Out of scope: budget presets or per-period `max_workers` values (br-37ed/6t29, a separate
task that depends on this one landing first); preempting or killing workers above a lowered
cap; anything about managers/PM/orchestrator counting toward a cap.

Model: Sonnet (touches the spawn path, a new live-override mechanism mirrored off an
existing one, and CLI/wire types together).

## Thread

### note · agent:pm-1 · 2026-09-28T22:21:37.771Z
dropped: Merged to main (00357de), incl. k7nr.
