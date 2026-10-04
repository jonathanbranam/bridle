---
id: e35h
title: Interactive sessions survive a daemon restart (the registry is lost on every self-upgrade)
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [gtzx, jttf]
tasks: []
---

## The ask

A daemon restart forgets every interactive session. The session registry
(`crates/bridle-daemon/src/sessions.rs`, `POST /v1/sessions`) is in memory only, and a session
registers only when its launcher starts. After tonight's two self-upgrade restarts (2026-10-04
01:51Z and 02:02Z) `bridle status` showed no sessions while six advisor launchers and the aide
ran. Effects until each launcher restarts:

- mail to `external:advisor/<name>` falls back to the shared advisor inbox (the orchestrator's
  m-4237 and m-4240 for doc-review reached the main advisor);
- `external:aide` is refused ("no such recipient");
- context warnings (jttf) stop for every session.

Self-upgrade now restarts the daemon routinely, so this recurs.

**Stopgap, until gtzx's seats table replaces the registry:** the daemon keeps the registry
across its own restarts. Smallest form: write it to a file under `$BRIDLE_HOME` (or the
database) on every change, reload it at start, and let the existing 10 s tick drop entries whose
pid is gone (check the pid's start time too, so a reused pid isn't adopted). No new API. Update
`orchestrator-supervision.md` ("The registry is in memory only") and `api.md`.

Reported by the main advisor (m-4249, 2026-10-04 02:28Z); gtzx P2/P4 fix it for good.
