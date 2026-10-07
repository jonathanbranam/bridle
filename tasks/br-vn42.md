+++
id = "br-vn42"
title = "Upgrades never go through under load: drain (no new turns, no timeout), then restart"
kind = "bug"
state = "planned"
created_at = "2026-10-07T02:10:31.279Z"
updated_at = "2026-10-07T02:10:55.522189Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: vn42
URGENT (the human's direct ask, 2026-10-07 ~10:30 PM ET; takes the next free worker slot, stay at two workers). Ticket (the human's words, analysis, fix, accepted/not-yet/rejected, verify; read all of it, it IS the design): docs/tickets/open/upgrades-never-go-through-under-load-drain-no-new-turns-no-t-vn42.md
Bug: an upgrade (or `bridle daemon restart`) waits for a quiet point by chance: `self_upgrade_tick` only starts at a quiet tick, the restart wait gives up after 600 s (`RESTART_WAIT`, 409) and the commit is not retried, and only spawns are held while every running agent still gets new turns. Under load it never restarts.
Files: crates/bridle-daemon/src/server.rs (`restart`, `perform_restart`, `self_upgrade_tick`), crates/bridle-daemon/src/restart.rs, crates/bridle-daemon/src/upgrade.rs, the supervisor's message/turn delivery path, docs/design/agent-host/daemon.md ("Restart in place", "Upgrade", "Automatic upgrade"), docs/design/cli.md. NOTHING else may touch these in parallel: the manager keeps other tasks that touch them after this one.
EXACT DESIGN (from the ticket; names fixed):
- Build starts at once when an upgrade is needed; `self_upgrade_tick` no longer waits for a quiet tick.
- After a good build and self-check the daemon enters a state named `draining`: spawns refused (as today), no task claims, and NO NEW TURNS for any running agent: messages, task updates/comments, queue nudges and wakes are stored (unread, in order) but not delivered as turns until after the restart's resume. The queue stays as it is. Turns already in progress run to their end; nothing is cut off. Interactive sessions (orchestrator, aide, advisors) are outside the daemon's turns and are not held.
- NO timeout: delete `RESTART_WAIT` and the 409 "no quiet point" path. When no agent is mid-turn and no spawn is in flight, restart in place and resume everyone, as today. A failed BUILD or self-check still fails the upgrade as today (that is not a drain problem); keep the 3 h retry for build failures only.
- `bridle status` shows exactly: `upgrade <sha> draining; waiting on <agent names mid-turn>`.
- If the drain is still waiting after 1 hour, wake the orchestrator ONCE with event/wake kind `upgrade_draining` naming the agents still in a turn. No other escalation; a stuck turn is the stall detector's job.
- A plain `bridle daemon restart` (no `--upgrade`) drains the same way. `daemon restart --wait`: read what it does today; keep it meaning "block until the new daemon answers" if it already means that, otherwise reword it; remove only the 600 s limit. Say what you did in the done note.
- The resume note already says the daemon restarted; add "re-run any background job you were waiting on" (accepted for now: ticket w8bz).
- A newer commit landing during a drain does not rebuild; the drained restart uses the built commit.
- Not building: cancelling a drain, draining other projects' daemons together.
Tests (the ticket's Verify, all with a fake build and fake claude, no live tests): (1) an agent mid-turn, messages arriving for it and for an idle agent: the upgrade holds the new turns, restarts when the turn ends (no timeout), delivers the held messages after the resume, in order, none lost or duplicated; (2) `self_upgrade_tick` starts the build while agents are busy; (3) spawns and claims are refused while draining; (4) status line text exactly as above; (5) the 1-hour `upgrade_draining` wake fires once (fake clock). Docs and CHANGELOG entry (read with a limit) updated in the same change.
Acceptance: just check passes. Migration: none, but the held-message state must survive the restart (it is stored unread, so the normal resume delivers it); say how you checked.
Model: Sonnet. Out of scope: the stall detector, w8bz, other daemons.

## Thread

### note · external:orchestrator · 2026-10-07T02:10:37.746Z
From orchestrator: br-vn42 is urgent, by the human's direct ask tonight (quote on the ticket vn42): upgrades must go through (drain: no new turns, no timeout, then restart). The ticket has the analysis, the fix and the verify list; brief it from there, Sonnet, and put it in tier 1 ahead of everything else so it takes the next free worker slot (stay at two workers). It touches server.rs, restart.rs, upgrade.rs, the message delivery path in the supervisor, daemon.md and cli.md: keep anything else touching those after it.
