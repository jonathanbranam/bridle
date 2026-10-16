+++
id = "br-x6q7"
title = "Upgrade drain: hold the landing lock through the restart, raise an incident at the cap, stop the land's check process group (follow-up to d7r5)"
kind = "fix"
state = "pending"
created_at = "2026-10-16T01:05:49.484Z"
updated_at = "2026-10-16T01:05:53.747558Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "x6q7"
+++

docs/tickets/open/upgrade-drain-hold-the-landing-lock-through-the-restart-rais-x6q7.md

## Thread

### note · agent:pm-1 · 2026-10-16T01:05:53.747Z
Brief (pm-1). Model: Sonnet. Size: S/M. Read first: the ask above, the d7r5 thread and its landed diff (git log --grep d7r5), docs/design/agent-host/daemon.md "Automatic upgrade". Code: crates/bridle-daemon/src/server.rs (land_task ~3507, state.landing ~3534, drain_and_restart ~5203). Do, in order, stopping at the first boundary that gets long and telling the manager: (1) at the quiet point the drain takes the landing lock and holds it through the restart; a land arriving later is refused with "retry after the restart" (match how spawns are refused while draining). (2) At the d7r5 cap, raise an event/incident naming the land (use the daemon's existing incident/notice mechanism, as br-36a3 did) instead of restarting silently. (3) If the cap forces a restart mid-land, kill that land's `just check` process group before the exec. (4) The drain status line/wake text names the land and minutes waited, if d7r5 did not. Tests: land arriving after the quiet check is refused; cap raises the notice; a forced restart kills the check's process group (fake slow check). Docs: daemon.md, CHANGELOG. Acceptance: just check. Out of scope: br-8swr's wider epic. No migration.
