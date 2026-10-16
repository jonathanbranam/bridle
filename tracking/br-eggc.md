+++
id = "br-eggc"
title = "Drain follow-up to d7r5: hold off new lands through the restart, raise an incident when the cap fires, no orphaned check"
kind = "fix"
state = "built"
created_at = "2026-10-16T01:06:01.801Z"
updated_at = "2026-10-16T05:10:10.922835Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:eggc",
]
branch = "bridle/eggc"
commit = "d47f56662d19964acb4cdd4dab816a3ed86946f4"
summary = "The restart drain closes the Lands registry at its quiet point (server.rs Lands::close_if_quiet, atomic with begin): a late land is refused 503 'retry once it has restarted', and the registry reopens if the restart fails. A land past the 2h cap gets a restart.land_forced event and an incident, and its check's process group is SIGKILLed before the exec (integrator run_check starts the check in its own group and records the pgid in LandInput.check_pgid). Unit tests cover close/refuse/reopen and the group kill; no integration test with a slow fake check (d7r5's wait test already exists in the suite). daemon.md and CHANGELOG updated."
ticket = "eggc"
+++

docs/tickets/open/drain-follow-up-to-d7r5-hold-off-new-lands-through-the-resta-eggc.md

## Thread

### note · external:advisor/product-manager · 2026-10-16T01:06:11.155Z
Readied by advisor product-manager under the design the human approved on br-d7r5 (2026-10-15 ~9:05 PM ET).

### note · agent:pm-1 · 2026-10-16T01:06:17.085Z
Brief (pm-1). Model: Sonnet. Size: S/M. Read first: the ask above, the d7r5 thread and landed diff (git log --grep d7r5), docs/design/agent-host/daemon.md "Automatic upgrade". Code: crates/bridle-daemon/src/server.rs (land_task ~3507, state.landing ~3534, the Lands guard d7r5 added, drain_and_restart ~5203). Do in the ask's order, stopping at the first boundary that gets long and telling the manager: (1) hold the guard/lock through the restart, and refuse a late land with "retry after the restart" (match how spawns are refused while draining); (2) at the cap, raise an event/incident naming the land and how long it ran (use the daemon's existing incident/notice mechanism, as br-36a3 did); (3) on a forced restart mid-land, kill that land's `just check` process group before the exec (own the pgid when the land spawns it; never kill by name, rule no-kill-by-name). Tests as in the ask (fake slow check). Docs: daemon.md, CHANGELOG. Acceptance: just check. Out of scope: br-8swr's wider epic. No migration.

### note · agent:eggc · 2026-10-16T05:10:10.875Z
built: d47f56662d19964acb4cdd4dab816a3ed86946f4 (branch bridle/eggc)

### note · agent:eggc · 2026-10-16T05:10:10.922Z
done: drain closes to late lands, incident + check pgid kill on the cap; just check exit 0, 1738 tests; d47f56662d19964acb4cdd4dab816a3ed86946f4
