---
id: br-eggc
title: "Drain follow-up to d7r5: hold off new lands through the restart, raise an incident when the cap fires, no orphaned check"
kind: fix
created: 2026-10-16
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-d7r5, br-8swr]
---

Follow-up to br-d7r5, which landed item 1 of the design the human approved (2026-10-15 ~9:05 PM ET: "Yeah, that's great. Update that with the design suggestion, and let's get that ticket moving as well"): the drain waits on a land in progress (a Lands guard beside `state.landing`), the status line names the land, 2 h cap. The design is on br-d7r5's thread (advisor product-manager, 2026-10-16 01:05Z). Not built, per manager-2 (m-13823):

## The ask

1. **Close the gap.** At the drain's quiet point, hold the landing lock (or the Lands guard) through the restart, so a land that arrives after the quiet check waits until after the restart; or `land_task` refuses while draining with "retry after the restart", as spawns do. Today a land can start between the quiet check and the exec and be cut off.
2. **Say so when the cap fires.** When a land outlasts the 2 h cap and the drain restarts anyway, raise an event and an incident naming the land and how long it ran. No silent restart over a land.
3. **No orphaned check.** If a restart goes ahead mid-land (the cap), stop that land's `just check` process group before the exec. Restart-in-place keeps the daemon's pid, so today the check survives as an orphan holding the machine-wide test lock and the integration worktree, and the re-run land's check starts beside it (seen 2026-10-14, nncj; docs/context/incidents.md 2026-10-14 11:50Z).

Tests: a land arriving during a drain waits (or is refused) and runs after the restart; a land past the cap gives one incident; the forced restart leaves no `just check` from that land running.
