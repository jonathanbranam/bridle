+++
id = "br-vabu"
title = "Flaky on Linux CI: store cancelled_blocking_task_returns_shutting_down_not_a_panic"
kind = "bug"
state = "planned"
created_at = "2026-10-09T17:20:34.959Z"
updated_at = "2026-10-09T17:36:28.764755Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
priority = "critical"
priority_at = "2026-10-09T17:20:35.089791Z"
summary = "The flaky store test only set worker_threads(1); the blocking pool stayed at 512 threads so the second spawn_blocking could finish before abort. The test now builds its runtime with worker_threads(1) and max_blocking_threads(1), so the task is truly queued when aborted. Test-only change in store.rs. Looped 30 of 30; just check exit 0, 1386 tests."
ticket = "vabu"
+++

docs/tickets/open/flaky-on-linux-ci-store-cancelled-blocking-task-returns-shut-vabu.md

## Thread

### note · external:orchestrator · 2026-10-09T17:20:35.089Z
priority: normal -> critical

### note · external:orchestrator · 2026-10-09T17:20:40.038Z
settle skipped by external:orchestrator: urgent fix: a flaky test broke CI on main (run 37962490045); critical per the human, 2026-10-03

### note · external:advisor/product-manager · 2026-10-09T17:20:57.952Z
watching the task

### note · agent:hua2 · 2026-10-09T17:36:28.764Z
done: store flake fixed (runtime built with max_blocking_threads(1), test-only); 30 of 30 loop; just check exit 0, 1386 tests; branch bridle/vabu tip a4f19163 (br-hua2 branch untouched)
