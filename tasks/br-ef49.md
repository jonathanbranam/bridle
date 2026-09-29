+++
id = "br-ef49"
title = "stop-daemon reports progress and completion (q23k)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T20:34:11.054Z"
updated_at = "2026-09-29T20:34:15.497063Z"
size = "S"
+++

Ticket: docs/questions/open/stop-daemon-reports-progress-q23k.md (read it: the human's ask, what changed, and the KISS Shape). Goal: bridle stop-daemon prints as it goes, not only at the end: 'requested shutdown', then on acknowledgement 'acknowledged; the daemon is stopping N agents, up to Ns' (N from Health agent_count, the limit from the daemon's stop_grace plus 5 s, exposed by the shutdown response or Health, not hard-coded), then 'shutdown complete (Ns)'. Optional and only if cheap: print when the agent count drops. On the 60 s timeout say so and point at bridle daemons and <workspace>/.bridle/daemon.log. Files: crates/bridle/src/commands.rs (stop-daemon, around line 460), crates/bridle-api/src/types.rs if the shutdown response needs the limit, the daemon shutdown handler in crates/bridle-daemon (lib.rs ~455-490 for the sequence), docs/design/cli.md, daemon.md, CHANGELOG. Wire changes go to types.rs, the client and the daemon together. Acceptance: just check passes; a test for the printed sequence against a fake or test daemon, and for the timeout message. Model: Sonnet. Out of scope: changing how long shutdown takes.
