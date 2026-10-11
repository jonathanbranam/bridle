+++
id = "br-uwcw"
title = "Flaky renew_test: resume_of_a_session_that_never_started_uses_a_fresh_session (macOS)"
kind = "fix"
state = "open"
created_at = "2026-10-11T06:05:26.116Z"
updated_at = "2026-10-11T17:18:58.710450Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "low"
priority_at = "2026-10-11T06:05:38.502194Z"
ticket = "uwcw"
+++

Ticket: docs/tickets/open/flaky-renew-test-resume-of-a-session-that-never-started-uses-uwcw.md (read it; human approved 2026-10-11). Low priority. The orchestrator disabled the test (#[ignore = "flaky on macOS CI, ticket uwcw"] in 9974ff89); fixing it means removing that ignore.

FIRST investigate whether the race is real for users: does the daemon lose "this session started a turn" when an agent is stopped right after Idle, so the second resume mints a new session id (the resumed agent would lose its conversation)? Read the turn-started record (p4ks, 0dc13324) and the resume path in crates/bridle-daemon (sessions/supervisor/store) against the failing assertion at crates/bridle-daemon/tests/renew_test.rs:377. If real, the fix is in the code: make the turn-started fact durable before Idle/stop can be observed, and say so on the task thread. If only the test races, make it wait on the stored fact rather than on Idle. Either way remove the #[ignore] and run the test repeatedly under load (loop it 50 times with cargo nextest, one plain command per call) to show it stable.
Files: crates/bridle-daemon (the code if real), crates/bridle-daemon/tests/renew_test.rs, CHANGELOG. Migration: none.
Acceptance: just check passes with the ignore removed; the repeated runs pass; the done note says whether the race was real. Model: Sonnet. Out of scope: other flaky tests.

## Thread

### note · external:advisor/product-manager · 2026-10-11T06:05:38.502Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-11T06:05:38.549Z
advisor (product-manager): kept pending at low under the human's flaky-test rule (after the features). Goes to the human in the morning with the note that a daemon-side race could lose a resumed agent's conversation.

### note · external:advisor/product-manager · 2026-10-11T07:22:56.899Z
kind: bug -> fix

### note · external:orchestrator · 2026-10-11T13:56:44.928Z
orchestrator: failed again on macOS CI at 13:55Z (run 38144038514, sha 39e33fa0, same assertion at renew_test.rs:377). Second time today, so disabled per the human's flaky-test rule: #[ignore = "flaky on macOS CI, ticket uwcw"] in 9974ff89. Fixing it means removing that ignore.

### note · external:advisor/product-manager · 2026-10-11T17:18:05.421Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied, low. First check whether the daemon really mints a new session id on the second resume (a resumed agent would lose its conversation); if so, that's the fix, not just the test.
