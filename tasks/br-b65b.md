+++
id = "br-b65b"
title = "bridle serve --detach gives up at 15 s while the daemon is still starting"
kind = "bug"
state = "integrated"
created_at = "2026-10-03T01:27:14.533Z"
updated_at = "2026-10-03T01:43:41.319482Z"
branch = "bridle/detach-wait"
commit = "b583695ef010509848f95ed5146abed207860543"
summary = "serve --detach now waits 60 s (DETACH_WAIT) via an extracted, timeout-injectable wait_for_daemon in crates/bridle/src/serve.rs. On timeout it prints a warning (pid, log path, 'bridle daemons', left running) to stderr and exits 0, since the ticket gave no exit code and asked for non-fatal; early child exit is still an error. Config warnings come from the daemon's own log, untouched. Tests: success, timeout leaves child running, message content. Docs: daemon.md, CHANGELOG."
+++

original id: cy5v
Ticket: docs/tickets/open/bridle-serve-detach-gives-up-at-15-s-while-the-daemon-is-sti-cy5v.md (read it all). Code: crates/bridle/src/serve.rs, the wait loop of 'bridle serve --detach' ONLY. Not the daemon's own start-up, restart or upgrade code.
Goal: (1) wait about 60 s (not 15) for the detached daemon to come up; (2) keep the existing config warnings it prints; (3) on timeout, say clearly that the daemon is still starting and was LEFT RUNNING, with its pid, its log path, and a pointer to 'bridle daemons', and exit non-fatally as the ticket says (follow what it asks for the exit code). Make the wait/timeout injectable so a test doesn't sleep 60 s. Tests: comes up inside the wait: success; never comes up: the timeout message names pid, log path and 'bridle daemons'; config warnings still shown. Docs: cli.md if it states 15 s, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: daemon start-up speed.

## Thread

### note · agent:detach-wait · 2026-10-03T01:42:21.723Z
done: serve --detach waits 60 s, timeout leaves daemon running (exit 0, message with pid/log/bridle daemons); just check green (1048 passed); 2d56fa3

### note · agent:manager-2 · 2026-10-03T01:42:26.214Z
br-6c6a landed (56e92ea, touches wake.rs/docs, not serve.rs). Merge main, run just check, message me sha and result.

### note · agent:detach-wait · 2026-10-03T01:43:26.623Z
main (56e92ea, br-6c6a) was already merged into my branch; just check green (1048 passed); tip 2d56fa3

### note · agent:manager-2 · 2026-10-03T01:43:28.627Z
integrated: b583695ef010509848f95ed5146abed207860543 (branch bridle/detach-wait)

### note · agent:manager-2 · 2026-10-03T01:43:41.319Z
cleanup: removed agent detach-wait, branch bridle/detach-wait
