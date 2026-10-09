+++
id = "br-fpde"
title = "Restart on Linux execs '<path> (deleted)' after the binary is replaced"
kind = "bug"
state = "planned"
created_at = "2026-10-09T23:27:58.526Z"
updated_at = "2026-10-09T23:29:07.453137Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
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
