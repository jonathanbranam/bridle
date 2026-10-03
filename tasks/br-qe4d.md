+++
id = "br-qe4d"
title = "jttf part 2: 'bridle session restart <id> [--handover|--fresh]' and every interactive session listed in bridle status"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T21:20:11.151Z"
updated_at = "2026-10-03T22:34:25.119226Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/session-restart"
commit = "7f2d779fe151220ff6a421440908b04a3458a0ef"
summary = "Added 'bridle session restart <id> [--handover|--fresh]' (session.rs): finds the registered advisor, --handover (default) messages it to write $BRIDLE_HOME/handover/<identity>.md and waits (10 min), --fresh restarts at once and is refused inside a session; then SIGTERMs the launcher's children and send-keys 'bridle [--project p] session advisor [name]' into the registered pane (prints the command if no pane); the new launcher reads the note (renamed .read). SessionInfo/SessionRegister gained project, machine, last_activity (context file mtime); bridle status session lines show them. Caveats: orchestrator isn't listed or restartable this way (it has its own handover); gateway listing not done; no end-to-end test of the tmux relaunch, only unit tests of the pieces. Docs: cli.md, orchestrator-supervision.md, CHANGELOG."
+++

Part 2 of 2 of br-jttf (read it and docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md, 'Decided' table). Build: (1) restart on request, with or without a handover: 'bridle session restart <identifier> [--handover|--fresh]'; restart without a handover is always the human's choice; relaunch in the tagged pane if bridle started the session, else tell the human. (2) 'bridle status' (and the gateway, if cheap) lists every interactive session: identifier, project, machine, context, uptime, last activity. Files: crates/bridle (CLI), crates/bridle-daemon, crates/bridle-api/src/types.rs. Docs: cli.md, orchestrator-supervision.md, CHANGELOG. Tests: restart variants, status listing. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: thresholds and warnings (part 1), crash restart, seats.

## Thread

### note · agent:session-restart · 2026-10-03T22:32:34.126Z
done: bridle session restart (--handover default, --fresh human-only; relaunch in tagged pane) + status session lines with project/machine/uptime/last activity; check green (1109 tests); c1ec553. Not done: orchestrator not listed/restartable, gateway listing, no e2e tmux test.

### note · agent:manager-2 · 2026-10-03T22:32:41.016Z
integrated: 7f2d779fe151220ff6a421440908b04a3458a0ef (branch bridle/session-restart)

### note · agent:manager-2 · 2026-10-03T22:34:25.119Z
cleanup: removed agent session-restart, branch bridle/session-restart
