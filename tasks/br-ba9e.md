+++
id = "br-ba9e"
title = "Interactive sessions A: advisors register with the daemon; context tracked and reported per session (jttf 1)"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T22:51:22.633Z"
updated_at = "2026-10-01T23:25:16.329625Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "M"
branch = "bridle/jttf-a"
commit = "10f6e7ab3d4b67239d4791a828d11f14232a1237"
summary = "Advisor sessions register with the daemon: new in-memory registry (bridle-daemon/src/sessions.rs), POST/GET /v1/sessions and POST /v1/sessions/end, a 10 s watcher task that drops sessions whose pid is gone (session.ended) and reads $BRIDLE_HOME/context/<id>, emitting session.context at the orchestrator's three token thresholds (re-armed on a lower reading); 'bridle status' gets a session line per advisor. 'bridle session advisor' registers at launch and ends at exit, 3 s best-effort so a daemon down never fails it; a hidden 'bridle session note' SessionStart hook (advisor settings only) adds the Claude session id via BRIDLE_SESSION_PID. Orchestrator path untouched. Caveat: the CLI-side register/end is covered by the existing session tests running with no daemon, not a dedicated test."
ticket = "jttf"
+++

Ticket: docs/tickets/open/interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf.md (decision 1; 'Today' first bullet; the advisors' inbox 'How bridle knows it'). Design: docs/design/agent-host/orchestrator-supervision.md (how the orchestrator's context file, thresholds and orchestrator.context events work today). Code: bridle session in crates/bridle/src/session.rs (advisor branch, around line 221), bridle statusline (writes $BRIDLE_HOME/context/<session id>), the daemon's orchestrator context watching in crates/bridle-daemon, wire types in crates/bridle-api/src/types.rs.

Goal: the daemon knows each running advisor session and reports its context.
1. 'bridle session advisor [<name>]' registers the session with the daemon at launch (identity advisor or advisor/<name>, pid, Claude session id, tmux pane id if any) via a new API call, and ends it at exit (best effort: a daemon down never blocks or fails the session). The daemon also marks it ended when its pid is gone, as it does for the orchestrator.
2. The daemon reads the session's context file (the same one the orchestrator's thresholds use) and reports tokens per session: a 'session.context' event at the existing threshold levels (reuse the orchestrator's thresholds; no new config) and a line per session in 'bridle status' (or the nearest existing display; keep it small).
3. Docs: orchestrator-supervision.md or a short 'interactive sessions' section, cli.md/api docs, storage.md if a table is added (prefer in-memory plus events; add a table only if restart-survival needs it), CHANGELOG.
SAFETY: advisors only. Do NOT touch 'bridle session orchestrator', the orchestrator relaunch path, or the daemon's start-up sequence beyond spawning one more watcher task that cannot fail startup. Lands normally.
Tests: register/end round trip; context event at a threshold from a fake context file; daemon down doesn't fail the session; pid gone marks ended. Acceptance: just check passes. Model: Sonnet.
Migration plan: none (no project files; any DB table migrates itself).
Out of scope: wakes, advisor addressing (slice B), restart, orchestrator.

## Thread

### note · agent:jttf-a · 2026-10-01T23:21:46.422Z
done: advisors register with the daemon, context per session (session.context events, status line); 2198484

### note · agent:manager-2 · 2026-10-01T23:25:05.713Z
integrated: 10f6e7ab3d4b67239d4791a828d11f14232a1237 (branch bridle/jttf-a)

### note · agent:manager-2 · 2026-10-01T23:25:16.329Z
cleanup: removed agent jttf-a, branch bridle/jttf-a
