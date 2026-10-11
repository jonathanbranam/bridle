+++
id = "br-3k7d"
title = "Externals (orchestrator, advisors, aides) can schedule messages for themselves"
kind = "bug"
state = "planned"
created_at = "2026-10-11T03:44:27.310Z"
updated_at = "2026-10-11T03:45:01.761949Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-11T03:44:28.901037Z"
ticket = "3k7d"
+++

Ticket: docs/tickets/open/externals-orchestrator-advisors-aides-can-schedule-messages-3k7d.md (read "What to build"; it is the spec). HIGH: the human wants it landed tonight, before the 3:00 AM ET idle-run cutoff. Small; start at once.

Goal: external principals (orchestrator, advisors, aides) can use bridle schedule add/list/rm for themselves. Today schedule_actor_ok in crates/bridle-daemon/src/server.rs (~line 1726) allows only Human | Agent and answers 403.

Build: an external follows the agent rule: add only with itself as the target (--to defaults to the caller; a named advisor is external:advisor/<name>, exactly that identity), list and rm only its own. The human keeps "any". Another principal as target gives 403. Update the timed-wake-up text in the advisor, aide and orchestrator roles under workflow/base/roles/ (where br-g5y2 added it for agents) so externals know they can. Remove the "today the daemon refuses the external roles" sentence from workflow/base/rules/scheduled-wakes.md. Docs: docs/design/agent-host/daemon.md (Scheduled messages, auth), api.md, cli.md, CHANGELOG.

Files: crates/bridle-daemon/src/server.rs (and wherever the schedule list/rm filter by owner), the three role files, scheduled-wakes.md, the docs above.
Migration: none (daemon behaviour and role text only; roles reach projects through the normal workflow sync); say so in the done note.
Acceptance: just check passes, plus tests: an external adds for itself (ok), for another principal (403), lists and removes only its own, the human still sees all.
Model: Sonnet. Out of scope: any change to what a schedule does, cbbn.

## Thread

### note · external:advisor/product-manager · 2026-10-11T03:44:28.901Z
priority: normal -> high

### note · external:orchestrator · 2026-10-11T03:45:01.761Z
From orchestrator: br-3k7d is urgent (the human, ~12:30 AM ET, via the PdM: 'get it done tonight'). Take the next free slot, ahead of other queued work; small (one auth match + role text). Land and push it before 2:30 AM ET, and tell me when it's pushed so I can upgrade the daemon onto it. From 3:40 AM nothing new starts (br-9d95 idle benchmark at 4:00).
