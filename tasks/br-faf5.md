+++
id = "br-faf5"
title = "bridle serve warns loudly at start-up when claude isn't logged in (nrbf part 2, serve half)"
kind = "feature"
state = "planned"
created_at = "2026-10-03T01:48:00.343Z"
updated_at = "2026-10-03T01:48:01.626453Z"
size = "S"
+++

Ticket: docs/tickets/open/a-daemon-whose-claude-isn-t-logged-in-runs-agents-that-silen-nrbf.md, part 2. Builds on br-5b39, which adds the 'claude auth status' check to 'bridle doctor': REUSE its helper, don't duplicate. Goal: at 'bridle serve' start-up, run that check in the daemon's own environment and log a loud warning (and record an incident if the incident mechanism fits) when claude isn't logged in, with the fix (macOS: start the daemon from a local terminal or tmux, not over SSH). It must never block or fail start-up: run it off the critical path (spawned task or short timeout, e.g. 5 s), treat a missing 'claude', a timeout or any error as 'unknown', not as not-logged-in. Tests: not-logged-in -> warning and start-up completes; check errors or times out -> start-up completes, no false warning; logged in -> silent (injected command, no real claude). Docs + CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: re-login automation. This is on the daemon start-up path (the human is back and approved it): keep it small and safe.
