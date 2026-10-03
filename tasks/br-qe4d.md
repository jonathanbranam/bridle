+++
id = "br-qe4d"
title = "jttf part 2: 'bridle session restart <id> [--handover|--fresh]' and every interactive session listed in bridle status"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:20:11.151Z"
updated_at = "2026-10-03T21:20:15.302139Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Part 2 of 2 of br-jttf (read it and docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md, 'Decided' table). Build: (1) restart on request, with or without a handover: 'bridle session restart <identifier> [--handover|--fresh]'; restart without a handover is always the human's choice; relaunch in the tagged pane if bridle started the session, else tell the human. (2) 'bridle status' (and the gateway, if cheap) lists every interactive session: identifier, project, machine, context, uptime, last activity. Files: crates/bridle (CLI), crates/bridle-daemon, crates/bridle-api/src/types.rs. Docs: cli.md, orchestrator-supervision.md, CHANGELOG. Tests: restart variants, status listing. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: thresholds and warnings (part 1), crash restart, seats.
