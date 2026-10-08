+++
id = "br-7h8e"
title = "Flaky test: upgrade_test a_long_drain_wakes_the_orchestrator_once sees no wake (fewer than one)"
kind = "bug"
state = "planned"
created_at = "2026-10-08T19:05:02.374Z"
updated_at = "2026-10-08T21:50:26.481360Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

From manager-2 (m-7196). Independent of br-7fr6 (thread cap): a land at load 2 still failed at 15:04 on this test, crates/bridle-daemon/tests/upgrade_test.rs:585 (`assert_eq!(seen.len(), 1)`, left 0 right 1: no upgrade_draining wake seen). It has also flaked for the focusnudge, psfork and roleclarify landings. br-7172 cannot land until it is fixed. Same kind of problem as br-5p3z (a race or short fixed wait in a test), so read that task's thread first.

The test (upgrade_test.rs:563-588): drain_wake_after = 1 s; spawns an agent running `SLEEP 5`, waits for Working, calls restart(upgrade), then loops `orch.orchestrator_wake(Some(1))` collecting reason "upgrade_draining" until `daemon.running.restart_requested()` turns true, then asserts exactly one wake.

Find the real cause before changing anything. Candidates to check in the code that raises the wake (grep drain_wake_after in crates/bridle-daemon/src):
1. The loop can exit (restart_requested true) before the wake reaches a poll, or the final poll returned empty while the wake was raised just after; the wake may be consumed by an earlier poll whose result is dropped on an error/timeout path.
2. The drain wake timer starts at a different point than the test assumes (e.g. after the agent exits, or only if still draining at the tick), so under a slow start the 5 s sleep ends before drain_wake_after + a tick.
3. The wake being delivered to another consumer.

Fix the cause: make the test wait for the wake itself (poll until one upgrade_draining wake is seen, with a generous overall timeout) rather than racing restart_requested, and fix the daemon if it genuinely can drop or miss the wake. Do not just raise sleeps. Keep the "exactly once" assertion: after seeing the first wake, keep polling until restart_requested and assert no second one.

Files: crates/bridle-daemon/tests/upgrade_test.rs, and the daemon source that raises the wake only if it has a real bug. CHANGELOG.md only if daemon behaviour changed.

Acceptance: just check passes; and the test passes 30 times in a row under load: `cargo nextest run -p bridle-daemon --test upgrade_test a_long_drain --stress-count 30` is not available, so use a shell loop of 30 runs of `cargo nextest run -p bridle-daemon -E 'test(a_long_drain_wakes)'` while another `just test` runs in a second terminal if you can; report how many passed before (on the unfixed code, to reproduce) and after. If you cannot reproduce it, say so and explain the fix by reasoning from the code.

Model: Sonnet. Migration: none. Out of scope: other flaky tests (list any you see on the thread), thread caps (br-7fr6), ps forks (br-9z2n).
