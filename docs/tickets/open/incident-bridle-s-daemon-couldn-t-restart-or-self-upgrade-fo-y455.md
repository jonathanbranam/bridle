---
id: y455
title: "Incident: bridle's daemon couldn't restart or self-upgrade for ~23 h: a stuck 'spawning' flag meant no quiet point ever came"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [2ax5, q7mv]
tasks: [br-y455, br-qect]
---

## The ask

The human, 2026-10-06 ~6:20 PM ET (verbatim): "File an incident and schedule a post-mortem." About the orchestrator's finding below.

## What happened (as known)

- bridle's daemon (pid 23474, launchd dev.bridle.bridle) could not restart or self-upgrade from about 2026-10-05 23:01Z until the human ran `launchctl kickstart -k` at 2026-10-06 22:12Z.
- Every graceful restart failed after its wait: "not restarted: no quiet point within 600s; still busy: a spawning agent". `bridle agents` showed every agent idle. server.rs (restart path) waits on `state.manager.spawning()`, which stayed true with no agent spawning: a leaked flag or counter.
- Likely trigger, unconfirmed: worker a-897zk (incident-2y3m) was stop-requested 3 s after `agent.spawned` (2026-10-06 03:20:31-36Z).
- Earlier self-upgrade failures were reported as "no quiet point ... still busy: usage-history" (2026-10-05 23:49Z), so the trigger may be older.
- Consequences: the daemon stayed on pre-br-3haz code, so it had no /v1/outbox and cross-project messaging broke (incident br-2ax5). bridle's self-upgrade never ran. The orchestrator couldn't restart it; only the human could.

## The postmortem must answer

- The exact cause of the stuck flag: which spawn path doesn't release it (stop during spawn, spawn error, a panic?). Reproduce it in a test.
- Since when it was stuck: daemon logs and events for the first "a spawning agent" refusal; why nobody noticed failing self-upgrades for about 23 h.
- Why a failed self-upgrade (repeated "no quiet point") raises no alert to the orchestrator or the human.
- Why the restart error doesn't name what holds the flag.
- Fixes: release on every exit path, an alert after N refused upgrades (cf. q7mv's recommendations for incident aqa7, which weren't acted on), and a safe forced restart the orchestrator may run.

Related: br-btdn (the bug), br-2ax5, q7mv (aqa7 postmortem), br-4zfa.

## Postmortem (br-qect, 2026-10-06)

Read-only investigation: daemon events (`bridle events`), `.bridle/daemon.log`, and the code in
`crates/bridle-daemon`. Nothing on the running daemon was touched.

### Summary

The only thing `manager.spawning()` reads is `Inner::spawning`, a counter (`supervisor.rs:191`) that
`AgentManager::spawn` raises and lowers through a drop guard (`supervisor.rs:803-814`). Because it
is a drop guard, **every return path, every error, a panic and a cancelled request release it**.
I found no spawn exit path that leaks it, and a test (below) shows that a stop during spawn, the
suspected trigger, does not. So the counter can only stay above zero while a `spawn` future is
still alive and never finishing. The root cause is therefore a spawn that hung, not one that
exited badly. Which await hung, and when, is **not proven**: the daemon was restarted by
`launchctl kickstart -k` (2026-10-06 22:12Z) and took the evidence with it. What I can show is the
set of unbounded awaits that can do it, and why nothing noticed (that part is fully explained).

### Timeline (UTC)

- 2026-10-05 23:01:46 - restart via POST /v1/restart; the new process (pid 23474) starts with the
  counter at 0 (log line 1823). Pre-br-3haz code.
- 23:22:19 - `warming worktree target/` (log); 23:29:16 - `agent.spawned` a-90rhi (usage-history,
  event 76858). The spawn sat in `warm_target` (`cp -cR` of a multi-GB `target/`, no timeout,
  `worktree.rs:164`) for about 7 min, so spawns can legitimately stay "in flight" for minutes.
- 23:34:39 upgrade.building 29296901d, 23:39:28 upgrade.built (events 76892, 76918); 23:49:28
  upgrade.failed, "no quiet point within 600s; still busy: usage-history" (event 76925, log line
  1843). That busy agent is a-90rhi. The message prints the busy agent names *instead of* "a
  spawning agent", so a stuck flag would have been invisible here: this refusal does not show
  the flag was clean.
- 23:58:31 - CI watch: `git ls-remote origin refs/heads/main: Permission denied (publickey)` (log
  line 1847). A second reason no new green commit was found (see "Why nothing noticed").
- 2026-10-06 01:00:15, 01:23:11, 02:54:13 - `warming worktree target/` lines with no matching
  `agent.spawned` (spawn requests that never produced an agent: failed, or the client gave up and
  the request was dropped). Candidates for the hang; not conclusive.
- 03:20:31.962 - `agent.spawned` a-897zk (incident-2y3m), event 77923; messages m-5906/m-5907
  sent in the next 10 ms; 03:20:33.063 working (init seen, so spawn's readiness wait was already
  satisfied); 03:20:36.147 `agent.stop_requested` by manager-2 (event 77937); 03:21:06.435
  stopped, exit 143, reason stdin_closed (event 77954). A clean stop. manager-2 reports its
  `bridle agent spawn` hung and it ran `bridle agent stop` once the agent appeared.
- 2026-10-06 21:34 - a-897zk removed (event 78041). 21:43 and 21:59 - new spawns succeed
  (events 78121, 78197): spawns still work while the flag is stuck; only the quiet-point check sees it.
- ~21:50 - orchestrator's `bridle daemon restart` fails after 600 s: "still busy: a spawning agent".
- 22:11:17 - upgrade.building 05d498f2 (event 78233) on the freshly kicked daemon; 22:12 human
  `launchctl kickstart -k`.

First "a spawning agent" refusal: **not recoverable**. Refusals of a manual restart go to the
caller only; neither the daemon log nor the event log records them (`perform_restart`,
`server.rs:3653-3680`, returns the error and logs nothing). The first logged refusal of any kind
is the 23:49:28 one above, which names a real agent.

### Answers to the questions

**1. Which spawn path doesn't release the flag?** None does. `spawn` (`supervisor.rs:800`) wraps
`spawn_agent` in an RAII `InFlight` guard. `spawn_agent`'s many `return Err(..)` paths (unknown
role, upgrade waiting, max_workers, name conflict, worktree/setup failure, `insert_agent`, token,
transcript, `process::spawn`) all drop it, as does cancellation of the HTTP request (the handler,
`server.rs:1010`, awaits it inline; a dropped future drops the guard) and a panic. The two
other callers (`lib.rs:1102` autostart, same function) are the same. `resume`/`renew` do not touch
the counter. So the counter is only stuck by a spawn that is parked forever. Awaits in
`spawn_agent` with no bound: `worktree::add` (a git subprocess), `warm_target`'s `cp -cR` (no
timeout, no `kill_on_drop`), the store calls (SQLite behind a mutex), `write_message`'s
`runtime.state.lock().await` (the tokio mutex shared with the agent's event task,
`supervisor.rs:3141`) and `fill_incident_notices`. The ones that are bounded: setup
(`setup_timeout`, 600 s) and the readiness wait (`SPAWN_READY_TIMEOUT`, 8 s, `supervisor.rs:1202`).
The a-897zk events show its spawn got past all of these (init, both messages written) within 3 s,
so if that spawn hung it was in the last step (`get_agent`) or in the HTTP reply, and I cannot tell
which. The suspected trigger "stop 3 s after spawned" is **refuted as a leak**: the agent stopped
cleanly at 03:21:06, and the reproduction below passes.

**Reproduction.** `crates/bridle-daemon/tests/spawning_flag_test.rs`,
`a_stop_during_spawn_leaves_no_spawning_flag`: spawns a worker with a prompt on the fake claude,
stops it the moment its row exists, then asks for a restart with a 3 s wait and expects it to be
accepted. It **passes today** (6 s), so there is no failing reproducer for br-btdn yet; the test is
committed un-ignored as a guard on the suspected path. Run it with
`cargo nextest run -p bridle-daemon --test spawning_flag_test`. A failing reproducer needs a way to
park a spawn forever (e.g. a test hook on one of the unbounded awaits above); I did not add a
production hook for it (out of scope). br-btdn should start from: put a bound on each unbounded
await, and make the counter observable, then the hang shows up the next time it happens.

**2. Since when stuck, and why nobody noticed.** Unknown start (see timeline); no evidence of a
refusal before the orchestrator's about 21:50Z attempt. The daemon never wakes anyone about
refused self-upgrades in this state: `self_upgrade_tick` (`server.rs:3441-3452`) returns **silently**
when `spawning` is true, before it claims an upgrade, so no `upgrade.*` event, no log line and no
wake exist for a tick skipped this way. From the 23:49:28 failure to 22:11:17 there is not a single
`upgrade.*` event: 22 h of nothing, which looks the same as "no new green commit". It also does
not help that CI watch was failing (`Permission denied (publickey)`, 23:58:31): "nothing to
upgrade" and "upgrade blocked" were indistinguishable from outside.

**3. Why a failed self-upgrade raises no alert.** For the automatic upgrade (`who == "system"`) a
refusal for "no quiet point" is treated as "try later" and escalated only after `GIVE_UP_AFTER`
(3 h, `upgrade.rs:73`), and only for an upgrade that got as far as a build
(`upgrade_in_background`, `server.rs:3585-3607`). A tick that is skipped before building (spawning
true, or not quiet) never starts that clock, so it never escalates. The 23:49 refusal, from the
older pre-br-3haz binary, logged a plain `upgrade failed` (log line 1843) and marked the commit
failed; nothing told the orchestrator or the human. This is the same gap q7mv recommended closing
for incident aqa7 ("after N refused builds wake the orchestrator and the human"): not acted on.

**4. Why the restart error doesn't name the holder.** `perform_restart` knows only the boolean
`manager.spawning()` (`server.rs:3653`) and prints "a spawning agent" whenever the *agent* busy
list is empty. The counter has no owner: it is a bare `AtomicUsize` with no record of which spawn,
which agent name or since when. When the busy list is non-empty the names are shown instead, which
hides a stuck counter behind a real busy agent.

### Recommendations (cheap first)

1. **Make the flag observable (fixes the diagnosis gap).** Replace the bare counter with a small
   registry (spawn id, agent name, started-at) behind the same `spawning()`; have `perform_restart`
   name them ("a spawning agent: incident-2y3m, 23 h") and show them in `bridle status`.
2. **Bound every await in `spawn_agent`**: a timeout on `worktree::add`, `warm_target` (add
   `kill_on_drop(true)`) and the whole `spawn` (say 15 min) that returns an error and releases the
   guard. This turns a hang into a logged failure. (The fix is br-btdn; the test above is its guard.)
3. **Alert when the self-upgrade is skipped for a long time.** In `self_upgrade_tick`, when a green
   commit is waiting and the tick has been skipped for N ticks / 3 h because `spawning` or a busy
   agent, emit `upgrade.waiting` with the holder and wake the orchestrator; escalate to the human
   after 6 h. Also log the first skip at info.
4. **Log and event every restart refusal** (`restart.refused`, with who asked and what blocked).
   Today the first refusal cannot be dated.
5. **A safe forced restart for the orchestrator**: `bridle daemon restart --force-spawning`
   (orchestrator or human only) that ignores a spawn counter older than N minutes but still waits
   for busy agents, so the orchestrator is not dependent on `launchctl` access.
6. **Fix the CI watch ssh key** (`Permission denied (publickey)`, 23:58:31): it blinds self-upgrade
   and, separately, may relate to push failures (br-2y3m).

### What went well / badly

Well: spawns kept working, no work was lost, and the human could kick the daemon. Badly: 22 h
without an upgrade and no signal; the restart error withheld the one fact needed; the stated
trigger sent the first investigation down a path that does not leak.

### Open

The exact hanging await is unproven. If the flag sticks again, before anyone restarts: record
`bridle status --json` and take a stack sample of the daemon (`sample <pid>`), then restart.
