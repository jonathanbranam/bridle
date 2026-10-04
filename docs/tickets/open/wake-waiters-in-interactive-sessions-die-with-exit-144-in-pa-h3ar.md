---
id: h3ar
title: Wake waiters in interactive sessions die with exit 144, in pairs across sessions and projects
kind: incident
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-h3ar]
---

## The ask

The human, verbatim (2026-10-04 ~7:30 PM ET, via the aide):

> this constitutes an incident. Please file it. It's happened to other agents as well. Looked like I
> saw it killed with maybe a 144. The waiters are getting killed all over the place or dying for some
> reason.

## Evidence (from Claude Code's background-task outputs on dalek, `/private/tmp/claude-501/*/*/tasks/*.output`)

Every background command on 2026-10-04 that ended `[exited with code 144]` with no other output
(times ET, from the output files' mtimes):

| Time (ET) | Session (cwd) | Command |
|---|---|---|
| 12:53:23 | bridle (advisor) | `bridle agent wake external:advisor --timeout 5400` |
| 12:53:23 | bridle (advisor doc-review) | `bridle agent wake external:advisor/doc-review --timeout 5400` |
| 14:23:39 | track-web (advisor) | `bridle agent wake external:advisor --timeout 5400` |
| 17:00:59 | bridle (advisor tickets) | `bridle agent wake external:advisor/tickets --timeout 5400` |
| 17:28:01 | bridle (aide) | `bridle agent wake external:aide --timeout 5400` |
| 17:28:01 | track-web (aide) | `bridle agent wake external:aide --timeout 5400` |
| 17:34:35 | wt/advisor-brief (worker) | four `sleep 60/120 && tail /tmp/br-6d49-check.log` |
| 18:20:07 | bridle (aide) | `bridle agent wake external:aide --timeout 5400` |
| 18:20:07 | track-web (aide) | `bridle agent wake external:aide --timeout 5400` |
| 20:05:59, 20:50:58 | wt/gateway-discovery, wt/gateway-actions (workers) | not identified |

What it shows:

- **Kills land in pairs at the same second in separate Claude Code sessions**, and across
  projects: two bridle advisors at 12:53:23, and bridle's and track-web's aides at 17:28:01 and
  again at 18:20:07. Those talk to **different daemons** (7401 and track-web's). Whatever kills
  them is outside a single session and a single daemon.
- **Not a daemon restart**: bridle's daemon ran from 16:08Z without a restart, and its log has
  nothing at 21:28Z or 22:20Z (17:28 and 18:20 ET).
- **Not only bridle**: plain `sleep && tail` jobs in a worker died the same way (17:34:35, four at
  once, possibly that worker ending its turn, a separate and expected case).
- **Exit 144 = 128 + 16.** On macOS signal 16 is `SIGURG`, whose default action is to ignore, so it
  shouldn't kill a process. Either Claude Code reports a kill this way, or the number isn't a
  signal. `bridle` never exits 144 itself (no such code in `crates/bridle`).
- The binary was last installed at 17:09:20 ET, which matches none of the times.
- Effect: the session stops waiting and nothing tells it. The aide noticed only because the task
  notification said "failed". An orchestrator or advisor that misses it goes deaf until something
  else wakes it. The system's "no wake command running" notices catch the orchestrator only.

## To find out

1. What sends the signal: run a waiter under `dtruss`/`log stream` or wrap it to log the signal it
   gets; check whether Claude Code kills background tasks (for example on a terminal or tmux event,
   or a memory or timeout cap) and how it reports that.
2. What happened at 17:28:01 and 18:20:07 ET on dalek (tmux, the terminal, system logs:
   `log show --start ... --end ...`).

## Mitigations to weigh

- Roles that wait (orchestrator, advisor, aide): treat any non-zero exit other than the timeout
  (exit 4) as "re-arm now", which is mostly what the prompts already say. The cost is only a missed
  wake for the gap.
- The daemon already records `waiter_open`/`last_wake_at`. Extend the "no wake command running"
  notice from the orchestrator to every registered interactive session (advisor, aide), so a dead
  waiter is visible.
