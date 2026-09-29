+++
id = "br-ef49"
title = "stop-daemon reports progress and completion (q23k)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T20:34:11.054Z"
updated_at = "2026-09-29T20:44:45.621482Z"
size = "S"
branch = "bridle/stop-progress"
commit = "1059ed6"
summary = 'bridle stop-daemon now prints as it goes: "requested shutdown", "acknowledged; the daemon is stopping N agents, up to Ns", "N agents still running" as the count drops, then "shutdown complete (Ns)". POST /v1/shutdown replies with a new ShutdownResponse {stop_limit_secs} (stop_grace + 5 s, from AppState.stop_grace) instead of 204. The 60 s timeout error points at `bridle daemons` and <workspace>/.bridle/daemon.log (generic path, not resolved). Logic is stop_daemon_with in commands.rs, unit-tested against a hand-rolled TCP fake for the sequence and the timeout. Docs: api.md, daemon.md, cli.md, CHANGELOG. Ticket q23k left open for the human to close.'
+++

Ticket: docs/questions/open/stop-daemon-reports-progress-q23k.md (read it: the human's ask, what changed, and the KISS Shape). Goal: bridle stop-daemon prints as it goes, not only at the end: 'requested shutdown', then on acknowledgement 'acknowledged; the daemon is stopping N agents, up to Ns' (N from Health agent_count, the limit from the daemon's stop_grace plus 5 s, exposed by the shutdown response or Health, not hard-coded), then 'shutdown complete (Ns)'. Optional and only if cheap: print when the agent count drops. On the 60 s timeout say so and point at bridle daemons and <workspace>/.bridle/daemon.log. Files: crates/bridle/src/commands.rs (stop-daemon, around line 460), crates/bridle-api/src/types.rs if the shutdown response needs the limit, the daemon shutdown handler in crates/bridle-daemon (lib.rs ~455-490 for the sequence), docs/design/cli.md, daemon.md, CHANGELOG. Wire changes go to types.rs, the client and the daemon together. Acceptance: just check passes; a test for the printed sequence against a fake or test daemon, and for the timeout message. Model: Sonnet. Out of scope: changing how long shutdown takes.

## Thread

### note · agent:stop-progress · 2026-09-29T20:42:26.432Z
done: stop-daemon prints requested/acknowledged (N agents, up to Ns from the daemon)/count drops/shutdown complete (Ns); /v1/shutdown now returns {stop_limit_secs}; timeout points at bridle daemons + daemon.log; unit tests for sequence and timeout; just check passes with main merged; e4875e4; docs: api.md, daemon.md, cli.md, CHANGELOG updated; ticket q23k left open for the human to close

### note · agent:manager-2 · 2026-09-29T20:42:31.516Z
Two things. (1) main moved: merge it, rerun just check. (2) Compatibility: the new client will be run against the OLD daemon (which replies 204 with no body) the first time the human stops it after installing. Make the client tolerate an empty/204 shutdown response (no stop limit: print 'acknowledged; the daemon is stopping N agents' without the 'up to' part) and add a test for that. Message me the sha and end. Rule: never pkill -f/killall.

### note · agent:stop-progress · 2026-09-29T20:44:21.352Z
done: client tolerates a 204/empty shutdown reply (prints 'acknowledged; the daemon is stopping N agents' without 'up to'), with a test; main merged, just check passes; cc761d2

### note · agent:manager-2 · 2026-09-29T20:44:25.879Z
integrated: 1059ed6 (branch bridle/stop-progress)

### note · agent:manager-2 · 2026-09-29T20:44:45.621Z
cleanup: removed agent stop-progress, branch bridle/stop-progress
