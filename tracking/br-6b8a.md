+++
id = "br-6b8a"
title = "Orchestrator relaunch liveness: one clock, never a second orchestrator (jf9u)"
kind = "bug"
state = "integrated"
created_at = "2026-10-01T02:16:26.855Z"
updated_at = "2026-10-12T04:43:23.840373Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
    "agent:w6b8a",
]
size = "M"
priority = "high"
branch = "bridle/w6b8a"
commit = "3e6f9c68e8788294fbf2beda201baec0857c2660"
summary = "Orchestrator pid file records '@<start epoch>', read by one helper (containment::start_epoch, LC_ALL=C TZ=UTC) in launcher and daemon, compared with 1 s slack, so TZ/LC_TIME no longer make a live session look dead. Old-format pid files count as alive when the pid is a bridle session. A failed match on a live bridle session files one incident and does not relaunch. Docs: orchestrator-supervision.md, CHANGELOG. Cherry-picked from bridle/relaunch-clock d2658d3 (prior worker's work), main merged. Branch only, not landed, per brief."
ticket = "6b8a"
+++

docs/tickets/open/orchestrator-relaunch-liveness-one-clock-never-a-second-orch-6b8a.md

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

### note · agent:orch-liveness · 2026-10-02T12:54:52.237Z
br-6b8a is already done: agent relaunch-clock finished it on branch bridle/relaunch-clock (d2658d3, just check green, 950 tests, summary already written). My branch bridle/orch-liveness has no changes. I won't redo it. Should I stop, or reimplement it from scratch on my branch?

### note · external:advisor/product-manager · 2026-10-09T11:04:36.228Z
watching the task

### note · agent:w6b8a · 2026-10-12T01:21:21.459Z
done: jf9u relaunch liveness (cherry-picked prior d2658d3 from bridle/relaunch-clock, main merged); just check exit 0, 1553 tests passed, check ran on 0ce7df8c, tip ad9fddc2bd888c50cdb42a2ca8801a7c47c95a6a differs by docs-only main merge; branch only, not landed per brief

### note · agent:w6b8a · 2026-10-12T01:56:57.323Z
update: merged main (c9a308a5) again; just check exit 0, 1555 tests passed, checked tip b9731cba48805899e65634c3e958485ee460ff10

### note · external:advisor/product-manager · 2026-10-12T03:58:31.577Z
From the human, via advisor (product-manager), 2026-10-11 ~11:58 PM ET: "land 6b8a". Branch bridle/w6b8a (tip b9731cba, just check green 1555). After it lands, br-10c0 (7d62) is unheld and goes to pm-1 to plan against what landed.

### note · agent:manager-2 · 2026-10-12T04:43:23.840Z
integrated: 3e6f9c68e8788294fbf2beda201baec0857c2660 (branch bridle/w6b8a)
