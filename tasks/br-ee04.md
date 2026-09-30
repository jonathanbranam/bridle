+++
id = "br-ee04"
title = "Calmer orchestrator wake loop: restart first, 25m poll, 15m alert, last wake in status (v9t9)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T02:52:58.072Z"
updated_at = "2026-09-30T03:23:41.908447Z"
size = "S"
summary = """wait-for-wake poll is 25m (wake.rs POLL_TIMEOUT); waiter_grace code default is now 15m (main's 11ea64c only changed .bridle/config.toml; the orchestrator incident test's ticks moved to match). Role file: restart the waiter first on every wake; heartbeat lines dropped from the handover steps and ORCHESTRATOR_STARTUP_STEPS. Status gains waiter_open and last_wake_at (in memory in Waiters, set when a poll answers with wakes; a daemon restart clears it), shown by `bridle status` as a "wake" line. Docs and CHANGELOG updated. just check passes (788 tests)."""
+++

original id: v9t9
Ticket: docs/questions/open/a-calmer-orchestrator-wake-loop-v9t9.md

## Thread

### note · agent:pm-1 · 2026-09-30T02:53:32.459Z
PM brief: the ticket's Shape (docs/questions/open/a-calmer-orchestrator-wake-loop-v9t9.md) is the spec. Note the default waiter_grace change lives in crates/bridle-daemon/src/config.rs; POLL_TIMEOUT in wake.rs; startup steps in crates/bridle/src/commands.rs ORCHESTRATOR_STARTUP_STEPS; role file workflow/base/roles/orchestrator.md (restart the waiter FIRST on every wake; drop the heartbeat lines incl. the handover steps). 'bridle status' shows the last wake delivered and whether a waiter is open (types.rs, clients and daemon together). Docs in step, CHANGELOG, just check passes. Model: Sonnet. Out of scope: other alerts.

### note · agent:calm-wake · 2026-09-30T03:23:29.906Z
done: 25m poll, 15m waiter_grace code default (main only had the config.toml change), restart-first in role file, heartbeat dropped, status shows waiter_open/last_wake_at; just check green (788), main merged; 26ddb3c

### note · agent:manager-2 · 2026-09-30T03:23:41.908Z
main moved. Merge main, confirm ancestor (just check if the merge touched code), message me the sha.
