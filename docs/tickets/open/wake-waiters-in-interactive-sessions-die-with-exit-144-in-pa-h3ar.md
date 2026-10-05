---
id: h3ar
title: Interactive sessions kill each other's wake waiters with pkill -f (exit 144)
kind: incident
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [m7mp, j28f, kuw2]
tasks: [br-h3ar]
---

## The ask

The human, verbatim (2026-10-04 ~6:30 PM ET, via the aide):

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
| 18:37:22 | bridle (aide) | `bridle agent wake external:aide --timeout 5400` |
| 18:37:22 | bridle-ui (a session) | a background command that had just run `bridle task comment` (output "commented on ui-pmkd") |
| 20:05:59, 20:50:58 | wt/gateway-discovery, wt/gateway-actions (workers) | not identified |

What it shows:

- **Kills land in pairs at the same second in separate Claude Code sessions**, and across
  projects: two bridle advisors at 12:53:23, and bridle's and track-web's aides at 17:28:01 and
  again at 18:20:07; bridle's aide and a bridle-ui session at 18:37:22. Those talk to **different daemons** (7401 and track-web's). Whatever kills
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

## Cause found (2026-10-04 22:46Z, the aide)

**Interactive sessions kill each other's waiters with `pkill -f`.** To stop its own old waiter before
starting a new one, a session ran `pkill -f "bridle agent wake external:aide"` (or
`external:advisor`). Identities aren't unique across projects (every project's aide is
`external:aide`), and `external:advisor` is a prefix of `external:advisor/doc-review` and the rest.
So the pattern matched **every** aide's or advisor's waiter on the machine. Each kill lines up to
the second with a `pkill` in a transcript (`~/.claude/projects/*/*.jsonl`):

| pkill (UTC) | run by | pattern | killed (ET) |
|---|---|---|---|
| 16:53:23.05 | track-web advisor | `bridle agent wake external:advisor` | two bridle advisors, 12:53:23 |
| 18:23:38.5 | bridle advisor | `bridle agent wake external:advisor --timeout 5400` | track-web advisor, 14:23:39 |
| 21:28:00.8 | bridle-ui aide | `bridle agent wake external:aide` | bridle's and track-web's aides, 17:28:01 |
| 22:20:06.2 | bridle-ui aide | `bridle agent wake external:aide` | bridle's and track-web's aides, 18:20:07 |
| 22:37:21.9 | track-web aide | `bridle agent wake external:aide` | bridle's aide and bridle-ui's, 18:37:22 |
| 22:45:25.4 | bridle-ui aide | `bridle agent wake ...` | bridle's, track-web's and two of bridle-ui's own, 18:45:27 |

Other `pkill -f` on waiters today, with no victim confirmed: `wait-for-wake --project track-web`
(12:56Z, 21:27Z), `wait-for-wake --project bridle-ui` (19:59Z), and `agent wake
external:advisor/doc-review` (15:25Z, 19:48Z).

This breaks an existing rule: `workflow/base/rules/no-kill-by-name.md` (never `pkill -f`, written after
fx7x, when workers' `pkill -f "just check"` killed the orchestrator). Interactive roles didn't reliably
get the rules at startup (m7mp, which just landed, addresses that). The aide and advisor prompts say
to wait with one background command but not how to cancel one. Exit 144 is how Claude Code reported
the killed command (pkill sends SIGTERM).

## Fix (replaces "To find out")

1. **Interactive roles get and follow no-kill-by-name** (check m7mp covers aide and advisor). The
   aide, advisor and orchestrator prompts say how to replace a waiter: never kill it by name. Leave it
   running (whether two waits for one identity both get woken is unverified; see 2). Or stop it with
   Claude Code's own task stop for that task id. Or `kill` the exact pid you started.
2. **Make the daemon's waiter the single source of truth**: a second `bridle agent wake` for the same
   identity takes over from the first (the old one exits 0 with "superseded"), so no session ever
   needs to kill one.
3. **Identities unique per machine**, or patterns can't collide: covered by the j28f/kuw2 discussions
   (project in the identity).
4. Keep the mitigation above: warn any interactive session with no waiter, not just the orchestrator.

**Gap found by bridle-ui's aide (2026-10-04):** `workflow/base/rules/no-kill-by-name.md` lists its
roles as orchestrator, project-manager, manager, worker, reviewer and advisor, **not aide**, so the
aide's priming never includes it. Fix 1 must add `aide` (and check every interactive role). Both
other aides confirmed they ran the `pkill -f` to clear a waiter they'd started wrongly (backgrounded
with `&`, or output sent to /dev/null). The prompts should also say how to start a waiter correctly.

## The human's answer (2026-10-04 ~8:50 PM ET, via the aide)

> It doesn't seem like H3AR has anything in it that needs approval. I don't really understand the
> question there. I thought I approved some of this work already, and I don't know why nobody can
> edit the workflow-based rules, but I can edit it now. Send me in today for that. That's fine.

No approval is needed for the fixes. The human makes the `aide` edit to the locked
`no-kill-by-name.md` themselves: to-do br-wbg2.
