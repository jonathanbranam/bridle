+++
id = "br-mb8p"
title = "Fix the macOS flake in principal_wake_test a_waiter_open_during_a_restart (carries br-tmmj)"
kind = "fix"
state = "planned"
created_at = "2026-10-13T13:07:23.476Z"
updated_at = "2026-10-13T13:07:40.525804Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "mb8p"
+++

docs/tickets/open/fix-the-macos-flake-in-principal-wake-test-a-waiter-open-dur-mb8p.md

## Thread

### note · agent:pm-1 · 2026-10-13T13:07:40.427Z
Brief (pm-1), carried from br-tmmj. Model: Sonnet. Size: S-M. The test is already #[ignore]d (4849f58b); what remains is to find why the restart sometimes never emits `daemon.stopping` within the 60 s guard on macOS, fix it, and remove the ignore. Start: crates/bridle-daemon/tests/principal_wake_test.rs (~line 422), tests/support/mod.rs (~384, the event wait), and where the daemon emits `daemon.stopping` on a restart (grep bridle-daemon/src; the restart/quiet-point logic and the waiter handling). Candidate causes: a quiet point that does not come, an ordering race with the open waiter, runner load. Find the cause by reading and looping the test on this Mac (`cargo nextest run -p bridle-daemon --test principal_wake_test -- a_waiter_open_during_a_restart`, 30 runs); a product race gets a code fix, a test race a test fix; do not just lengthen the timeout. Say the cause in a comment. Acceptance: the loop passes, the test is no longer ignored, `just check` passes. Out of scope: other flaky tests.
