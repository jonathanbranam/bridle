+++
id = "br-eggc"
title = "Drain follow-up to d7r5: hold off new lands through the restart, raise an incident when the cap fires, no orphaned check"
kind = "fix"
state = "planned"
created_at = "2026-10-16T01:06:01.801Z"
updated_at = "2026-10-16T01:17:04.190596Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "eggc"
+++

docs/tickets/open/drain-follow-up-to-d7r5-hold-off-new-lands-through-the-resta-eggc.md

## Thread

### note · external:advisor/product-manager · 2026-10-16T01:06:11.155Z
Readied by advisor product-manager under the design the human approved on br-d7r5 (2026-10-15 ~9:05 PM ET).

### note · agent:pm-1 · 2026-10-16T01:06:17.085Z
Brief (pm-1). Model: Sonnet. Size: S/M. Read first: the ask above, the d7r5 thread and landed diff (git log --grep d7r5), docs/design/agent-host/daemon.md "Automatic upgrade". Code: crates/bridle-daemon/src/server.rs (land_task ~3507, state.landing ~3534, the Lands guard d7r5 added, drain_and_restart ~5203). Do in the ask's order, stopping at the first boundary that gets long and telling the manager: (1) hold the guard/lock through the restart, and refuse a late land with "retry after the restart" (match how spawns are refused while draining); (2) at the cap, raise an event/incident naming the land and how long it ran (use the daemon's existing incident/notice mechanism, as br-36a3 did); (3) on a forced restart mid-land, kill that land's `just check` process group before the exec (own the pgid when the land spawns it; never kill by name, rule no-kill-by-name). Tests as in the ask (fake slow check). Docs: daemon.md, CHANGELOG. Acceptance: just check. Out of scope: br-8swr's wider epic. No migration.
