---
id: br-tmmj
title: "principal_wake_test a_waiter_open_during_a_restart is flaky on macOS: daemon.stopping times out"
kind: fix
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-13T13:47:55Z
---

## The ask

`bridle-daemon::principal_wake_test::a_waiter_open_during_a_restart_is_told_why_and_the_event_counts_it` (`crates/bridle-daemon/tests/principal_wake_test.rs:422`) is flaky on CI's macOS runner.

Seen 2026-10-13 03:23Z, run 38340405253 (check, macos-latest) on 6111b0b7, a docs-only commit: `timed out waiting for event daemon.stopping` (`tests/support/mod.rs:384`, the 60 s hang guard). The run before it (38340375954, d1fe554a, the same code) passed, and Linux passed both.

Per the human (2026-10-11), it's marked `#[ignore]` naming this ticket until fixed. Find why the restart sometimes never emits `daemon.stopping` within the guard on macOS (a quiet point that doesn't come, an ordering race with the open waiter, or runner load), fix it, and un-ignore.

Verify: the test passes in a loop (`cargo nextest run -p bridle-daemon --test principal_wake_test -- a_waiter_open_during_a_restart` 30 times, or under `stress`) on macOS, and is no longer ignored.
