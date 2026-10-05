+++
id = "br-cyvf"
title = "Agent renewal hands over through the managed record: the outgoing agent writes it, the replacement gets it and its task (e9yu follow-up, qdw8)"
kind = "bug"
state = "planned"
created_at = "2026-10-05T00:23:35.618Z"
updated_at = "2026-10-05T05:05:56.567074Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
summary = """Context wind-down notice now tells the outgoing agent to run `bridle handover write --file -`. Renewal (supervisor.rs renew_inner) builds its continuation from renewal_lead_in: the agent's claimed task id (claims keyed agent:<name>) plus the newest handover note for that identity (id, show command, body), or "No handover note was written; continue from the task thread". Unit tests cover task+note and task+no note. CHANGELOG line added. Caveat: just check had 2 failures in untouched tests (sessions::the_hard_limit_has_no_override..., serve::login_warning_only_when_known_logged_out), both stalling on exec of freshly written stub scripts; they also fail alone."""
+++

Ticket: docs/tickets/open/per-project-sessions-aide-share-one-handover-file-and-one-id-e9yu.md (the human: "every agent in every project with the proper name has the right handover"); incident br-qdw8 (a renewed worker got no task and adopted the wrong one).

Goal: when the daemon renews a manager or worker for context, the handover goes through the managed record from br-e9yu, and the replacement gets it.
- The context governor's message to the outgoing agent (tick_context_check, crates/bridle-daemon/src/supervisor.rs) tells it to run `bridle handover write --file -` (no file, no vague "handoff note").
- The renewal continuation (supervisor.rs ~2822, send_continuation_note) names the agent's claimed task ID, if any, and gives the newest handover note for that agent's identity (the body, or the exact `bridle handover show <id>`). If no note was written, it says so and points at the task thread.
Acceptance: just check green; tests: a renewed worker with a claimed task and a note gets both in its continuation; one with no note gets the task and "no note".
Model: sonnet. Blocked by br-e9yu (needs its per-identity record).
Out of scope: anything br-e9yu does; the qdw8 polling/early-done fixes awaiting the human.

## Thread

### note · external:aide · 2026-10-05T00:23:51.293Z
watching the task

### note · agent:renewal-handover · 2026-10-05T04:39:48.442Z
br-cyvf code committed (55196f80, main merged) but just check is NOT green: 2 failures in untouched tests that stall on exec of freshly written stub scripts (see task comment); they fail alone too. Does main pass them on this machine? Land as-is or what do you want?

### note · agent:renewal-handover · 2026-10-05T04:39:51.003Z
just check failed (exit 100, 776/778 run passed): sessions::tests::the_hard_limit_has_no_override_and_forces_a_restart (stub calls log missing after 30s) and serve::tests::login_warning_only_when_known_logged_out (fake claude exec stalls past 60s). Neither touches my change; both fail run alone. Likely exec of freshly written scripts stalling on this Mac.

### note · agent:manager-2 · 2026-10-05T05:00:36.974Z
Landing check failed on a real test: bridle-daemon::renew_test renew_sends_a_continuation_note_with_no_pending_messages (crates/bridle-daemon/tests/renew_test.rs); it likely asserts the old continuation text your change replaced. Update it to the new text (or fix your code if the test is right), merge main, run just check once, report done with exit status, count, sha. Your 2 earlier stub failures did not recur in the landing check.

### note · agent:renewal-handover · 2026-10-05T05:05:56.567Z
done: renew_test updated to the new continuation text; just check exit 0, 1196 tests passed; 44fc2786 (main already merged)
