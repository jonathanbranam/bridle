---
id: vn42
title: "Upgrades never go through under load: drain (no new turns, no timeout), then restart"
kind: bug
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-vn42]
closed: 2026-10-07T03:38:01Z
---

## The ask

The human, 2026-10-07 ~10:30 PM ET (directly to the orchestrator), after the upgrade of 57983073
gave up with "no quiet point within 600s; still busy: page-title-rev":

"okay, that setup is not going to work, right? What if we had 5 workers? It would literally never
restart, so clearly there's a design problem here. Fix the design problem. Write up a ticket,
analyze it, and fix the design problem.

When we need an upgrade, it should get put through. That means no new work starts, and there's a
pending. We stop all work. The queue sits as it is. Everything that's currently being done
finishes, then we upgrade. No 10-minute timeout, none of that. Just get the upgrade through, okay?

We have to have upgrades, especially here. Right now, we're making changes rapidly. We need
upgrades. We don't have to kill everything to upgrade, but it has to happen"

## Analysis: why upgrades don't go through

The design (docs/design/agent-host/daemon.md, "Restart in place" and "Upgrade";
`crates/bridle-daemon/src/server.rs` `restart`, `perform_restart`, `self_upgrade_tick`;
`crates/bridle-daemon/src/restart.rs`) waits for a quiet point to happen by chance instead of
making one:

1. **The automatic upgrade only starts at a quiet tick.** `self_upgrade_tick` returns unless, at
   the one-minute CI tick, no running agent is mid-turn. With two or more workers in long turns
   that tick rarely comes, so the build doesn't even start.
2. **The wait gives up.** After the build, the restart waits up to 600 s (`RESTART_WAIT`) for every
   agent to be idle, then answers 409 and does nothing. A manual upgrade then counts as failed and
   that commit is not retried (in memory) until main moves; the automatic one retries for 3 h, then
   gives up on the commit too.
3. **Only spawns are held.** While a built commit waits, new worker spawns are refused, but every
   running agent still gets new turns: messages, task comments and updates, queue nudges, the
   manager telling a worker to merge main. Each one restarts the busy period, so "all idle at once"
   may never happen. More workers make it strictly worse.

Tonight (2026-10-07 01:35-02:05Z): built 57983073 at the human's go; `page-title-rev` stayed in one
turn the whole 600 s; `upgrade_failed`; spawns re-allowed; 57983073 not retried.

## The fix: drain, then upgrade

When an upgrade is needed (manual `restart --upgrade`, or `self_upgrade` finding a newer green
commit), it goes through:

- **Build first, at once.** No quiet point is needed to build (it runs in its own worktree and
  target dir). The automatic upgrade no longer waits for a quiet tick to start.
- **Then drain.** After a good build and self-check the daemon enters a draining state:
  - no new work starts: spawns refused (as today), no task claims, and no new turns for any
    running agent. Messages, task updates, nudges and wakes are still stored but not delivered as
    turns; they are delivered after the restart's resume;
  - the queue stays as it is;
  - every turn already in progress runs to its end. Nothing is cut off.
- **No timeout.** When no agent is mid-turn (and no spawn in flight) the daemon restarts in place
  and resumes everyone, as today. No 600 s limit, no give-up, no "not retried" for lack of a quiet
  point.
- **Visible.** `bridle status` shows `upgrade <sha> draining; waiting on <agents mid-turn>`.
- **Someone looks if it takes long**, without giving up: if the drain is still waiting after an
  hour, wake the orchestrator once (`upgrade_draining`, naming the agents still in a turn). A
  truly stuck turn is the stall detector's job, not the upgrade's.
- A plain `bridle daemon restart` (no `--upgrade`) drains the same way.

Interactive sessions (orchestrator, aide, advisors) are outside the daemon's turns and aren't held.

### Accepted for now

- An agent idle while its own background shell job runs (ticket w8bz) counts as idle, so the
  restart ends that job. The resume note already says the daemon restarted; it should add "re-run
  any background job you were waiting on". Worst case: one `just check` re-run.
- A newer commit landing during the drain doesn't rebuild; the drained restart uses the built
  commit and the next upgrade picks up the newer one.

### Not yet (reasons)

- A way to cancel a drain: not needed until a drain is wrong to finish; the human can stop the
  daemon by hand.
- Draining other projects' daemons together: each daemon upgrades itself.

### Rejected

- A longer timeout: still gives up under steady load, which is the bug.
- Killing turns to force the restart: the human: "We don't have to kill everything to upgrade."

### Seen again, 02:13-02:20Z: spawns allowed during the build

The second request (e4a617ea, `--wait 3600`) found every worker idle, but spawns are only refused
once the build is done. manager-2 spawned two workers (vn42-drain, hesj-focus-local) during the
~5 min build, so the quiet point was gone again. The fix: hold new work from the moment the upgrade
is requested (or the automatic upgrade picks a commit), not from the end of the build. The build
can't use the idle time otherwise; a failed build lifts the hold.

## Verify

- A daemon test: with an agent mid-turn and messages arriving for it and for an idle agent,
  an upgrade restart (a fake build) holds the new turns, restarts when the turn ends (no timeout),
  and delivers the held messages after the resume.
- `self_upgrade_tick` starts the build while agents are busy.
- docs/design/agent-host/daemon.md ("Restart in place", "Upgrade", "Automatic upgrade") and
  docs/design/cli.md (`daemon restart --wait` removed or reworded) match; CHANGELOG entry.
- `just check`.
