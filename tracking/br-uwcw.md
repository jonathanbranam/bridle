+++
id = "br-uwcw"
title = "Flaky renew_test: resume_of_a_session_that_never_started_uses_a_fresh_session (macOS)"
kind = "fix"
state = "integrated"
created_at = "2026-10-11T06:05:26.116Z"
updated_at = "2026-10-17T11:37:13.919400Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wuwcw",
]
priority = "low"
priority_at = "2026-10-11T06:05:38.502194Z"
branch = "bridle/wuwcw"
commit = "54d04340bfd2c13a0b0e9bacc73cff7c88b1f8b1"
summary = "The race was only in the test, not the daemon: Idle is set when the process starts (supervisor.rs ~1341), before the first turn's system/init, where mark_session_started runs (~1472). The test's second wait_for_state(Idle) returned at once and stop --now could beat init, so the next resume rightly minted a fresh session (nothing to lose: no turn ever started). Test now polls agents.session_started in the DB; #[ignore] removed. 50 stress runs passed. No CHANGELOG (test-only)."
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

### note · external:orchestrator · 2026-10-11T21:32:11.293Z
orchestrator: not moot. The #[ignore] was the stopgap under the flaky-test rule; the human approved this ticket (via PdM, ~12:00 PM ET) to find whether the daemon really mints a fresh session id on a second resume (a resumed agent would lose its conversation), fix it if real, make the test deterministic and remove the #[ignore]. Slots are free and nothing else is startable, so start it now.

### note · external:orchestrator · 2026-10-11T21:32:11.314Z
From orchestrator: br-uwcw is not moot, comment on the ticket. Please start a worker on it now (slots are free).

### note · agent:wuwcw · 2026-10-11T21:51:15.894Z
built: 39cb128c (branch bridle/wuwcw)

### note · agent:wuwcw · 2026-10-11T21:51:17.730Z
done: race was test-only (Idle precedes first turn init; daemon durable fact is fine); test waits on session_started, ignore removed; just check exit 0, 1547 tests passed, 50/50 stress runs; 39cb128c

### note · agent:manager-2 · 2026-10-11T21:51:22.877Z
integrated: 54d04340bfd2c13a0b0e9bacc73cff7c88b1f8b1 (branch bridle/wuwcw)

### note · agent:manager-2 · 2026-10-11T21:51:36.349Z
cleanup: removed agent wuwcw, branch bridle/wuwcw

### note · external:advisor/product-manager · 2026-10-17T11:37:13.919Z
Verified by advisor (product-manager) 2026-10-17 ~8 AM ET: task integrated, landed on origin/main, every item of the ask done (or confirmed done by the human). Resolving.
