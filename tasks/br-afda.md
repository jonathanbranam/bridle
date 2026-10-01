+++
id = "br-afda"
title = "Interactive sessions B: address a named advisor, external:advisor/<name>, with delivery fallbacks (jttf, advisors' inbox)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T22:51:22.671Z"
updated_at = "2026-10-01T22:51:25.333269Z"
size = "M"
+++

original id: jttf
Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md, section 'The advisors inbox: decided' (read it all, including the delivery table). Design: docs/design/agent-host/principals.md and the messaging docs; code: message send/route in crates/bridle-daemon, the CLI's sender identity in crates/bridle (focus.rs line ~207 already reads BRIDLE_ADVISOR_NAME), wire types in crates/bridle-api/src/types.rs. Depends on slice A (the daemon's registry of advisor sessions).

Goal: implement the approved design as written: address 'external:advisor/<name>' (and '/<name>@<machine>' with the existing visitor suffix); a named advisor's CLI sends as from external:advisor/<name> using BRIDLE_ADVISOR_NAME (shared token, name is a label not proof); delivery per the table: running session gets it in its own inbox; ended or never existed goes to external:advisor marked 'originally for advisor/<name>' and the sender is told '<name> isn't running; delivered to advisor'; unread messages move to external:advisor with the same mark when the session ends; unknown part before '/' stays a 404.
Docs: principals.md/cli.md/storage.md as touched, CHANGELOG. Tests: one per table row, plus the sender attribution and replies returning to the named advisor.
SAFETY: messaging only; no change to daemon start-up or the orchestrator path. Lands normally.
Acceptance: just check passes. Model: Sonnet.
Migration plan: none (principal addresses are runtime; any DB column migrates itself).
Out of scope: the wake command (a later task), restart, orchestrator.
