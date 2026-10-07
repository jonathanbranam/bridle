+++
id = "br-jttf"
title = "Supervise every interactive session like the orchestrator: context warnings every 50k from 150k, handover and restart on request (jttf)"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T21:19:06.442Z"
updated_at = "2026-10-03T22:51:25.144179Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/session-context"
commit = "c769c0dc76d85bc217a42f9885aa046b90c8629a"
summary = "Every interactive session (advisors, named advisors, triage) now gets context steps at [sessions] warn (default 150k/200k/250k/300k; per role in [sessions.advisor]/[sessions.triage]; config.rs SessionsConfig). sessions.rs: each step fires once (highest only on a jump, re-armed by a lower reading), emits session.context (+step), and sends a system note to the session and to external:triage (one note for triage itself), asking again each step. 'bridle session keep <id>' (POST /v1/sessions/keep, session.override event) records the human's override and tells the session; refused before step 0 and at the hard limit. At 300k the daemon runs 'bridle session restart <id>' (current_exe, in the repo; handover first, then --fresh if no note) and tells triage the outcome. Triage now registers as a session (identity 'triage'), reads a handover note at start, and relaunches via 'session triage'. Caveats: the restart is tested with a stub program, not a live tmux; the daemon's restart needs a resolvable human/daemon token from the repo cwd; per-role config covers advisor and triage only. Docs: orchestrator-supervision, roles-and-config, cli, CHANGELOG."
ticket = "jttf"
+++

Part 1 of 2 (part 2: the restart command and session listing, a separate task). Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md: 'Decided by the human' (2026-10-01) and 'Context warnings and a ceiling for every interactive session' with its 'Decided' table (2026-10-03). Human, 2026-10-03: 'I want to work to start pushing ahead with next steps to manage all RC sessions similar to orch.' (Stalled because its task br-b4ac was dropped in the k7tm sort.)
Built already: advisor session registration (POST /v1/sessions), session.context events and the session line in bridle status; the statusline writes every session's tokens to $BRIDLE_HOME/context/<session id>.
Goal: every interactive session bridle starts (advisors, named advisors, triage; the orchestrator keeps its own numbers) is supervised for context like the orchestrator. Thresholds: warn at 150k, 200k, 250k, 300k (configurable per role, e.g. [sessions] warn = ["150k","200k","250k","300k"]). Each warning goes to the session (a wake/message it reads) and to the human (through triage once br-r8kv lands; the human's inbox until then), and asks again at every step. At 200k the session plans a handover unless the human overrides; 250k is the normal ceiling (hand over or shut down unless the human overrides again); 300k is the hard limit: handover forced, session stopped and restarted (relaunched in its tagged pane if bridle started it, else the human is told). Handover = a short note in the repo the next session reads at start-up (like the orchestrator's).
Files: crates/bridle-daemon (supervisor, session registry), crates/bridle-api/src/types.rs if events change. Docs: orchestrator-supervision.md (or a new sessions section), roles-and-config.md, CHANGELOG. Tests: thresholds fire once per step, override recorded, hard limit forces. Acceptance: just check passes. Model: Sonnet. Migration: config keys optional with defaults; none. Out of scope: restart command and status listing (part 2), automatic crash restart for advisors (tabled), seats, replacing Remote Control (gcvj).

## Thread

### note · agent:pm-1 · 2026-10-03T21:20:15.415Z
Split: this is part 1 (thresholds); part 2 is br-qe4d (restart command, session listing). Part 1's hard-limit restart uses part 2's command, so br-jttf is blocked by br-qe4d.

### note · agent:session-context · 2026-10-03T22:50:14.770Z
done: [sessions] warn steps 150/200/250/300k for advisors+triage, warnings to session and triage, 'session keep' override, forced restart at 300k; just check green (1112 tests); 9f67724

### note · agent:manager-2 · 2026-10-03T22:50:20.179Z
integrated: c769c0dc76d85bc217a42f9885aa046b90c8629a (branch bridle/session-context)

### note · agent:manager-2 · 2026-10-03T22:51:25.144Z
cleanup: removed agent session-context, branch bridle/session-context
