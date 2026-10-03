+++
id = "br-jttf"
title = "Supervise every interactive session like the orchestrator: context warnings every 50k from 150k, handover and restart on request (jttf)"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:19:06.442Z"
updated_at = "2026-10-03T21:20:15.415169Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

original id: jttf
Part 1 of 2 (part 2: the restart command and session listing, a separate task). Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md: 'Decided by the human' (2026-10-01) and 'Context warnings and a ceiling for every interactive session' with its 'Decided' table (2026-10-03). Human, 2026-10-03: 'I want to work to start pushing ahead with next steps to manage all RC sessions similar to orch.' (Stalled because its task br-b4ac was dropped in the k7tm sort.)
Built already: advisor session registration (POST /v1/sessions), session.context events and the session line in bridle status; the statusline writes every session's tokens to $BRIDLE_HOME/context/<session id>.
Goal: every interactive session bridle starts (advisors, named advisors, triage; the orchestrator keeps its own numbers) is supervised for context like the orchestrator. Thresholds: warn at 150k, 200k, 250k, 300k (configurable per role, e.g. [sessions] warn = ["150k","200k","250k","300k"]). Each warning goes to the session (a wake/message it reads) and to the human (through triage once br-r8kv lands; the human's inbox until then), and asks again at every step. At 200k the session plans a handover unless the human overrides; 250k is the normal ceiling (hand over or shut down unless the human overrides again); 300k is the hard limit: handover forced, session stopped and restarted (relaunched in its tagged pane if bridle started it, else the human is told). Handover = a short note in the repo the next session reads at start-up (like the orchestrator's).
Files: crates/bridle-daemon (supervisor, session registry), crates/bridle-api/src/types.rs if events change. Docs: orchestrator-supervision.md (or a new sessions section), roles-and-config.md, CHANGELOG. Tests: thresholds fire once per step, override recorded, hard limit forces. Acceptance: just check passes. Model: Sonnet. Migration: config keys optional with defaults; none. Out of scope: restart command and status listing (part 2), automatic crash restart for advisors (tabled), seats, replacing Remote Control (gcvj).

## Thread

### note · agent:pm-1 · 2026-10-03T21:20:15.415Z
Split: this is part 1 (thresholds); part 2 is br-qe4d (restart command, session listing). Part 1's hard-limit restart uses part 2's command, so br-jttf is blocked by br-qe4d.
