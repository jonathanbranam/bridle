---
id: q7rx
title: Bridle upgrades and restarts itself; the human no longer restarts the daemon
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [maintenance-during-budget-holds-m7wn, reload-config-without-a-restart-9t54, do-workers-resume-after-a-daemon-restart-2fkk]
---

## The ask

The human, 2026-09-30 (to the orchestrator), after asking "When are we going to get away from
needing me to restart bridle?":

> I'm happy with bridle restarting itself; IDK if that decision needs to be approved by your or
> is mechanical or not. I think you should be able to request a restart.

This settles the open decision in m7wn ("whether the daemon may restart itself").

## Today

Every rebuild or config change waits for the human to run `bridle stop-daemon` and `bridle
serve`. Workers come back `lost` and the orchestrator resumes each one by hand
([[docs/design/agent-host/daemon#Restart and recovery|restart and recovery]]; 2fkk).

## Shape

1. **Upgrade = build verified `main`.** Build only a commit on `main` whose CI run on GitHub
   Actions is green (`gh run list` or the API; the daemon already watches CI for the failed-run
   wake). Build it with `cargo install --path crates/bridle` from the clone, in the background,
   with the machine's normal priority. A failed build leaves the running daemon untouched and
   reports why.
2. **Restart in place.** Save state (flush the state branch, and push it: we2r), stop agents
   through the shutdown path (`daemon_shutdown`), then `exec` the new binary
   (`std::os::unix::process::CommandExt::exec` on `current_exe()`: safe Rust, same PID, same
   terminal, so the human's `bridle serve` window keeps running and Ctrl-C still works).
   The new process rebinds the same port; clients retry through the gap.
3. **Resume everyone.** After the restart, resume every agent that was running before it:
   workers too, not only `resume_on_restart` roles. Each gets a message that the daemon
   restarted for an upgrade and to carry on. This settles 2fkk for this path.
4. **Who triggers it.**
   - The orchestrator may request it: `bridle restart` (optionally `--upgrade`), an API
     endpoint for `external:orchestrator` and the human only.
   - Mechanically, the daemon may also do it itself when `main` has moved past the running
     binary and it's a quiet point (every agent idle, or a budget hold with workers wound
     down). Behind a config switch; on for bridle's own project.
   - Never mid-turn: wait for a quiet point, with a timeout that gives up and reports rather
     than cutting work off.
5. **Tell, don't ask.** A wake to the orchestrator before and after (commit, what resumed).
   Nothing to the human's inbox unless it failed.

Config reload without a restart (9t54) is the smaller follow-up; with self-restart in place, a
config change can simply request a restart.

## Out of scope

- Upgrading other projects' daemons (one daemon per project today; each would restart itself by
  the same mechanism, but the trigger is per project).
- Rolling back a bad binary: done (br-4524; daemon.md, "Rollback"). A hard crash before serving
  is only recovered by the next manual start, which rolls back.

## Seen 2026-09-30: the upgrade starves while both worker slots stay busy

Twice in a row (6e9fbf4 at ~12:25Z, 717a444 at ~13:50Z) the build succeeded but "no quiet point
within 600s; still busy: <two workers>". The manager refills a slot as soon as a worker lands, so
with a full queue there is never a moment when every agent is idle, and the fixes that most needed
to be running (the rollback, 9d16b03; d3wq, e682bd7) weren't. Worked around by hand: the
orchestrator told the manager to spawn nothing new until the restart. Proposed: once a green
build is waiting, the daemon refuses new worker spawns (as a budget hold does, with a clear
reason) until it has restarted, so running workers drain to a quiet point; the timeout then
covers only a worker that never finishes.

Done (br-e990): once a green build waits for a quiet point, worker spawns are refused (409, named in
`bridle status`) until the restart; a timeout give-up lifts the refusal.
