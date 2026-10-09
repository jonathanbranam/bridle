+++
id = "br-h7gt"
title = "Flaky on Linux CI: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes times out"
kind = "bug"
state = "planned"
created_at = "2026-10-09T16:02:35.489Z"
updated_at = "2026-10-09T16:30:01.121502Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
priority = "critical"
priority_at = "2026-10-09T16:02:49.417144Z"
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
