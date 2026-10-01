+++
id = "br-6b8a"
title = "Orchestrator relaunch liveness: one clock, never a second orchestrator (jf9u)"
kind = "bug"
state = "planned"
created_at = "2026-10-01T02:16:26.855Z"
updated_at = "2026-10-01T02:31:45.930Z"
size = "M"
priority = "high"
summary = "Orchestrator pid file now records `@<start epoch>`, read by one helper (containment::start_epoch, LC_ALL=C TZ=UTC) in both the launcher and the daemon and compared with 1 s slack, so TZ/LC_TIME no longer make a live session look dead. Old-format pid files (local lstart string) are treated as alive when the pid is a `bridle session`. In the supervisor, a pid that fails the match but is a live `bridle session` files one incident and does not relaunch. Agents' own start-string identity (DB, containment tracker) is untouched to avoid breaking stored strings on upgrade. Docs: orchestrator-supervision.md, CHANGELOG. Not landed, per brief."
+++

Ticket: docs/tickets/open/context-wakes-never-reach-the-orchestrator-on-a-client-machi-jf9u.md (read all of it, especially "Likely cause: the pid file start time never matches" and "The human's diagnosis agrees"). Code: write_pid_file in crates/bridle/src/session.rs; RealProcs::is_alive in crates/bridle-daemon/src/orchestrator.rs; containment::start_time.

Goal: (1) process identity and age use ONE clock, never a local-time string: stop comparing `ps lstart` strings that move with TZ/LC_TIME and drift by a second on Linux. Compare start times as instants: read them in a fixed locale and zone (LC_ALL=C TZ=UTC) on both sides, or on Linux use /proc/<pid>/stat starttime, with a second of slack, or share one helper so both sides produce the same value. Keep reading old-format pid files (written by an already-running session) without treating them as dead: if the stored string cannot be compared, fall back to the safe answer (alive when the pid is a bridle session / its claude child). (2) The daemon never starts a second orchestrator while the first is alive: before relaunching after "found dead", double-check; if the recorded pid is alive and is a `bridle session` (or its child claude) but its start string does not match, file an incident and do NOT relaunch. (3) With this, a client-machine orchestrator gets context wakes.
Tests: a live session looks alive under a different TZ and LC_TIME for the daemon vs the session (set TZ in the test process env for one side); one-second drift is tolerated; a dead pid is still found dead; reused pid (different instant) is still rejected; relaunch is refused with an incident when the pid is alive but the string mismatches; the old pid-file format still works. Docs: orchestrator-supervision.md, CHANGELOG. Acceptance: just check passes.
Model: Sonnet.

OVERNIGHT, BRANCH ONLY (human approval 2026-10-01): this is the relaunch/start-up path. Build it on its branch, pass just check, and hand off. Do NOT land: the manager parks the checked branch until Sat 2026-10-03; it merges after the human's review.
Out of scope: per-project pid files (7d62), the other causes in the ticket (enabled flag, BRIDLE_HOME), self-upgrade.

## Thread

### note · agent:pm-1 · 2026-10-01T02:16:26.856Z
priority: normal -> high

### note · agent:relaunch-clock · 2026-10-01T02:31:45.930Z
done (branch only, not landed): orchestrator identity by UTC start epoch with 1s slack, old pid files still alive, no relaunch beside a live bridle session (incident instead); just check green (950 tests); d2658d3
