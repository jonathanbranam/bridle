---
id: tr22
title: "Flaky on Linux CI: queue_nudge_test a_burst_is_one_message_after_it_settles sends 2 nudges, not 1"
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-tr22]
---

## What happened

CI run 37256470250 on main (f75c4e12, the br-e9yu landing) failed on ubuntu-latest only; macOS passed.
`bridle-daemon::queue_nudge_test a_burst_is_one_message_after_it_settles` panicked at
crates/bridle-daemon/tests/queue_nudge_test.rs:113: `left: 2, right: 1` (two queue nudges after
a burst of three queue changes 100 ms apart, debounce 1 s, settle 3 s). e9yu touches handover paths,
not the queue nudge, so this looks like a timing flake or a real debounce race, not e9yu.
The same week, `spawn_child_orphan_is_swept_on_stop` timed out once in a worker's check (br-vhss).

## Why it matters

Anything that breaks CI on main, a flaky test included, is critical (the human, 2026-10-03).
While main is red nothing else lands, and tonight's batch (m-0317) is waiting on it.

## The ask

- Find why a burst can yield two nudges (a debounce race in the daemon, or test timing on a slow
  runner). Fix the cause; only widen the test's margins if the daemon is right.
- Look at `spawn_child_orphan_is_swept_on_stop` too if it shares the cause; otherwise leave it.
