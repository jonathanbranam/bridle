+++
id = "br-ds96"
title = "Before a planned restart or shutdown, the daemon ends every waiter with the reason (shutting down, upgrading, restarting by request) and records it in events"
kind = "feature"
state = "planned"
created_at = "2026-10-05T02:11:30.511Z"
updated_at = "2026-10-05T02:27:56.638848Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: ds96
Build docs/tickets/open/before-a-planned-restart-or-shutdown-the-daemon-ends-every-w-ds96.md (read it and 75h2 for the wake code). Before a planned restart (self-upgrade, restart by request) or a shutdown, the daemon first ends every open waiter (bridle agent wake, orchestrator wait-for-wake; principal_wake in crates/bridle-daemon/src/server.rs, Waiters in wake.rs) with a reason: shutting down; restarting for an internal upgrade (to which build); restarting by request (by whom). The waiter prints the reason on stderr and exits with its own code, not the timeout's 4: use 6 ('daemon restarting or shutting down; re-arm once it is back'), since 75h2 part 2 (br-dahs) takes 5 for 'superseded'. --json output stays clean. Post one event for the restart/shutdown with its reason and the number of waiters ended. Find the planned-restart and shutdown paths (upgrade, restart handler, graceful stop) and hook there, before the listener closes; an unplanned crash can't do this, and that's fine. Update docs/design/agent-host/ (wake, exit codes, events) and cli.md. Acceptance: just check passes; tests: a waiter open during a planned restart gets exit 6 with the reason, a real timeout still exits 4, the event is recorded. br-dahs touches the same wake code and runs after this (dependency edge). Model: Sonnet. Out of scope: replacement/--stop (br-dahs), unplanned crashes.

## Thread

### note · external:orchestrator · 2026-10-05T02:27:09.827Z
Readied by orchestrator: the human, via aide (m-5240, 2026-10-04 ~10:15 PM ET): "Schedule it."
