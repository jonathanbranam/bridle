---
id: n6gy
title: Do time-based test assertions need wider margins on a loaded machine?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [zk4p]
---

## The question

Some tests are flaky specifically when the machine is already busy — for
example, timing-sensitive budget-governor staleness tests
(`crates/bridle-daemon/src/governor.rs` and its tests) were observed to pass
or fail depending on how loaded the machine already was at the time.

Do time-based test assertions in this suite need wider margins, a way to run
them serially rather than concurrently with the rest of the suite, fake/
mocked time instead of wall-clock sleeps, or something else?

## Why it matters

Flaky tests that fail only under load erode confidence in `just check`
as a green/red signal (see
[[mergers-must-verify-checks-themselves-zk4p|should merging a worker's
branch require the merger's own passing test run]]), and make it hard to
tell a real regression from machine noise.

## Resolution

Reproduced by running several full `cargo nextest run --workspace` loops
concurrently in the background (2-3 of them, 20-40 iterations each) while
running `just check` (or a targeted `cargo nextest run -p bridle-daemon`
loop) in the foreground, on a 16-core box. At 2-3x concurrent full runs,
individual daemon-suite tests routinely went from 2-6s to 10-20s; at 5x
concurrent, some pushed past 40s. Three real races turned up this way, all
in test code, not the daemon itself; each is now fixed on
`bridle/deflake`:

1. **`bridle::cli_e2e::sigint_shuts_down_cleanly_with_a_store_call_in_flight`**
   — the test SIGKILLed the daemon 10s after SIGINT if it hadn't exited by
   then, but the daemon's own shutdown sequence allows up to `stop_grace`
   (30s default) + 5s to stop agents gracefully (`lib.rs`). Under load,
   stopping 8 agents plus the 8 sequential `spawn` CLI calls before it
   routinely took 15-40s. Fixed by raising the test's post-SIGINT wait to
   60s (comment cites the 35s the daemon allows itself, plus measured
   margin), rather than guessing a bigger fixed number — the polling loop
   underneath (`wait_timeout_or_kill`) was already correct, only the
   deadline was too tight for what the daemon is allowed to take.

2. **`governor_test::idle_agent_is_stopped_at_once_on_wind_down_then_resumed`**
   — the last assertion checked `has_message_containing(..., "pause is
   over")` once, immediately after polling the agent into a running state.
   But `Governor::maybe_resume` (`governor.rs`) calls `manager.resume(...)`
   (which flips the agent's state) before the separate `manager.send(...)`
   that writes the resume note, so there's a real window where the state
   has flipped but the message hasn't landed yet — normally sub-millisecond,
   wide enough to lose under load. Fixed by polling for the message with
   `wait_for`, matching the pattern the sibling assertion in the same file
   (`working_agent_is_notified_then_stopped_when_its_turn_ends`) already
   used for the same kind of check.

3. **`lifecycle_test::spawn_child_orphan_is_swept_on_stop`** — waited for
   `agent.turns >= 1` then asserted `state == Idle` in a separate step.
   `Store::end_turn` bumps `turns` before the supervisor flips the agent's
   state back to `Idle` (`supervisor.rs`'s `Result` handling), so the same
   kind of window as above let the assertion catch the agent still
   `Working`. Fixed by folding the state check into the wait predicate
   (`turns >= 1 && state == Idle`), which two other tests in the suite
   (`events_stream_test`, `spawn_messaging_test`) already did for the exact
   same reason — matched their style instead of introducing a new pattern.

None of the three needed a longer sleep-then-assert, mocked time, or
serializing tests: two were "poll for the real condition, not a proxy for
it" bugs in test helpers, and one was a deadline that didn't match what the
code it's timing out already promises to allow. `governor.rs`'s own
poll/staleness cadence was not implicated in any of the three; the ticket's
opening guess (staleness assertions) didn't pan out as the culprit once
reproduced.

Validated with 5 consecutive clean `just check` runs while a second full
`cargo nextest run --workspace` loop ran concurrently in the background
(and, in an earlier round before dialing back to avoid overloading the
shared machine, 5 consecutive clean runs under 2-3 concurrent full-workspace
loops, plus a 15-iteration focused loop on just `governor_test` +
`lifecycle_test` under 3x load that caught fix #3). No other flaky test
surfaced across roughly 60 stressed full-suite runs total.
