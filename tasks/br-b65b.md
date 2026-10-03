+++
id = "br-b65b"
title = "bridle serve --detach gives up at 15 s while the daemon is still starting"
kind = "bug"
state = "planned"
created_at = "2026-10-03T01:27:14.533Z"
updated_at = "2026-10-03T01:27:53.529846Z"
+++

original id: cy5v
Ticket: docs/tickets/open/bridle-serve-detach-gives-up-at-15-s-while-the-daemon-is-sti-cy5v.md (read it all). Code: crates/bridle/src/serve.rs, the wait loop of 'bridle serve --detach' ONLY. Not the daemon's own start-up, restart or upgrade code.
Goal: (1) wait about 60 s (not 15) for the detached daemon to come up; (2) keep the existing config warnings it prints; (3) on timeout, say clearly that the daemon is still starting and was LEFT RUNNING, with its pid, its log path, and a pointer to 'bridle daemons', and exit non-fatally as the ticket says (follow what it asks for the exit code). Make the wait/timeout injectable so a test doesn't sleep 60 s. Tests: comes up inside the wait: success; never comes up: the timeout message names pid, log path and 'bridle daemons'; config warnings still shown. Docs: cli.md if it states 15 s, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: daemon start-up speed.
