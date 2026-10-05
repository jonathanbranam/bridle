+++
id = "br-dahs"
title = "Waiters: a new wait replaces the old (by session), bridle agent wake --stop, pid on stderr, prompts (75h2 part 2)"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T23:54:00.140Z"
updated_at = "2026-10-05T19:28:55.204789Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/wait-replace"
commit = "3960f1abbce213f0182d4b0387bd46287e4e2fdc"
summary = """Waits are tracked by session in the daemon (Waiters in wake.rs): a new `bridle agent wake` carrying the same BRIDLE_SESSION_PID ends the open one, which answers reason `superseded` (CLI exit 5, nothing marked read); no session replaces nothing, so sessions sharing an identity don't end each other. New POST /v1/wake/stop and `bridle agent wake --stop [identity]` (own only, human any; prints "stopped N wait(s)" or "no wait open"). The waiter prints `waiting as <identity> (pid N, timeout S s)` on stderr. Prompts (aide, advisor, orchestrator), cli.md, api.md and CHANGELOG updated. Caveats: the orchestrator's wait-for-wake isn't covered (br-2672 moves it into agent wake); a session-only --stop isn't checked against the caller's identity (the session pid is the only key). Tests in principal_wake_test.rs; project_resolution_test needed a GIVEN entry because the identifier is now optional."""
+++

Part 2 of docs/tickets/open/a-waiter-is-stopped-through-bridle-never-by-a-kill-a-new-wai-75h2.md (read it: P1, P2, P5, P6 and 'Decided'). Part 1 (br-75h2, kill deny and rule text) lands first. Build: P1 the daemon tracks open waits by SESSION (Waiters in crates/bridle-daemon/src/wake.rs today only counts; principal_wake in server.rs); a new wait from the same session ends the old one, which exits with code 5 'superseded by a newer wait' and marks nothing read. The session comes from the launcher (SessionRegister, exported to the session environment; bridle agent wake sends it); a wait with no session replaces nothing. Replacement is per session, not identity, because identities are shared. P2 bridle agent wake --stop ends this session's open wait through the daemon (no signals), or with <identity> that identity's waits; you may stop only your own, the human may stop any; prints what it stopped or 'no wait open'; exit code 5 for the stopped waiter. P5 the waiter prints 'waiting as <identity> (pid N, timeout S s)' on stderr so --json stays clean. P6 update the waiting sections of the orchestrator, aide and advisor role prompts (start a waiter only with Claude Code's background command, never & or discarded output; replace by starting a new one; stop with --stop), and docs/design/cli.md and docs/design/agent-host/ (wake, exit codes). Overlaps wake code with br-2672 (wakes move into bridle agent wake): run after it if both are live; check its state first. Acceptance: just check passes; tests for replacement within a session, no replacement across sessions with a shared identity, --stop own/other/none, exit code 5. Model: Sonnet. Out of scope: denying plain kill.

## Thread

### note · agent:wait-replace · 2026-10-05T19:23:29.477Z
done: waits tracked by session, a new wait replaces the old (exit 5), agent wake --stop, pid on stderr, prompts/docs/CHANGELOG; just check exit 0, 1250 tests passed; main already merged; commit 835fdc6d

### note · agent:manager-2 · 2026-10-05T19:23:34.686Z
integrated: 3960f1abbce213f0182d4b0387bd46287e4e2fdc (branch bridle/wait-replace)

### note · agent:manager-2 · 2026-10-05T19:28:55.204Z
cleanup: removed agent wait-replace, branch bridle/wait-replace
