+++
id = "br-e35h"
title = "Interactive sessions survive a daemon restart (the registry is lost on every self-upgrade)"
kind = "bug"
state = "planned"
created_at = "2026-10-04T02:29:11.334Z"
updated_at = "2026-10-04T02:29:24.783823Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: e35h
Bug. Full description: docs/tickets/open/interactive-sessions-survive-a-daemon-restart-the-registry-i-e35h.md (read it). The interactive-session registry (crates/bridle-daemon/src/sessions.rs, POST /v1/sessions) is in memory only, so every self-upgrade restart forgets all sessions: named-advisor mail falls back to the shared inbox, external:aide is refused, jttf context warnings stop.
Build (stopgap, keep it small; gtzx's seats table replaces it later): persist the registry to a file under $BRIDLE_HOME (or the database, whichever is smaller in this code) on every change; reload at daemon start; the existing 10 s tick drops entries whose pid is gone, checking the pid's start time too so a reused pid isn't adopted. No new API. Files: sessions.rs and its startup wiring in the daemon. Docs: orchestrator-supervision.md ('The registry is in memory only'), api.md, CHANGELOG.
Tests: register, 'restart' (reload), entry survives; dead pid dropped; reused pid (start time differs) dropped. Acceptance: just check passes. Model: Sonnet. Migration: none (a missing file means an empty registry). Out of scope: seats table (gtzx), re-registering sessions from launchers.
