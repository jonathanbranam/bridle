+++
id = "br-d7r5"
title = "Self-upgrade restarts in the middle of a task land's background check, so the land is lost and redone"
kind = "bug"
state = "integrated"
created_at = "2026-10-11T18:15:25.764Z"
updated_at = "2026-10-16T01:15:34.227432Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:d7r5",
]
branch = "bridle/d7r5"
commit = "f44743a1a253a0410170723c5db245d9f99b5116"
summary = "Part (1) done: server.rs gains a Lands registry (AppState.lands) held by a guard for the life of land_task; drain_and_restart counts lands younger than LAND_DRAIN_CAP (2 h) as busy, so a restart waits for a land, and a stuck land cannot block it forever. The 1 h upgrade_draining wake names the landing. Test: Lands hold/cap/guard-drop (unit; the drain loop itself is not covered, it execs). Docs: daemon.md, CHANGELOG. Part (2) (orphaned check surviving restart-in-place, holding the test lock and integration worktree) not done: follow-up. Note: sessions::the_hard_limit_has_no_override_and_forces_a_restart failed once under load (ENOENT reading a 'calls' file), passed on rerun; unrelated, possibly flaky. br-8swr stays pending; mention there when this lands."
ticket = "d7r5"
+++

docs/tickets/open/self-upgrade-restarts-in-the-middle-of-a-task-land-s-backgro-d7r5.md

## Thread

### note · external:orchestrator · 2026-10-14T12:52:05.298Z
orchestrator, 2026-10-14 ~12:55Z: seen again (nncj). New detail: the land's integration 'just check' is a child of the daemon (pid 88537), and restart-in-place execs the same pid, so the killed land's check survives the restart, orphaned: it kept the machine-wide test lock and the integration worktree, and the re-run land's own 'just check' (same worktree) started beside it and queued for the lock. So a killed land costs its check twice, plus a second check running in the same worktree. Overlaps br-8swr; incident in docs/context/incidents.md (2026-10-14 11:50Z).

### note · external:advisor/product-manager · 2026-10-15T22:52:37.964Z
Readied by the human via advisor product-manager, 2026-10-15 ~6:55 PM ET: "Ready more tasks" (advisor picked this one).

### note · agent:pm-1 · 2026-10-15T22:53:18.067Z
Brief (pm-1). Model: Sonnet. Size: M, design care needed. The human readied this; it is the same defect as br-8swr (full fix; its thread has the evidence and br-cvfu already landed the manual postpone/cancel). Read both threads, br-cvfu's change (bridle daemon restart --cancel/--postpone, held while a land runs) and docs/design/agent-host/daemon.md "Automatic upgrade" first. Do: (1) the drain counts a land in progress (its check, merge, push and done) as busy, like a turn, so the restart waits for it; find where the drain decides idle (git grep -n "drain" crates/bridle-daemon/src/server.rs supervisor.rs) and how `bridle ticket land` / `task land` runs the check in the background; register the land (a counter or a guard held for its life) so a bounded land delays the restart, never blocks it forever (cap it, say the land's own timeout). (2) Restart-in-place execs the same pid, so a land check that survives is orphaned and holds the test lock and integration worktree: make the restart (or startup) detect and not collide with an orphaned check in the integration worktree, only if it is small; otherwise note it on the thread as a follow-up. Tests: drain with a land in progress waits then proceeds when it ends; a land that exceeds the cap does not block the restart forever. Docs: daemon.md, CHANGELOG. Acceptance: just check. Stop point: land (1) alone if (2) grows; tell the manager. Overlap: br-8swr stays pending for the human; mention on it when this lands. Do not remove the cvfu postpone.

### note · agent:d7r5 · 2026-10-16T00:59:31.550Z
built: 25ca69eee772a6d9c1adadcc0c1c606b7ea2e979 (branch bridle/d7r5)

### note · agent:d7r5 · 2026-10-16T00:59:31.595Z
done: restart drain waits for a land in progress (2h cap); part (2) orphaned check left as follow-up; just check exit 0, 1723 tests; 25ca69eee772a6d9c1adadcc0c1c606b7ea2e979

### note · external:advisor/product-manager · 2026-10-16T01:05:31.588Z
Design suggestion (advisor product-manager, 2026-10-15; the human approved putting it here: "Yeah, that's great. Update that with the design suggestion"). Read in code:

- A land is one daemon call, `land_task` (crates/bridle-daemon/src/server.rs ~3507). It already takes `state.landing` (an Arc<tokio Mutex>, ~3534) and holds it for the whole land: merge, check, push, done.
- The upgrade's drain, `drain_and_restart` (~5203), restarts when no agent is mid-turn, no spawn is in flight and no postpone is set. It never looks at `state.landing`. That is the bug.

Fix:
1. Quiet also requires the landing lock to be free (`try_lock` succeeds). While a land holds it, the drain waits, and the status line and the `upgrade_draining` wake say "waiting on landing br-xxxx, N min".
2. Close the gap: at the quiet point the drain takes the landing lock itself and holds it through the restart, so a land arriving after the check waits (or `land_task` refuses while draining, "retry after the restart", as spawns do).
3. Cap: if a land holds the lock past its own bound (one `just check` plus margin), the drain stops waiting and raises an event/incident naming the land. No silent kill.
4. If the cap forces a restart mid-land, stop that land's `just check` process group first, so no orphaned check survives the exec holding the test lock and worktree (the Oct 14 case).

Test: a land with a slow fake check; a restart requested meanwhile waits until the land is done, then restarts.

### note · agent:d7r5 · 2026-10-16T01:05:37.915Z
d7r5: read the advisor's design suggestion after building. Built covers its items 1 (drain waits on a land; uses a land registry held for land_task's life, not try_lock on state.landing, same effect plus it counts a land queued for the lock) and 3 partly (2 h cap, no event/incident yet; the wake text names the landing). Not built: 2 (a land arriving after the quiet check, before the exec) and 4 (kill the check's process group on a cap-forced restart). Each is a follow-up; say if you want them on this branch.

### note · agent:d7r5 · 2026-10-16T01:05:37.970Z
advisor design suggestion arrived after build: covers items 1 and part of 3; items 2 and 4 not built, see ticket comment. Say if you want them added.

### note · agent:manager-2 · 2026-10-16T01:05:39.161Z
manager-2: the land of 25ca69ee is running; it already covers item 1 (drain waits on a land in progress, status says 'landing <ids>', 2h cap) via a Lands guard beside state.landing. Not covered, left for a follow-up ticket from the PM: item 2 (drain holds the landing lock through the restart / land_task refuses while draining), item 3 (event/incident when the cap forces a restart), item 4 (stop the land's just check process group before the exec, the orphaned-check case).

### note · agent:pm-1 · 2026-10-16T01:05:40.099Z
pm-1, for the manager: d7r5 is built (25ca69ee: drain waits for a land, 2h cap). The PdM's design suggestion above (human approved) goes further. Before landing, check the built code against it and fold the missing points in if they are small, in this order of value: (a) the drain's quiet check uses `state.landing` try_lock (if the build used another signal, say why) and the status line / upgrade_draining wake name the land and minutes waited; (b) at the quiet point the drain holds the landing lock through the restart so a land arriving after the check waits or is refused with "retry after the restart"; (c) on the cap, raise an event/incident naming the land, no silent kill; (d) if the cap forces a restart mid-land, kill that land's `just check` process group first so no orphan keeps the test lock and worktree. If (b)-(d) are more than a small change on top of the build, land what is built and tell me: I will file one follow-up ticket for the rest (the PdM readies it). Tests as the suggestion says: slow fake check, restart requested meanwhile waits, then restarts.

### note · agent:manager-2 · 2026-10-16T01:15:09.162Z
integrated: f44743a1a253a0410170723c5db245d9f99b5116 (branch bridle/d7r5)

### note · agent:manager-2 · 2026-10-16T01:15:34.227Z
cleanup: removed agent d7r5, branch bridle/d7r5
