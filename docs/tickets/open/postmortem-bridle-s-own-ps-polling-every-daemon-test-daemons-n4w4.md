---
id: n4w4
title: "Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dalek's load to 76-144 and the load hold blocked spawns for ~26 h; br-3p3h fixed only the idle case"
kind: incident
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [xypj, 58c9, z7y5, npj2, b7cz, y55w]
tasks: [br-n4w4]
---

## The ask

The human, on the overnight load hold and its fix (br-3p3h), relayed by the orchestrator
(2026-10-08): "What? Why did this happen?" They asked for a postmortem.

Written from the three dalek daemons' message tables (read-only), `bridle task show br-3p3h` and
its thread, `git log` / `git show`, and the code at a9183146. Times are US Eastern (UTC-4).

## Summary

Every bridle daemon forks `ps -axo pid=,ppid=,pgid=,lstart=` on a fixed timer to track agents'
processes (containment). Until a1bde105 it did so even with no agent live. Test daemons run the
same loop, and the test harness sets the timer to **200 ms**, not 2 s. Each test that starts a
daemon (229 daemon tests in 43 files) therefore ran `ps` almost back to back for its whole life.
A `just check` runs up to 8 tests at once (nextest `test-threads = 8`), and several worktrees ran
checks at the same time. On this Intel Mac each `ps` walks the whole process table and is a fresh
exec, so the `ps` storm, plus fake-claude's pyenv shim (bash, bash, python per fake agent), plus
real builds, drove load to 76-120 (up to 144 later). The load hold added the day before
(br-58c9) then refused every new spawn on all three projects while load was above 2.5 per core.

The `ps` loop is from v1 (2026-09-27). It only became an outage when the load hold landed on
2026-10-07. The hold's own note named `(ps)` as the top consumer from 06:34 ET on 10-07, but it
was read as build load for ~26 hours. br-3p3h (a1bde105) stops the snapshot when no agent is
live. **It did not close the incident:** holds naming `(ps)` went on after it (29 notes from the
bridle daemon alone between 10:34 ET and 20:36 ET on 10-08, peak 144.5 at 19:22 ET), because
test daemons with fake agents still poll at 200 ms. Fix 3 (no fork) is br-9z2n, in progress.

## Impact

- Spawns held across every project on dalek, on and off, from 10-07 02:36 ET to now. The three
  daemons sent 139 "Machine load is high" notes to the orchestrator before the fix (51 bridle,
  44 bridle-ui, 44 track-web, from 10-07 02:00 ET) and 82 after it. `(ps)` was the named top
  consumer in 17-20 notes per daemon.
- bridle-ui stalled overnight (orchestrator to aide, 10-08 01:40 ET, m-6958: "bridle-ui stalled
  on the machine-load hold (load 76-95 overnight; spawns held)").
- Held spawns: br-2672 (20:06 ET, m-6678), br-ezpj (20:52 ET, m-6770), the critical br-grdg
  (21:05 ET, m-6827; the orchestrator said wait both times), ui-bpsd (19:18 ET on 10-08, m-0883),
  br-9z2n itself (13:55 ET on 10-08, m-7165).
- Load made timing tests flake, so checks failed and were rerun, adding load: br-4yc8's land and
  wsl2guide (m-6786, m-6799), waitfallback at load 137 (m-6868), br-at2j landed with a waived
  check (m-6907), br-eyu3 and br-7172 couldn't get a green check on 10-08 (m-7125, m-7160).
  Follow-up br-5p3z.
- The human paused br-npj2's build measurements on 10-07 08:23 ET until the evening reboot
  (m-6532). That pause was mainly about build load; how much `ps` added is not measured.
- How long spawns were actually held in total is **unknown**: the daemon records the start of a
  hold (a message) but not its end.

## Timeline (US Eastern)

- **2026-09-27 12:26** v1 (38532ef2): `containment::snapshot` forks `ps -axo` every
  `tracker_interval` (2 s, lib.rs), with or without agents; the test harness's
  `default_overrides()` sets `tracker_interval: 200 ms` (crates/bridle-daemon/tests/support/mod.rs:218).
- 2026-09-29 to 10-06: repeated high-load reports (b7cz, 58c9 on 10-04 at load 70-80, z7y5 on
  10-06). All attributed to builds, target-dir copies, syspolicyd/XProtect and Spotlight. `ps` is
  never named. Whether it contributed then is **unverified**.
- **10-07 02:17** br-58c9 lands (28634360): the load watch refuses new spawns above 2.5 per core
  and notes the orchestrator once per crossing with the top three consumers. The bridle daemon
  runs it from 02:33.
- **02:36** first hold note (43.2). **06:34** first note naming `(ps)` as top consumer (143%, load 71).
- **08:23** the human holds br-npj2's builds until the evening (load 92-106).
- **18:28** dalek rebooted; all three daemons restart (pids 640, 648, 662).
- **19:09** notes resume from all three daemons at once (95.2, 68.7, 53.9). **19:14** the
  orchestrator writes "The daemon's load note blamed python/ps, but the rustc fan-out plus XProtect
  is the real load" (m-6584).
- **20:06-21:05** spawns refused: br-2672, br-ezpj, br-grdg. The orchestrator says wait.
- **21:02** upgrade_test fails repeatedly under load; becomes br-5p3z.
- **21:45** the orchestrator sees `(ps)`, `(bash)`, `(python3.11)` on top during `just check` and
  attributes it to the pyenv shim around fake-claude (m-6872, to br-npj2).
- **22:13** 120.8 (bridle note). Notes continue to 22:51.
- **10-08 00:53 and 01:04** 76.6 and 83.6 (syspolicyd 503%). **01:08** the orchestrator's
  6 AM list: "overnight load spikes 76-95 ... find the cause" (m-6956). **01:40** it reports
  bridle-ui stalled (m-6958).
- **09:04** 95.1, `(ps)` 186%. ~17 test binaries from one worktree plus the 3 daemons, each with
  its own `ps` in flight (br-3p3h body). The orchestrator files br-3p3h, urgent.
- **10:22** worker psfork done; **10:33** a hold note names `(ps)` during the fix's own check;
  **10:34** a1bde105 integrated. Load not measured before or after; fix 2 skipped.
- **10:57** the bridle daemon builds a1bde105; it restarts into it at **12:45**.
- **13:49** holds still name `(ps)` at 107-160% (load 4.4/core) during governor_test and land_test
  runs. The orchestrator asks for fix 3; pm-1 files br-9z2n.
- **13:55** manager-2: br-7172 and br-9z2n are stuck behind load 35-55, "every just check flake[s]".
- **19:22** 144.5, the highest note in the record. Notes continue past 20:36.
- **20:54** br-9z2n's worker (wt/nofork) is running its tests.

## Root cause

1. **The tracker forks `ps` on a timer, independent of need.** `spawn_loop(...,
   overrides.tracker_interval, ...)` (lib.rs:775) calls `AgentManager::tick_tracker`
   (supervisor.rs:601), which called `containment::snapshot` (containment.rs:26, `ps -axo
   pid=,ppid=,pgid=,lstart=`) before checking for live agents. The design (agents.md,
   Containment) needs a whole-table snapshot to follow agents' descendants, but nothing needs one
   when no agent is live.
2. **Test daemons ran it 10x faster.** The harness's `default_overrides()` (support/mod.rs:218),
   plus context_governor_test, restart_test and tasks_test, set `tracker_interval` to 200 ms, so
   stop and containment tests are quick. Every test that starts a daemon inherited it. Ticks don't
   overlap within one daemon (spawn_loop sleeps then ticks), but `ps` took 1-2 s under load (per
   br-3p3h), so each test daemon kept about one `ps` running at all times.
3. **Many test daemons at once.** nextest runs 8 tests at a time per check (`.config/nextest.toml`,
   lowered from one per core in br-7fr6 for this same kind of load), and workers and the land
   check run checks in parallel worktrees. Fake agents add a bash-bash-python chain each through
   the pyenv shim (m-6872).
4. **Feedback.** More load makes each `ps` slower, so more of them overlap. More load makes timing
   tests fail, so checks are rerun. Each new test binary is also a fresh exec that syspolicyd and
   XProtect look at (z7y5, cs7x).
5. **The hold turned slowness into an outage.** Before 10-07 the same load only made things slow.
   The hold (load.rs, `refuse_if_load_held`, supervisor.rs:505) refuses every new spawn of every
   project on the machine while load per core is above 2.5, and the three daemons hold together.

## Why it wasn't caught

- It shipped with v1 as part of containment. No review or test looked at the cost of an idle
  daemon or of the test harness's 200 ms setting.
- No test measures background resource use (forks per tick, per idle daemon). The fix added one
  (`tracker_takes_no_snapshot_without_agents`), for the idle case only.
- Earlier load incidents (b7cz, 58c9, z7y5) had real, visible causes (rustc, cp, syspolicyd,
  Spotlight). Short-lived `ps` processes don't show in a casual `top`, so the base load from bridle
  itself was never isolated.
- br-58c9's worker saw 4.9 per core on this host while testing the load watch (m-6433) and the
  default 2.5 shipped anyway, with no check of what the idle load of the machine was.

## Why it went on for ~26 hours

- Only the orchestrator gets the hold notes. The human sees load only through `bridle status` or
  the orchestrator.
- The notes did name the cause. `(ps)` led them from 06:34 ET on 10-07. The orchestrator read the
  notes as build load and said so (m-6584, 19:14 ET), then blamed the pyenv shim (m-6872).
- The orchestrator role says, for a load note: "Add no work ..., wait, and tell the human only if
  it lasts or names a cause you can act on" (workflow/base/roles/orchestrator.md:94-96). "Wait" is
  what it did, all night. Nothing says "a bridle process is the top consumer" or "spawns have been
  held for N hours".
- Three daemons send the same note for the same machine, and holds flap around the threshold:
  28-29 notes per daemon between 19:00 ET and 01:30 ET. That is noise, and it trains readers to skim.
- The note says "the daemon resumes them itself". It doesn't: a held spawn is refused with a
  conflict, not queued (supervisor.rs:505-513); it happens only when a manager retries. So the
  hold looked self-healing.
- Diagnosis was put off to a morning list (01:08 ET) and done at 09:04 ET.

## The fix (br-3p3h, a1bde105, 10:34 ET on 10-08)

`tick_tracker` now returns before the snapshot when no runtime is live (`tick_tracker_with`,
supervisor.rs:598-624), with a unit test that counts zero snapshot calls with no agents.
agents.md (Containment) and CHANGELOG updated.

What it does not cover:

- Test daemons with live fake agents still snapshot every 200 ms. Most daemon test files spawn
  agents (32 of the 43 that start a daemon). Holds naming `(ps)` at up to 160% continued (13:49,
  14:14, 18:45 and 19:43 ET). Fix 2 (longer test interval) was skipped; fix 3 (no fork) is
  br-9z2n.
- The before/after load measurement in the acceptance was not done, and manager-2 accepted it as
  optional (m-7085). Activity after the fix was lower, so the note counts above don't prove much
  either way, but they show no clear drop.
- The bridle-ui and track-web daemons (pids 648, 662) have run since 18:28 ET on 10-07 with no
  restart or upgrade recorded since, so they likely still fork `ps` every 2 s while idle
  (**unverified**: their binary's commit wasn't checked; minor next to the test daemons).
- The pyenv shim around fake-claude, the hold's design and its notes are unchanged.

## Recommendations

1. **Finish br-9z2n** (read the process table without forking), and until it lands, give the test
   harness a long `tracker_interval` by default, with 200 ms only in the tests that need it (stop,
   containment, governor). Record load before and after on dalek during `just check`.
2. **Add a resource budget test**: start a test daemon with no agents and with one fake agent, and
   count the child processes it forks over a few seconds through an injected command runner. Fail
   if an idle daemon forks anything on a timer.
3. **Audit every periodic loop** in lib.rs (stall, tracker, governor, CI, flush, ports, disk, load,
   doc watch) for what it forks or reads per tick and whether it runs with nothing to do; list them
   in daemon.md with their cost.
4. **Make the hold notes say what matters**: one note per machine, not per daemon (xypj); flag
   when a bridle-owned process (`ps`, a test binary, `bridle`, fake-claude) is a top consumer;
   fix the false "the daemon resumes them itself" text (or queue held spawns so it's true); and
   emit `load.hold.started` / `load.hold.ended` events so total hold time is visible.
5. **Escalate a long hold**: if spawns have been held for more than 60 minutes in a 2-hour window,
   or a critical task's spawn is refused, tell the orchestrator to investigate and the human in
   the morning list. Change the orchestrator role text from "wait" to "wait, and if the note names
   a bridle process or the hold lasts, find the cause now".
6. **Cap concurrent full test runs per machine** (a machine-wide lock or semaphore for `just
   check`, or fewer nextest threads when another check is running). Today N worktrees multiply
   the test daemons by N.
7. **Remove the pyenv shim from fake-claude spawns**: resolve the interpreter once and call it by
   absolute path in the harness (m-6872).
8. **Restart the bridle-ui and track-web daemons** onto a build with a1bde105, and check whether
   project daemons other than bridle's pick up upgrades at all.
9. **Log this in docs/context/incidents.md** (no entry yet), with a pointer here.
