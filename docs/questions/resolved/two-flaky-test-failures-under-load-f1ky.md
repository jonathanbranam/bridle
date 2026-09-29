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

### The promptness test, diagnosed, 2026-09-28 (orchestrator)

`spawn_with_a_crashing_first_message_returns_promptly` failed again (5.79 s
against its 5 s bound, load ~20, on `cfe3cea`); the next run was green.
`spawn` returns on the first of `system/init`, the process's exit, or
`SPAWN_READY_TIMEOUT` (8 s, `supervisor.rs`). The fake emits `system/init`
before it crashes, so `spawn` returns at init, before the crash. The 5 s bound
therefore measures only how fast the fake starts under load, and the 10.4 s
failure was the 8 s timeout plus overhead. The test's doc comment says the
failure "shows up in the spawn response itself", but nothing asserts on the
response, and with init arriving first it likely doesn't.

### Promptness diagnosis addressed, 2026-09-28 (housekeeping pass)

The diagnosis above has been acted on:

- `spawn_with_a_crashing_first_message_returns_promptly` no longer exists as
  of the test rename. It is now `spawn_with_a_crashing_first_message_reaches_crashed_state`
  (crates/bridle-daemon/tests/spawn_messaging_test.rs:147), which tests the
  Crashed end-state directly instead of enforcing a wall-clock timing bound.
- The "daemon.json should be removed on clean shutdown" panic is explained and
  fixed by the daemon-shutdown-race merge (commits 6d5d22b, 67489eb).

The other listed failures (process_test.rs timeouts, the wait-until-condition
helper idea) remain open and are still being monitored as per this ticket's
Status section.

### `renew_reapplies_the_spawn_s_extra_allowed_tools_and_env`, fixed, 2026-09-28

`crates/bridle-daemon/tests/renew_test.rs::renew_reapplies_the_spawn_s_extra_allowed_tools_and_env`
(added by the persist-spawn-overrides merge, `2d1588c`) hit the same 20 s
`wait_for` timeout once under load, green on rerun and 3/3 alone. Root cause
was structural, not a scheduling artifact: `fake-claude.py`'s
`FAKE_CLAUDE_ARGV_FILE`/`FAKE_CLAUDE_ENV_FILE` dump was a single file,
overwritten (mode `"w"`) by every `claude` invocation sharing an overridden
`claude_program` — including the daemon's own governor probes, which reuse
the test's `claude_program` override for their periodic usage/rate-limit
checks. A probe invocation landing right after the invocation under test
could permanently clobber its dump before the test read it.

Fixed by having each invocation write its own `<path>.<pid>` file instead of
sharing one, so no invocation can ever overwrite another's dump; a shared
`support::wait_for_dump` scans all per-pid files under a path for one
matching a predicate. Applied to every caller of `fake_claude_argv_dump_wrapper`,
`fake_claude_env_dump_wrapper` and `fake_claude_argv_and_env_dump_wrapper`
(`renew_test.rs`, `spawn_messaging_test.rs`), adding a `--name`/
`BRIDLE_AGENT_NAME` filter to the two `spawn_messaging_test` callers that
didn't already have one — they shared the same root cause but hadn't yet
been observed to flake on it.

### Shared wait helper and wall-clock asserts removed, 2026-09-29 (br-648a)

- Daemon tests: `support::wait_for` (poll a condition) is the one helper, now
  bounded by `HANG_GUARD_TIMEOUT` (60 s) instead of its own 20 s. The 5 s
  polling loop in `lifecycle_test` (open-file 409) uses it.
- `cli_e2e.rs`: `wait_until` (sync twin, 60 s guard) backs `wait_for_file`,
  the "first turn finished" polls (one `wait_for_first_turn` instead of two
  copies) and `wait_or_kill`, which replaces the two 10 s and one 60 s
  `wait_timeout_or_kill` shutdown waits.
- The `elapsed() < 5 s` / `< 1 s` asserts are gone from `lifecycle_test`,
  `spawn_messaging_test` and `process_test`; the calls run under the 60 s
  hang guard instead. `spawn_without_a_prompt_returns_promptly_and_stays_idle`
  therefore no longer detects a spawn that waits out `SPAWN_READY_TIMEOUT`
  (8 s); that can't be checked without a wall-clock bound.
- `bridle-claude`'s tests don't poll a condition (they wait on channels under
  the guard), so they need no polling helper. No automatic retries anywhere.

## Resolution

Nothing remains. The tests wait on conditions under one shared 60 s hang
guard (`crates/bridle-daemon/tests/support/mod.rs`, `crates/bridle/tests/cli_e2e.rs`);
no retries. Reopen if a new load flake appears.
