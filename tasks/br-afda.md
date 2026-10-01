+++
id = "br-afda"
title = "Interactive sessions B: address a named advisor, external:advisor/<name>, with delivery fallbacks (jttf, advisors' inbox)"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T22:51:22.671Z"
updated_at = "2026-10-01T23:47:05.169218Z"
size = "M"
branch = "bridle/jttf-b"
commit = "cb85c8b25a4a81f9395050bd55e36784cd8eeda7"
summary = "Named advisors addressable as external:advisor/<name>[@machine]. Client::with_advisor adds x-bridle-advisor from BRIDLE_ADVISOR_NAME; the auth middleware signs the shared advisor token as external:advisor/<name>. resolve_targets routes to the session's own inbox if registered (not for @machine addresses), else to external:advisor with an '(originally for advisor/<name>)' body prefix (send prints the 'isn't running' line when to differs). Sessions end moves unread mail the same way (Store::move_unread_messages; Sessions now takes the Store). Caveats: ask --to doesn't add the mark; a named advisor's 'inbox' reads only its own inbox, not the shared one."
+++

original id: jttf
Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md, section 'The advisors inbox: decided' (read it all, including the delivery table). Design: docs/design/agent-host/principals.md and the messaging docs; code: message send/route in crates/bridle-daemon, the CLI's sender identity in crates/bridle (focus.rs line ~207 already reads BRIDLE_ADVISOR_NAME), wire types in crates/bridle-api/src/types.rs. Depends on slice A (the daemon's registry of advisor sessions).

Goal: implement the approved design as written: address 'external:advisor/<name>' (and '/<name>@<machine>' with the existing visitor suffix); a named advisor's CLI sends as from external:advisor/<name> using BRIDLE_ADVISOR_NAME (shared token, name is a label not proof); delivery per the table: running session gets it in its own inbox; ended or never existed goes to external:advisor marked 'originally for advisor/<name>' and the sender is told '<name> isn't running; delivered to advisor'; unread messages move to external:advisor with the same mark when the session ends; unknown part before '/' stays a 404.
Docs: principals.md/cli.md/storage.md as touched, CHANGELOG. Tests: one per table row, plus the sender attribution and replies returning to the named advisor.
SAFETY: messaging only; no change to daemon start-up or the orchestrator path. Lands normally.
Acceptance: just check passes. Model: Sonnet.
Migration plan: none (principal addresses are runtime; any DB column migrates itself).
Out of scope: the wake command (a later task), restart, orchestrator.

## Thread

### note · agent:jttf-b · 2026-10-01T23:43:31.796Z
done: named advisor addressing + delivery fallbacks; just check green (975 tests); 98f3397

### note · agent:manager-2 · 2026-10-01T23:43:35.067Z
Main moved (br-7e5a, br-b6e1 landed). Merge main into your branch, re-run just check, then message me the new sha. No other changes needed.

### note · agent:jttf-b · 2026-10-01T23:44:39.985Z
done: merged main (incl. br-b6e1), just check green (978 tests); ad4edce

### note · agent:manager-2 · 2026-10-01T23:46:54.966Z
integrated: cb85c8b25a4a81f9395050bd55e36784cd8eeda7 (branch bridle/jttf-b)

### note · agent:manager-2 · 2026-10-01T23:47:05.169Z
cleanup: removed agent jttf-b, branch bridle/jttf-b
