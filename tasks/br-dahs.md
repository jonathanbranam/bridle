+++
id = "br-dahs"
title = "Waiters: a new wait replaces the old (by session), bridle agent wake --stop, pid on stderr, prompts (75h2 part 2)"
kind = "feature"
state = "planned"
created_at = "2026-10-04T23:54:00.140Z"
updated_at = "2026-10-05T00:28:49.444152Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Part 2 of docs/tickets/open/a-waiter-is-stopped-through-bridle-never-by-a-kill-a-new-wai-75h2.md (read it: P1, P2, P5, P6 and 'Decided'). Part 1 (br-75h2, kill deny and rule text) lands first. Build: P1 the daemon tracks open waits by SESSION (Waiters in crates/bridle-daemon/src/wake.rs today only counts; principal_wake in server.rs); a new wait from the same session ends the old one, which exits with code 5 'superseded by a newer wait' and marks nothing read. The session comes from the launcher (SessionRegister, exported to the session environment; bridle agent wake sends it); a wait with no session replaces nothing. Replacement is per session, not identity, because identities are shared. P2 bridle agent wake --stop ends this session's open wait through the daemon (no signals), or with <identity> that identity's waits; you may stop only your own, the human may stop any; prints what it stopped or 'no wait open'; exit code 5 for the stopped waiter. P5 the waiter prints 'waiting as <identity> (pid N, timeout S s)' on stderr so --json stays clean. P6 update the waiting sections of the orchestrator, aide and advisor role prompts (start a waiter only with Claude Code's background command, never & or discarded output; replace by starting a new one; stop with --stop), and docs/design/cli.md and docs/design/agent-host/ (wake, exit codes). Overlaps wake code with br-2672 (wakes move into bridle agent wake): run after it if both are live; check its state first. Acceptance: just check passes; tests for replacement within a session, no replacement across sessions with a shared identity, --stop own/other/none, exit code 5. Model: Sonnet. Out of scope: denying plain kill.
