+++
id = "br-8ff8"
title = "Flaky on CI: settle_wake_test blocked-task note races its edge setup"
kind = "bug"
state = "planned"
created_at = "2026-10-09T23:23:12.045Z"
updated_at = "2026-10-09T23:23:20.004947Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "critical"
priority_at = "2026-10-09T23:23:12.261517Z"
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
