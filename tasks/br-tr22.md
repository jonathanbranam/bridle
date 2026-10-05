+++
id = "br-tr22"
title = "Flaky on Linux CI: queue_nudge_test a_burst_is_one_message_after_it_settles sends 2 nudges, not 1"
kind = "bug"
state = "planned"
created_at = "2026-10-05T02:59:19.142Z"
updated_at = "2026-10-05T02:59:42.819245Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
+++

original id: tr22
Ticket: docs/tickets/open/flaky-on-linux-ci-queue-nudge-test-a-burst-is-one-message-af-tr22.md (the ask; read it first). CRITICAL: main CI is red on this flake and nothing lands until it is fixed.

Goal: find why crates/bridle-daemon/tests/queue_nudge_test.rs `a_burst_is_one_message_after_it_settles` sees 2 nudges instead of 1 on ubuntu-latest (line 113; burst of 3 queue changes 100 ms apart, debounce 1 s, settle 3 s). Decide whether it is a real debounce race in the daemon's queue-nudge code or test timing on a slow runner. Fix the cause; widen the test's margins only if the daemon is right. Prefer making the test deterministic (e.g. paused/controlled time) over larger sleeps.
Also glance at `spawn_child_orphan_is_swept_on_stop` (timed out once, br-vhss) and fix it only if it shares the cause; otherwise leave it.

Files likely: crates/bridle-daemon/tests/queue_nudge_test.rs and the daemon's queue-nudge/debounce code (grep for the nudge in crates/bridle-daemon/src); docs/design/agent-host/ if behaviour changes.
Acceptance: just check passes; run the nudge test many times in a loop locally (e.g. 50x) with no failure; say in the done note what the cause was.
Model: Sonnet. Out of scope: other flaky tests, any queue redesign.

## Thread

### note · external:orchestrator · 2026-10-05T02:59:23.594Z
priority: normal -> high

### note · external:orchestrator · 2026-10-05T02:59:23.621Z
Critical (orchestrator): a CI-breaking flake on main; the human, 2026-10-03: anything that breaks CI on main, a flaky test included, is critical. Readied by orchestrator.
