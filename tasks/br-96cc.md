+++
id = "br-96cc"
title = "Wake the manager when a task's settle period ends (ny9u follow-up)"
kind = "bug"
state = "planned"
created_at = "2026-10-02T23:44:04.679Z"
updated_at = "2026-10-02T23:44:06.854894Z"
size = "S"
+++

Ticket: docs/tickets/open/a-settle-period-an-approved-task-waits-a-few-minutes-before-ny9u.md, section 'Follow-up: nothing wakes the manager when a settle period ends'. Seen twice (br-5924, br-1e88): an idle manager isn't woken when a task finishes settling, so a startable task sits until nudged. Read how the daemon wakes/messages agents (docs/design/agent-host/messages.md, the supervisor's existing periodic ticks, the settle computation from br-3c71 in crates/bridle-daemon). Goal: when a queued task's settle period ends (it becomes startable through settling alone), the daemon sends the manager role's agent one short note ('br-xxxx is now startable'), once per task. KISS: a supervisor tick that compares 'settled now' vs 'settled at last tick' in memory is enough; no schema change, no stored state (after a daemon restart a missed note is acceptable). Never block or affect daemon start-up: the tick must not run work before start-up completes and must log, not fail, on error. Tests: a task crossing its settle time produces exactly one note; already-settled tasks produce none; a task blocked by deps produces none. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: other wake causes. HOLD: do not start before the gateway's first wave (br-1e88, br-c601, br-9418, br-8545) has landed AND the human is back (human away until late 10-02 night).
