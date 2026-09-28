---
id: f1ky
title: Two flaky test failures observed under high machine load
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [n6gy]
---

## The question

Two test failures were observed while verifying main under load on a heavily
utilized machine, neither of which reproduced when the test was run again or
run in isolation. Are these real races, transient measurement errors under
load, or test flakes?

### Failure 1: spawn_messaging_test::spawn_with_a_crashing_first_message_returns_promptly

The test took 10.4 seconds to complete, exceeding its 5-second timing bound.
Machine load was around 9 at the time. When run alone in isolation, the same
test passed 5 consecutive times, each completing in approximately 1.5 seconds.

The test expects to detect an early process crash and return promptly from
`spawn`, but the 10.4s duration suggests the spawn may have missed the early
crash signal and instead waited out the full readiness timeout before
responding.

### Failure 2: cli_e2e::cli_end_to_end_against_a_foreground_daemon panic

The test panicked at `cli_e2e.rs` line 257 with the assertion message
"daemon.json should be removed on clean shutdown". Machine load was between 23
and 50 at the time of the failure. The test has not panicked again on
subsequent runs.

## Why it matters

Like the races documented in [[flaky-time-based-tests-on-a-loaded-machine-n6gy|the
previous flaky test investigation]], these failures appeared under load and
vanished on retry, making it difficult to distinguish real concurrency bugs
from transient measurement errors or scheduling artifacts. Recording them here
preserves the observation for future reproduction attempts and pattern matching
against similar failures.

## Status

Neither failure has reproduced. This ticket is a record to monitor, not a
confirmed bug to fix. If either recurs, it should be investigated with the
techniques that successfully reproduced the prior flaky test races (concurrent
load, isolated runs, and timing analysis).

### More failures of the same kind, 2026-09-27 (orchestrator)

Two more 5-second timeouts in `crates/bridle-claude/tests/process_test.rs`,
each once in a full `just check` and green on the rerun, with no change to
that crate in between:

- `close_stdin_with_no_turn_exits_zero` (panicked at `process_test.rs:206`,
  "exit timeout"), load average ~17, on `c312fad`.
- `events_channel_closes_at_stdout_eof` (panicked at `process_test.rs:312`),
  load average ~11, on `fc737e5`.

Both wait 5 s for the Python fake `claude` to exit or close stdout.

### Another, 2026-09-28 (orchestrator)

- `close_stdin_is_idempotent` (`process_test.rs`), 16.6 s, in a full
  `just check` at load average ~50, on `66ce86a`. The run before it on the
  same commit, at load ~18, was green.

## The human's decision, 2026-09-28

Scheduled right after P0-3. The human's words:

> yes, move the flaky tests up after P0-3; [...] the first task, the first
> effort is to do the easy thing and increase the timeouts as you suggested.

What the orchestrator suggested, which the first task takes up:

> **Safety-net timeouts:** raise them to 30–60 s through one shared helper.
> A passing test costs no time either way, and only a truly hung one waits.

The later steps the orchestrator proposed, not yet scheduled: check whether
`spawn_with_a_crashing_first_message_returns_promptly` hides a real missed
crash signal, and test the behaviour instead of wall-clock time; one shared
wait-until-condition helper for every test; reproduce the two unexplained
failures under load; no automatic retries, which would hide real bugs.

### The timeout fix, 2026-09-28

Merged as `cce2bec` and `3f1abe3`: the hang guards in `process_test`,
`restart_test`, `events_stream_test` and the daemon tests' support module are
now a shared 60 s constant. Left as they were: the `elapsed() < 5 s` speed
checks (`spawn_messaging_test.rs`, `lifecycle_test.rs`), and `cli_e2e.rs`'s two
10 s `wait_timeout_or_kill` shutdown waits, which sit beside the unexplained
"daemon.json should be removed on clean shutdown" failure above.
