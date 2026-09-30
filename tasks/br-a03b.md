+++
id = "br-a03b"
title = "Mark a message unread: API, bridle inbox unread, TUI key (n94h follow-up)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T02:12:21.646Z"
updated_at = "2026-09-30T02:26:22.060011Z"
+++

GOAL (human, via advisor m-2192): a message can be marked unread again. Add POST /v1/messages/{id}/unread (crates/bridle-daemon server + store; mirror the existing mark-read path, and change crates/bridle-api/src/types.rs, client, daemon together), 'bridle inbox unread <id>' (crates/bridle/src/cli.rs, commands.rs), and a key in the TUI inbox (crates/bridle-tui/src/app.rs; show it in the title bar like the done key from br-6441). Docs in step: docs/design/agent-host/api.md, docs/design/cli.md, CHANGELOG. Acceptance: just check passes; tests: mark read then unread returns the message to the unread list, unknown id is not_found. Model: Sonnet. Out of scope: bulk operations, other TUI redesign.

## Thread

### note · agent:manager-2 · 2026-09-30T02:26:22.060Z
You were interrupted by a daemon restart. Check git status in your worktree (uncommitted work), finish br-a03b per 'bridle task show br-a03b', merge main, run the check per the worker skill (full just check, output to /tmp/br-a03b-check.log), write the summary, commit, message me.
