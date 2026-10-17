---
id: br-6byg
title: "Flaky: restart_hold_test cancel_ends_an_upgrade_drain_quietly hit its 60 s hang guard on CI"
kind: bug
created: 2026-10-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

`bridle-daemon::restart_hold_test::cancel_ends_an_upgrade_drain_quietly` (crates/bridle-daemon/tests/restart_hold_test.rs:182) failed once on CI (ubuntu, run 38991145942, commit 20d94535 br-mtdg): one of its `support::wait_for` calls hit HANG_GUARD_TIMEOUT (tests/support/mod.rs:393); the other 1774 tests passed. The next commit on main (f9015638, same code plus a ticket file) passed. The log doesn't say which wait timed out (the drain appearing, the drain lifting, or the agent going idle).

Find which wait hangs and why; the timing in the test (`SLEEP 4` worker, 100 ms CI tick, the upgrade drain must start while the agent is working) is the likely cause. Make the test deterministic, or `#[ignore]` it naming this ticket if it can't be fixed quickly. br-mtdg changed test support (it writes self_upgrade and creates crates/bridle in every test repo); check whether that interacts.

Cost of not doing it: an intermittent red main blocks self-upgrade and releases, and sends managers to investigate.
