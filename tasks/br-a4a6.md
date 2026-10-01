+++
id = "br-a4a6"
title = "Interactive sessions C1: 'bridle agent wake <identifier>', the daemon decides when any agent wakes; first reason is a new message (jttf 2, phyy gap 3)"
kind = "feature"
state = "planned"
created_at = "2026-10-01T23:54:48.298Z"
updated_at = "2026-10-01T23:54:55.897242Z"
size = "M"
+++

original id: jttf
Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md (decision 2); phyy gap 3 in docs/tickets/open/a-life-assistant-agent-on-the-notes-repo-phyy.md. Code to read first: how 'bridle orchestrator wait-for-wake' is served (crates/bridle/src/commands/orchestrator.rs, the daemon's wake-reason logic in crates/bridle-daemon) and the inbox long-poll if one exists; wire types in crates/bridle-api/src/types.rs.

Goal, first slice: one command an interactive session runs, 'bridle agent wake <identifier>' (identifier = principal such as external:advisor or external:advisor/<name> once slice B lands; any existing principal name works), that blocks and returns when the DAEMON decides that principal should wake. The decision lives in one daemon function 'wake_reasons(principal)' so later reasons (task comment/state change, schedule, ticket comment) are added there, not in prompts. This slice implements ONE reason: the principal has an unread message. Output: the reason and the message ids (JSON with --json); a --timeout, exit code distinguishing timeout from woken. The caller's token must belong to that principal (or the human); otherwise 403.
SAFETY: do NOT change 'bridle orchestrator wait-for-wake' or its daemon route, supervision, or start-up; this is a new route beside them (merging them is a later, parked task). Lands normally.
Docs: cli.md, the agent-host API doc, CHANGELOG; a line in the advisor role/prime text telling advisors to loop on it ONLY if that is small and clearly correct, else leave it for a follow-up. Tests: wakes on a message sent after the call starts; returns at once when one is already unread; times out; wrong principal refused; no message for others doesn't wake it.
Acceptance: just check passes. Model: Sonnet.
Migration plan: none (a new command and route; no project files).
Out of scope: other wake reasons, schedules, wait-for-wake changes, restart.
