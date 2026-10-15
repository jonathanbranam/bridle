+++
id = "br-d7r5"
title = "Self-upgrade restarts in the middle of a task land's background check, so the land is lost and redone"
kind = "bug"
state = "planned"
created_at = "2026-10-11T18:15:25.764Z"
updated_at = "2026-10-15T23:04:02.698844Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
