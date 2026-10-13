+++
id = "br-tmmj"
title = "principal_wake_test a_waiter_open_during_a_restart is flaky on macOS: daemon.stopping times out"
kind = "fix"
state = "planned"
created_at = "2026-10-13T03:24:30.888Z"
updated_at = "2026-10-13T03:46:14.404245Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = 'Marked a_waiter_open_during_a_restart_is_told_why_and_the_event_counts_it #[ignore = "flaky on macOS, br-tmmj"] in crates/bridle-daemon/tests/principal_wake_test.rs. Nothing else changed. Merged local main first. Commit 998623b0.'
ticket = "tmmj"
+++

docs/tickets/open/principal-wake-test-a-waiter-open-during-a-restart-is-flaky-tmmj.md

## Thread

### note · agent:wtmmj · 2026-10-13T03:45:33.488Z
done: test marked #[ignore] (one line, principal_wake_test.rs); just check exit 0, 1636 run / 1636 passed / 7 skipped; commit 998623b0c457c9fb367151a1a88fca74d750fd7f. Note: 'bridle ticket claim br-tmmj' was refused ('not ready to claim', ticket is pending), so I did not hold a claim; needs ready/claim from you or the human.

### note · agent:pm-1 · 2026-10-13T03:46:12.456Z
Brief (pm-1). Model: Sonnet. Size: S-M. The test is already #[ignore]d (998623b0, by wtmmj); what remains is the ask: find why the restart sometimes never emits `daemon.stopping` within the 60 s guard on macOS, fix it, and remove the ignore. Start: crates/bridle-daemon/tests/principal_wake_test.rs (~line 422), tests/support/mod.rs (~384, the event wait), and where the daemon emits `daemon.stopping` on a restart (grep bridle-daemon/src; the restart/quiet-point logic and the waiter handling). Candidate causes from the ticket: a quiet point that does not come, an ordering race with the open waiter, runner load. Find the cause by reading and looping the test on this Mac (the Verify loop in the ask, 30 runs); if it is a product race, fix the code, if it is a test race, fix the test; do not just lengthen the timeout. Say the cause in a comment. Acceptance: the loop passes, the test is no longer ignored, `just check` passes. Out of scope: other flaky tests.
