+++
id = "br-h7gt"
title = "Flaky on Linux CI: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes times out"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T16:02:35.489Z"
updated_at = "2026-10-09T16:54:12.929066Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
priority = "critical"
priority_at = "2026-10-09T16:02:49.417144Z"
branch = "bridle/h7gt"
commit = "f5035c85b5ad5e7f730328a089003ef51d264744"
summary = "Flaky upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes made deterministic (crates/bridle-daemon/tests/upgrade_test.rs; test-only). Cause inferred, not reproduced: CI's ~69 s is the 8 s spawn-ready wait plus the 60 s wait_for, i.e. the drain began while the spawn was in flight, held the agent's first prompt (no init, so no turn), and the agent never read working. The test's build command now waits for a go file that the test writes only after it sees the agent working, so the drain cannot start before the turn. Looped 30 of 30 under 8 CPU-burners locally (macOS; the flake never reproduced locally before the change either). Caveat for a possible follow-up, not fixed: lifting a drain after a failed restart (perform_restart error path) does not deliver messages held during the drain, so a held prompt could stay stuck; unproven as the CI cause. The spawn-in-flight-vs-quiet-check race is no longer covered by this test."
ticket = "h7gt"
+++

docs/tickets/open/flaky-on-linux-ci-upgrade-test-self-upgrade-restarts-only-af-h7gt.md

## Thread

### note · external:orchestrator · 2026-10-09T16:02:49.417Z
priority: normal -> critical

### note · external:orchestrator · 2026-10-09T16:02:54.203Z
settle skipped by external:orchestrator: urgent fix: a flaky test broke CI on main (run 37954619068); critical per the human, 2026-10-03

### note · external:advisor/product-manager · 2026-10-09T16:23:16.753Z
watching the task

### note · external:orchestrator · 2026-10-09T16:30:01.121Z
From orchestrator: recurred on main ff9958b7 (br-6nzj, test daemons now poll at 200 ms), Linux only: CI run 37958674369, panicked at crates/bridle-daemon/tests/support/mod.rs:346 after 68.8s. Check whether 6nzj's polling change bears on the timeout. main is red until this lands.

### note · agent:h7gt · 2026-10-09T16:52:57.157Z
done: mid-turn upgrade test made deterministic (build waits for a go file written after the agent is seen working); looped 30 of 30 under CPU load; just check exit 0, 1386 tests; 065b9443. Cause inferred from CI timing, not reproduced; caveats in the summary.

### note · agent:manager-2 · 2026-10-09T16:53:38.294Z
integrated: f5035c85b5ad5e7f730328a089003ef51d264744 (branch bridle/h7gt)

### note · agent:h7gt · 2026-10-09T16:53:40.201Z
h7gt: CI run 37958674369 (ubuntu, ff9958b7) failed only on the same test, self_upgrade_restarts_only_after_the_mid_turn_agent_finishes, 68.8 s, the same wait_for panic (support/mod.rs:346, line shifted by 6nzj). It is the same flake, not a new failure; 8 s spawn-ready wait + 60 s wait_for as before. Fix 065b9443 (already contains ff9958b7) removes the race. Ready to land.

### note · agent:h7gt · 2026-10-09T16:53:42.624Z
br-h7gt: run 37958674369 is the same upgrade_test flake, only that test failed (see comment); tip 065b9443

### note · external:orchestrator · 2026-10-09T16:54:12.929Z
From orchestrator: read the diff (065b9443, test-only); approved to land. The old test already passed on a requested restart, so on CI neither the agent working nor the restart came in 60 s: a possible daemon liveness bug, now uncovered by this test. Filed as ticket b6mu (pending; not urgent once main is green).
