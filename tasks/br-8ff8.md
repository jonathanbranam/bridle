+++
id = "br-8ff8"
title = "Flaky on CI: settle_wake_test blocked-task note races its edge setup"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T23:23:12.045Z"
updated_at = "2026-10-10T02:02:09.319945Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "critical"
priority_at = "2026-10-09T23:23:12.261517Z"
branch = "bridle/w8ff8"
commit = "d0869bab343e1c27a3e4188b2c80e6df8d8cb90a"
summary = "Fixed the flaky settle_wake_test::a_task_blocked_by_a_dependency_makes_no_note (ticket 8ff8). Test-only change in crates/bridle-daemon/tests/settle_wake_test.rs: the blocked-task case now uses a 5 s settle (was 1 s) and sleeps 6500 ms (was 2500 ms). Reason: the settle clock starts at task creation (tasks.rs settle_until), so the blocks edge must land before the settle expires; planning two tasks plus the edge can outrun 1 s on a slow runner. Assertion unchanged. The other tests in the file have no edge-setup race and are untouched. Diagnosis confirmed; no product code changed. Verified: the blocked test passed 20/20 consecutive runs; just check green on the merged tree (1432 passed, 5 skipped). Earlier red runs were timing tests failing under machine load 120-135 (session_test a_session_can_restart_itself, governor_test working_agent_is_notified_then_stopped_when_its_turn_ends), not changed by this work. Commit 5c81b767 message still says NOT YET GREEN: stale now that the check passed. Merged main (41d6e0fd, br-7ufd)."
ticket = "8ff8"
+++

docs/tickets/open/flaky-on-ci-settle-wake-test-blocked-task-note-races-its-edg-8ff8.md

## Thread

### note · external:orchestrator · 2026-10-09T23:23:12.261Z
priority: normal -> critical

### note · external:orchestrator · 2026-10-09T23:23:12.330Z
From orchestrator: main: the macOS orphan-test failure on b798c8bc passed on re-run (one-off). But d9f8505b went red on Linux from a second flake, settle_wake_test (race between the 1 s settle and the edge setup; diagnosis in ticket 8ff8). br-8ff8 is critical: next free slot, Haiku, test-only. Later main runs are green, so merges may resume.

### note · external:orchestrator · 2026-10-09T23:23:12.580Z
From orchestrator: br-8ff8 (critical flake fix, ticket 8ff8) is ready; plan it at the top. Brief is in the ticket.

### note · agent:w8ff8 · 2026-10-10T01:44:04.300Z
done: settle_wake blocked-task test fixed (test-only, 5 s settle); just check green on merged tree (exit 0, 1432 passed, 5 skipped); blocked test 20/20; main merged (41d6e0fd); commit 5c81b767. Note: that commit's message says NOT YET GREEN, which is stale now; say if you want it amended.

### note · agent:manager-2 · 2026-10-10T01:57:55.923Z
main moved; merge main into bridle/w8ff8 (docs-only? then no re-check), message me the tip at once.

### note · agent:w8ff8 · 2026-10-10T01:58:07.948Z
tip fcd9f4f8 (merge of main; main's new commits are docs, tickets and a .bridle role note only, no code, so no re-check). Last green check: 5c81b767.

### note · agent:manager-2 · 2026-10-10T02:02:09.319Z
integrated: d0869bab343e1c27a3e4188b2c80e6df8d8cb90a (branch bridle/w8ff8)
