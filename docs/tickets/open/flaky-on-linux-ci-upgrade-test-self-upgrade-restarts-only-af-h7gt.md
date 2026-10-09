---
id: h7gt
title: "Flaky on Linux CI: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes times out"
kind: bug
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [f4xu, 7h8e, 5p3z]
tasks: []
---

## The ask

CI run 37954619068 on fa22d829 (main, the br-f4xu merge), Linux job, 2026-10-09 15:56 UTC:
`bridle-daemon::upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes` panicked
at `crates/bridle-daemon/tests/support/mod.rs:335` with "timed out waiting for the agent to work or
the restart" after ~69 s. macOS passed on the same commit. Found by the orchestrator's `ci_failed` wake.

The first `wait_for` in the test ends when the agent reads `working` or `restart_requested()` is set.
Neither happened in 60 s: the agent (`worker("SLEEP 3")`) never read as working, and no restart was
requested. Either the 3 s turn began and ended between polls (unlikely at the poll rate), the spawn
never reached a turn, or the drain held the first prompt without the quiet check ever requesting the
restart. The test's own comment covers the drain-holds-the-prompt case only when the restart follows.

Not caused by br-f4xu: that changed only the fake's SIGTERM handler, and this test sends no signal
before the restart.

Third upgrade_test timing flake (br-5p3z, br-7h8e). Anything that breaks CI on main is critical
(the human, 2026-10-03).

Fix: find which state it was stuck in (log the agent's state and the drain flag on timeout) and make
the test, or the daemon's quiet check if it is a real bug, deterministic. Accept: the test passes
looped (e.g. 30 runs) under load on Linux, and `just check`.
