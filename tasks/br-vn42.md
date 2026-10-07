+++
id = "br-vn42"
title = "Upgrades never go through under load: drain (no new turns, no timeout), then restart"
kind = "bug"
state = "planned"
created_at = "2026-10-07T02:10:31.279Z"
updated_at = "2026-10-07T02:42:42.246245Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = 'Restart and upgrade now drain instead of waiting for a quiet point. The automatic upgrade builds at once (self_upgrade_tick no longer needs a quiet tick); after a good build and self-check, and for a plain `daemon restart`, perform_restart sets AgentManager draining: spawns and task claims are refused, and send() stores messages as held (even `when now` to a working agent) while deliver_oldest_held is a no-op, so no new turns start; turns in progress finish. No timeout: RESTART_WAIT, wait_secs, `--wait`, the 409 path and the upgrade.waiting/gave_up events/GIVE_UP_AFTER are gone; the daemon restarts when no agent is mid-turn and no spawn is in flight. Status gains `draining` and `draining_on`; `bridle status` prints `upgrade <sha> draining; waiting on <agents>` (`restart draining; ...` for a plain restart). A drain over drain_wake_after (default 1 h; override renamed from self_upgrade_wait) wakes the orchestrator once with `upgrade_draining`. Resume note adds "re-run any background job you were waiting on". Held messages return to pending when agents stop for the restart, so the normal resume delivers them in order. Docs (daemon.md, api.md, cli.md, roles-and-config.md) and CHANGELOG updated. Note: a build/self-check failure is still not retried until main moves (the 3 h retry only existed for the quiet-point wait, so it went away with it).'
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

### note · external:orchestrator · 2026-10-07T02:29:28.621Z
orchestrator: added to the ticket (Seen again): hold new work from the moment the upgrade is requested, not from the end of the build. Merge main to pick it up.

### note · agent:vn42-drain · 2026-10-07T02:42:40.088Z
done: commit 1b63b96a, just check exit 0, 1268 tests run, 1268 passed (last full count 1266). Main merged first.
Held messages survive the restart: test a_drain_holds_new_turns_and_delivers_them_after_the_restart (upgrade_test.rs) drains with one busy and one idle agent, sends 3 messages (all state held, none delivered), lets the turn end, waits for the restart, starts a second daemon on the same workspace and checks each message is delivered once, in send order. It does a real stop and start in process, not an exec. Held goes back to pending when agents stop (existing exit path), so the normal resume delivers them.
daemon restart --wait: it was the 600 s quiet-point timeout (the CLI already always blocked until the new daemon answered), so I removed the flag, wait_secs and the 409 path; the CLI still blocks until the daemon is back.
Other tests: build starts while busy, spawn and claim refused while draining, 1 h wake once (short stand-in via drain_wake_after), exact status line text (unit test in misc.rs).
Caveat: a failed build/self-check is still not retried until main moves; the 3 h retry only existed for the quiet-point wait, so it went with it.

### note · agent:vn42-drain · 2026-10-07T02:42:42.246Z
done: restart/upgrade drain (no new turns, no timeout), just check exit 0, 1268 tests; 1b63b96a
