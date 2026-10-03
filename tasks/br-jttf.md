+++
id = "br-jttf"
title = "Supervise every interactive session like the orchestrator: context warnings every 50k from 150k, handover and restart on request (jttf)"
kind = "feature"
state = "open"
created_at = "2026-10-03T21:19:06.442Z"
updated_at = "2026-10-03T21:19:06.442Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

original id: jttf
Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md: "Decided by the human" (2026-10-01) and "Context warnings and a ceiling for every interactive session" with its "Decided" table (2026-10-03). The human, 2026-10-03: "I want to work to start pushing ahead with next steps to manage all RC sessions similar to orch. Idk where that has stalled." (It stalled because its task br-b4ac was dropped in the k7tm sort.)
Built already: advisor session registration (`POST /v1/sessions`), `session.context` events and the `session` line in `bridle status`; the statusline writes every session's tokens to $BRIDLE_HOME/context/<session id>.
Goal: every interactive session bridle starts (advisors, named advisors, triage; the orchestrator keeps its own numbers) is supervised for context like the orchestrator:
(1) Thresholds: warn at 150k, 200k, 250k, 300k (configurable per role, e.g. [sessions] warn = ["150k","200k","250k","300k"]). Each warning goes to the session (a wake/message it reads) and to the human (through triage once br-r8kv lands; the human's inbox until then), and asks again at every step. At 200k the session plans a handover unless the human overrides; 250k is the normal ceiling (hand over or shut down unless the human overrides again); 300k is the hard limit: handover forced, session stopped and restarted (relaunched in its tagged pane if bridle started it, else the human is told).
(2) Restart on request, with or without a handover: e.g. `bridle session restart <identifier> [--handover|--fresh]`; restart without a handover is always the human's choice. The handover is a short note in the repo the next session reads at start-up (like the orchestrator's).
(3) `bridle status` (and the gateway, if cheap) lists every interactive session: identifier, project, machine, context, uptime, last activity.
Not in scope: automatic crash restart for advisors (tabled by the human 2026-10-03), seats/one inbox per seat, replacing Remote Control (gcvj).
Docs: orchestrator-supervision.md (or a new sessions section), cli.md, roles-and-config.md, CHANGELOG. Tests: thresholds fire once per step, override recorded, hard limit forces. Acceptance: just check passes. Model: Sonnet. Migration: config keys optional with defaults; none.
