+++
id = "br-96cc"
title = "Wake the manager when a task's settle period ends (ny9u follow-up)"
kind = "bug"
state = "integrated"
created_at = "2026-10-02T23:44:04.679Z"
updated_at = "2026-10-03T01:59:07.277524Z"
size = "S"
branch = "bridle/settle-wake"
commit = "36053bc6e28ed4cef4ef909562a7eaa6527eb89d"
summary = "Added queue_nudge::SettleWake: a 30 s daemon tick (spawn_loop, so it never runs before start-up; no error path) keeps the in-memory set of planned, otherwise-ready tasks still settling and sends the running manager (else the orchestrator) '<id> is now startable' once when one leaves it. Dependency-blocked/question-blocked and already-settled tasks make no note. New Overrides.settle_wake_interval; no schema change. Docs: coordination.md; tests: settle_wake_test.rs."
+++

Ticket: docs/tickets/open/a-settle-period-an-approved-task-waits-a-few-minutes-before-ny9u.md, section 'Follow-up: nothing wakes the manager when a settle period ends'. Seen twice (br-5924, br-1e88): an idle manager isn't woken when a task finishes settling, so a startable task sits until nudged. Read how the daemon wakes/messages agents (docs/design/agent-host/messages.md, the supervisor's existing periodic ticks, the settle computation from br-3c71 in crates/bridle-daemon). Goal: when a queued task's settle period ends (it becomes startable through settling alone), the daemon sends the manager role's agent one short note ('br-xxxx is now startable'), once per task. KISS: a supervisor tick that compares 'settled now' vs 'settled at last tick' in memory is enough; no schema change, no stored state (after a daemon restart a missed note is acceptable). Never block or affect daemon start-up: the tick must not run work before start-up completes and must log, not fail, on error. Tests: a task crossing its settle time produces exactly one note; already-settled tasks produce none; a task blocked by deps produces none. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: other wake causes. HOLD: do not start before the gateway's first wave (br-1e88, br-c601, br-9418, br-8545) has landed AND the human is back (human away until late 10-02 night).

## Thread

### note · agent:settle-wake · 2026-10-03T01:58:53.451Z
done: settle-wake tick notes the manager once when a task finishes settling; just check passed (1051 tests); 2f382dc

### note · agent:manager-2 · 2026-10-03T01:58:57.649Z
integrated: 36053bc6e28ed4cef4ef909562a7eaa6527eb89d (branch bridle/settle-wake)

### note · agent:manager-2 · 2026-10-03T01:59:07.277Z
cleanup: removed agent settle-wake, branch bridle/settle-wake
