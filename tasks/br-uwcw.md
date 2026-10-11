+++
id = "br-uwcw"
title = "Flaky renew_test: resume_of_a_session_that_never_started_uses_a_fresh_session (macOS)"
kind = "fix"
state = "pending"
created_at = "2026-10-11T06:05:26.116Z"
updated_at = "2026-10-11T13:56:44.928743Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "low"
priority_at = "2026-10-11T06:05:38.502194Z"
ticket = "uwcw"
+++

docs/tickets/open/flaky-renew-test-resume-of-a-session-that-never-started-uses-uwcw.md

## Thread

### note · external:advisor/product-manager · 2026-10-11T06:05:38.502Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-11T06:05:38.549Z
advisor (product-manager): kept pending at low under the human's flaky-test rule (after the features). Goes to the human in the morning with the note that a daemon-side race could lose a resumed agent's conversation.

### note · external:advisor/product-manager · 2026-10-11T07:22:56.899Z
kind: bug -> fix

### note · external:orchestrator · 2026-10-11T13:56:44.928Z
orchestrator: failed again on macOS CI at 13:55Z (run 38144038514, sha 39e33fa0, same assertion at renew_test.rs:377). Second time today, so disabled per the human's flaky-test rule: #[ignore = "flaky on macOS CI, ticket uwcw"] in 9974ff89. Fixing it means removing that ignore.
