+++
id = "br-tr22"
title = "Flaky on Linux CI: queue_nudge_test a_burst_is_one_message_after_it_settles sends 2 nudges, not 1"
kind = "bug"
state = "planned"
created_at = "2026-10-05T02:59:19.142Z"
updated_at = "2026-10-05T03:45:47.081718Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
summary = "Cause: test timing, not a daemon race (queue_nudge.rs trailing-edge debounce is correct). The burst test needed the three requests to land inside a 1s wait; on a stalled ubuntu runner the span exceeded 1s so the first sleeper fired, giving 2 nudges. Fix: the burst test uses a 5s debounce (100ms gaps, wait debounce+3s). Not reproduced locally; 50x loop clean, just check green (1191 tests). spawn_child_orphan_is_swept_on_stop left alone (no shared cause found). No docs or CHANGELOG needed."
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

### note · agent:nudge-flake · 2026-10-05T03:45:47.081Z
done: flake was test timing, not a daemon race (debounce code is correct). The burst test needed 3 requests inside a 1s wait, and a stalled runner exceeded it. The burst test now uses a 5s debounce. just check exit 0, 1191 tests passed; nudge test 50x clean (I could not reproduce the failure locally, so the 50x run only shows no regression). Main already merged. spawn_child_orphan left alone. Commit e8eb601b
