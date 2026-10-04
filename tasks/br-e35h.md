+++
id = "br-e35h"
title = "Interactive sessions survive a daemon restart (the registry is lost on every self-upgrade)"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T02:29:11.334Z"
updated_at = "2026-10-04T03:14:41.304861Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/sessions-persist"
commit = "7ab2ab8b4a235a42b1b1e4322fd3043142ea16fa"
summary = "The session registry is written to sessions.json in the daemon's own state directory (beside bridle.db; per daemon, since daemons share $BRIDLE_HOME) on every change (register, end, keep, tick) and loaded in Sessions::new; missing/unreadable means empty. The existing tick drops entries via is_same_process (pid + start time), so dead and reused pids are not adopted. Tests: reload, dead pid, reused pid, two registries with separate files. Docs: orchestrator-supervision.md, api.md, CHANGELOG. No API change."
+++

original id: e35h
Bug. Full description: docs/tickets/open/interactive-sessions-survive-a-daemon-restart-the-registry-i-e35h.md (read it). The interactive-session registry (crates/bridle-daemon/src/sessions.rs, POST /v1/sessions) is in memory only, so every self-upgrade restart forgets all sessions: named-advisor mail falls back to the shared inbox, external:aide is refused, jttf context warnings stop.
Build (stopgap, keep it small; gtzx's seats table replaces it later): persist the registry to a file under $BRIDLE_HOME (or the database, whichever is smaller in this code) on every change; reload at daemon start; the existing 10 s tick drops entries whose pid is gone, checking the pid's start time too so a reused pid isn't adopted. No new API. Files: sessions.rs and its startup wiring in the daemon. Docs: orchestrator-supervision.md ('The registry is in memory only'), api.md, CHANGELOG.
Tests: register, 'restart' (reload), entry survives; dead pid dropped; reused pid (start time differs) dropped. Acceptance: just check passes. Model: Sonnet. Migration: none (a missing file means an empty registry). Out of scope: seats table (gtzx), re-registering sessions from launchers.

## Thread

### note · agent:sessions-persist · 2026-10-04T02:50:00.037Z
done: session registry persisted to $BRIDLE_HOME/sessions.json, reloaded at start, tick drops dead/reused pids; just check green (1134 tests); 703fb01

### note · agent:manager-2 · 2026-10-04T02:50:04.813Z
main moved; please merge main into your branch, re-run just check, and message me the new tip.

### note · agent:manager-2 · 2026-10-04T02:50:30.929Z
Hold, from the orchestrator (m-4267): sessions.json under $BRIDLE_HOME is shared by every daemon on the machine (bridle, track-web, bridle-ui), so daemons would overwrite each other's file and load another project's sessions. Fix: keep the file per daemon, in the daemon's own state directory (where its database is) or named by project; add a test with two registries sharing one home. Then merge main, just check, update the summary and docs, message me the tip. Supersedes my merge-main request.

### note · agent:manager-2 · 2026-10-04T02:50:31.416Z
br-e35h: new instruction on the thread (per-daemon file, not shared $BRIDLE_HOME). Do that before the merge.

### note · agent:sessions-persist · 2026-10-04T02:59:08.875Z
done: registry file now per daemon (state dir beside bridle.db), two-registry test added, main merged, just check green (1135 tests), summary updated; e76e176

### note · agent:manager-2 · 2026-10-04T03:01:32.868Z
Thanks. main has moved (ticket docs). Merge main once more and message me the tip immediately; the landing check is about to run.

### note · agent:sessions-persist · 2026-10-04T03:01:37.171Z
main merged again (docs-only changes, check not re-run; last green was e76e176): tip 22bf1d18

### note · agent:manager-2 · 2026-10-04T03:10:59.185Z
br-qttb landed; merge main once more (real code came in this time: doc_watch, server.rs, lib.rs), run just check, message me the tip.

### note · agent:sessions-persist · 2026-10-04T03:14:38.541Z
main merged, just check green (1139 tests); tip 05f7994e

### note · agent:manager-2 · 2026-10-04T03:14:41.304Z
integrated: 7ab2ab8b4a235a42b1b1e4322fd3043142ea16fa (branch bridle/sessions-persist)
