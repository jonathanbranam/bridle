+++
id = "br-fpde"
title = "Restart on Linux execs '<path> (deleted)' after the binary is replaced"
kind = "bug"
state = "planned"
created_at = "2026-10-09T23:27:58.526Z"
updated_at = "2026-10-10T04:17:05.640050Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
summary = "Item 2 (agent_path using crate::exe_path()) was already in place on main, with a unit test; no change. Item 1: when the exec after a restart-requested shutdown fails and no upgrade rollback is pending, `run` (crates/bridle-daemon/src/lib.rs) no longer exits: it calls `start_after_failed_restart`, a fresh in-process `start`. The shutdown order closes the listener and stops the agents before the exec, so it re-binds (new listener), resumes the agents the restart recorded, and runs with a fresh, non-draining state, so messages held in the store during the drain are delivered (lift_drain was not needed on this path; perform_restart already lifts it for errors before shutdown). The error is logged, emitted as a new `restart.failed` event (bridle-api event_kind::RESTART_FAILED) and woken to the orchestrator (`restart_failed`). Test: restart_in_place_test a_failed_exec_leaves_the_daemon_serving_with_the_drain_lifted. Docs: daemon.md, CHANGELOG. Caveat: full `just check` could not go green on a host at load 40-55: failures were cli_e2e sigint (spawn held by machine-load guard) and events_stream shutdown timeout (passes alone); the 1003 tests that ran passed."
ticket = "fpde"
+++

Ticket: docs/tickets/open/restart-on-linux-execs-path-deleted-after-the-binary-is-repl-fpde.md (read it, especially "Wanted" and "Status"; the path fix itself landed as br-7b79). Approved by the human 2026-10-09. Model: Sonnet.

Build the two remaining Status items:
1. Stay up when the exec fails. Today a failed exec (restart, self-upgrade, rollback) still exits after a clean drain and shutdown, so an unattended machine loses its daemon. exec returns only on failure, so the process is alive: find the restart path (crates/bridle-daemon/src/lib.rs exec_self and roll_back; server.rs perform_restart; upgrade.rs) and make a failed exec lift the drain, resume serving (re-bind or keep the listener, whichever the shutdown order allows; read it first and say which in the task comment), deliver messages held during the drain (br-b6mu's note: lifting a drain after a failed restart must do this; check whether b6mu already did), record the error as an event and in the log, and tell the orchestrator. Do not exit.
2. `agent_path()` in crates/bridle-daemon/src/supervisor.rs (~line 2952) still calls current_exe(); use crate::exe_path() like the other callers, so a Linux daemon after a binary replace hands agents a real path, not one ending in " (deleted)".
Tests: a unit test that a failed exec (inject a restart function that returns Err, or an exec target that does not exist) leaves the daemon serving and the drain lifted; a unit test for agent_path using the resolved path. A Linux-only replace-the-binary test only if cheap.
Docs: docs/design/agent-host/daemon.md (restart and automatic upgrade sections), CHANGELOG.md. Update the ticket's Status? No: the manager resolves tickets.
Acceptance: just check passes. Migration: none. Out of scope: an external supervisor; batching of upgrades (br-7ufd, which touches the same watcher: merge main first if it is in flight).

## Thread

### note · external:advisor/product-manager · 2026-10-09T23:28:06.858Z
advisor (product-manager): build the ticket's remaining Status items: (1) stay up when the exec fails (today a failed exec still exits after a clean drain), (2) agent_path() in supervisor.rs still uses current_exe(); give it the same resolved path. The path fix itself landed as br-7b79. Normal priority; pairs with the upgrade work (br-7ufd, br-88d4). Approved by the human 2026-10-09.

### note · agent:wfpde · 2026-10-10T04:17:05.640Z
not done, check not green: code+test+docs committed and main merged (50d0afc5). Targeted tests pass (incl. new failed-exec test); 3 full just-check runs failed only on load-sensitive tests (cli_e2e sigint: spawn held, machine load 3-7/core; events_stream shutdown timeout, passes alone) with host load 40-55. Item 2 (agent_path) was already on main. Need a quieter host or your call on landing. Summary written.
