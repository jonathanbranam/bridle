+++
id = "br-faf5"
title = "bridle serve warns loudly at start-up when claude isn't logged in (nrbf part 2, serve half)"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T01:48:00.343Z"
updated_at = "2026-10-03T02:20:06.142400Z"
size = "S"
branch = "bridle/serve-login-warn"
commit = "5e0b2c672b238b58501bfd50aad0229f93dde6cd"
summary = "bridle serve (foreground; detached child runs it too) spawns a task that runs doctor's now-shared claude_logged_in (Option<bool>) with a 5 s timeout and logs a warning with the fix only on a known logged-out answer; missing claude/timeout/error are silent. Never blocks start-up. Tests with injected fake claude; docs daemon.md + CHANGELOG."
+++

Ticket: docs/tickets/open/a-daemon-whose-claude-isn-t-logged-in-runs-agents-that-silen-nrbf.md, part 2. Builds on br-5b39, which adds the 'claude auth status' check to 'bridle doctor': REUSE its helper, don't duplicate. Goal: at 'bridle serve' start-up, run that check in the daemon's own environment and log a loud warning (and record an incident if the incident mechanism fits) when claude isn't logged in, with the fix (macOS: start the daemon from a local terminal or tmux, not over SSH). It must never block or fail start-up: run it off the critical path (spawned task or short timeout, e.g. 5 s), treat a missing 'claude', a timeout or any error as 'unknown', not as not-logged-in. Tests: not-logged-in -> warning and start-up completes; check errors or times out -> start-up completes, no false warning; logged in -> silent (injected command, no real claude). Docs + CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: re-login automation. This is on the daemon start-up path (the human is back and approved it): keep it small and safe.

## Thread

### note · agent:serve-login-warn · 2026-10-03T02:18:47.455Z
done: serve warns at start-up when claude is logged out (spawned, 5 s limit, unknown=silent, reuses doctor helper); just check passed (1055 tests); 203a1b3

### note · agent:manager-2 · 2026-10-03T02:18:50.959Z
Diff looks good. main moved; merge it, run just check, message me sha and result.

### note · agent:serve-login-warn · 2026-10-03T02:19:52.913Z
merged main; just check passed (1055 tests); c46a387

### note · agent:manager-2 · 2026-10-03T02:19:54.954Z
integrated: 5e0b2c672b238b58501bfd50aad0229f93dde6cd (branch bridle/serve-login-warn)

### note · agent:manager-2 · 2026-10-03T02:20:06.142Z
cleanup: removed agent serve-login-warn, branch bridle/serve-login-warn
