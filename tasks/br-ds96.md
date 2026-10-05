+++
id = "br-ds96"
title = "Before a planned restart or shutdown, the daemon ends every waiter with the reason (shutting down, upgrading, restarting by request) and records it in events"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T02:11:30.511Z"
updated_at = "2026-10-05T10:46:38.950562Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/waiter-reason"
commit = "7c1accb724f73fe0c217c99e2ff7d3fcd123efcc"
summary = """Planned restarts and shutdowns now end every open waiter with the reason. Waiters (wake.rs) holds the stop reason (first planned path wins: /v1/shutdown, /v1/restart, upgrade with the build sha) and a watch channel; the shutdown task in lib.rs announces it before agents stop and emits daemon.stopping with reason and waiters_ended (orchestrator and `agent wake` waiters both counted, principal ones via a new guard that stays out of the orchestrator presence check). Both wake routes answer a daemon_stopping reason (api types: DAEMON_STOPPING_WAKE, PrincipalWakeReason.text); the CLI maps it to new CliError::Stopping, exit 6 with the reason on stderr; real timeouts still exit 4. A signal stop uses the default "shutting down". Test: principal_wake_test waiter open during a restart. Docs: cli.md, api.md, CHANGELOG. Caveat: the orchestrator waiter route has no dedicated test; the orchestrator's queued "restart" wake may win the race there and exit 0."""
+++

original id: ds96
Build docs/tickets/open/before-a-planned-restart-or-shutdown-the-daemon-ends-every-w-ds96.md (read it and 75h2 for the wake code). Before a planned restart (self-upgrade, restart by request) or a shutdown, the daemon first ends every open waiter (bridle agent wake, orchestrator wait-for-wake; principal_wake in crates/bridle-daemon/src/server.rs, Waiters in wake.rs) with a reason: shutting down; restarting for an internal upgrade (to which build); restarting by request (by whom). The waiter prints the reason on stderr and exits with its own code, not the timeout's 4: use 6 ('daemon restarting or shutting down; re-arm once it is back'), since 75h2 part 2 (br-dahs) takes 5 for 'superseded'. --json output stays clean. Post one event for the restart/shutdown with its reason and the number of waiters ended. Find the planned-restart and shutdown paths (upgrade, restart handler, graceful stop) and hook there, before the listener closes; an unplanned crash can't do this, and that's fine. Update docs/design/agent-host/ (wake, exit codes, events) and cli.md. Acceptance: just check passes; tests: a waiter open during a planned restart gets exit 6 with the reason, a real timeout still exits 4, the event is recorded. br-dahs touches the same wake code and runs after this (dependency edge). Model: Sonnet. Out of scope: replacement/--stop (br-dahs), unplanned crashes.

## Thread

### note · external:orchestrator · 2026-10-05T02:27:09.827Z
Readied by orchestrator: the human, via aide (m-5240, 2026-10-04 ~10:15 PM ET): "Schedule it."

### note · agent:waiter-reason · 2026-10-05T10:43:12.769Z
done: waiters ended with reason before planned restart/shutdown, exit 6, daemon.stopping records reason + waiters_ended; just check exit 0, 1222 tests passed; 1349e7c6

### note · agent:manager-2 · 2026-10-05T10:44:07.709Z
main moved: merge main into your branch, rerun just check, then report the new sha with exit status and test count.

### note · agent:waiter-reason · 2026-10-05T10:46:33.652Z
done: main merged; just check exit 0, 1222 tests passed; d79aafa1

### note · agent:manager-2 · 2026-10-05T10:46:38.950Z
integrated: 7c1accb724f73fe0c217c99e2ff7d3fcd123efcc (branch bridle/waiter-reason)
