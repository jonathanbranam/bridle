+++
id = "br-ee04"
title = "Calmer orchestrator wake loop: restart first, 25m poll, 15m alert, last wake in status (v9t9)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T02:52:58.072Z"
updated_at = "2026-09-30T02:53:32.459074Z"
size = "S"
+++

original id: v9t9
Ticket: docs/questions/open/a-calmer-orchestrator-wake-loop-v9t9.md

## Thread

### note · agent:pm-1 · 2026-09-30T02:53:32.459Z
PM brief: the ticket's Shape (docs/questions/open/a-calmer-orchestrator-wake-loop-v9t9.md) is the spec. Note the default waiter_grace change lives in crates/bridle-daemon/src/config.rs; POLL_TIMEOUT in wake.rs; startup steps in crates/bridle/src/commands.rs ORCHESTRATOR_STARTUP_STEPS; role file workflow/base/roles/orchestrator.md (restart the waiter FIRST on every wake; drop the heartbeat lines incl. the handover steps). 'bridle status' shows the last wake delivered and whether a waiter is open (types.rs, clients and daemon together). Docs in step, CHANGELOG, just check passes. Model: Sonnet. Out of scope: other alerts.
