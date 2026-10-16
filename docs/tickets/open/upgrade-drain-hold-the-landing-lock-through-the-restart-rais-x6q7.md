---
id: br-x6q7
title: "Upgrade drain: hold the landing lock through the restart, raise an incident at the cap, stop the land's check process group (follow-up to d7r5)"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Follow-up to br-d7r5, which landed only part 1 of the design suggestion (the drain waits on a land in progress, 2h cap). The rest, from the PdM's suggestion on the d7r5 thread (the human approved it, 2026-10-15):

- A land is one daemon call, land_task (crates/bridle-daemon/src/server.rs), holding state.landing for the whole land. The drain, drain_and_restart, restarts when no agent is mid-turn, no spawn is in flight and no postpone is set.
- Close the gap: at the quiet point the drain takes the landing lock itself and holds it through the restart, so a land arriving after the check waits or land_task refuses ("retry after the restart"), as spawns do.
- At the cap, if a land holds the lock too long, the drain stops waiting and raises an event/incident naming the land. No silent kill.
- If the cap forces a restart mid-land, stop that land's just check process group first, so no orphaned check survives the exec holding the test lock and worktree (the Oct 14 case).
- The status line and upgrade_draining wake say "waiting on landing br-xxxx, N min" (check whether d7r5 already did this).
