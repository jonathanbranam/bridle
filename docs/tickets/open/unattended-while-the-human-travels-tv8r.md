---
id: tv8r
title: Unattended while the human travels (Thu 2026-10-01 to Fri 2026-10-02)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [bridle-restarts-itself-q7rx, agents-and-nightly-auto-reboots-t39j, a-clean-handover-of-a-project-between-machines-24mj]
---

## The ask

The human, verbatim (2026-09-30, to the orchestrator):

> I will be traveling out of town Thursday afternoon until Friday late. I don't plan to bring my
> laptop, so this and the nuc will be running at home during my trip. If there are any concerns on
> reliability we should address them in advance and maybe practice them

## What can stop the work, and what brings it back (checked 2026-09-30 ~11:15 UTC)

Nobody is at either keyboard, and the human reaches bridle only through Remote Control on the phone.
Anything below that ends in "the human restarts it" means the work stops until Friday night.

1. **A daemon dies.** All three laptop daemons (bridle, meta-notes, track-web) run as `bridle serve`
   in tmux shells; nothing restarts them. `bridle launchd install` and
   `docs/context/launchd-restart-plan.md` exist but aren't in use. With launchd `KeepAlive`, a
   crash would come back on its own, and managers resume (`resume_on_restart`).
   Recommend: move at least bridle's daemon to launchd before the trip, track-web first as the
   rehearsal (the plan's own order). Human-only steps (they load the plist).
2. **A bad self-upgrade.** `self_upgrade = true`: the daemon builds green `main` and restarts
   itself in place. If the new binary fails to start, there is a rollback (br-4524) for a failed pre-flight, start-up error or
   failed exec, but not for a hard crash before serving (the daemon is dead). Recommend: `self_upgrade = false` for the trip (one planned restart
   before leaving), unless item 1 is done and verified to recover.
3. **The laptop sleeps.** `pmset` shows `sleep 1` (one minute); sleep is prevented only by
   Claude Code's own short `caffeinate -i -t 300` calls while a session works. If every session is
   idle, the machine may sleep, and the wake loop, Remote Control and CI polling all stop.
   Human to-do br-1b47: `sudo pmset -c sleep 0` for the trip.
4. **A reboot.** Automatic macOS installs are already off (the human, 2026-09-30; to-do br-15af
   dropped). FileVault is on, so any reboot (a power cut, a crash) stops at the unlock screen and
   nothing comes back until the human types the password: no remote recovery, so no point in
   `pmset autorestart`. The NUC reboots itself at 04:00 when an update needs it
   (unattended-upgrades, `docs/context/nuc-host.md`; t39j): human to-do br-276e turns that off.
   The NUC's Remote Control unit is inactive: br-769d; test both from the phone: br-be96.
5. **The orchestrator session dies.** The daemon relaunches it in its tmux pane with backoff (30 s,
   2 m, 10 m), then gives up (orchestrator-supervision.md section 4). The launch path changed on
   2026-09-30 (mrhe 1, `bridle session orchestrator`) and a relaunch hasn't been watched since.
   Remote Control must come back with it, or the human loses their only way in.
6. **Budget.** The governor holds, winds down and resumes by itself. A usage reading older than
   `max_staleness` (600 s) also holds (seen 2026-09-30 11:42Z, cleared at the next reading): safe.
7. **Disk.** 405 GiB free on /Volumes/Data (worktrees 16 GiB); nothing to do.

## Drills (before Thursday noon ET)

- **Orchestrator relaunch:** the human kills the orchestrator session. Pass: bridle relaunches it
  and it shows up in Remote Control on the phone under its name, with the wake loop running.
- **Daemon crash under launchd** (after item 1): `kill -9` the track-web daemon. Pass: launchd
  restarts it, `bridle daemons` lists it, its manager resumes.
- No reboot drill: with FileVault, a reboot always needs the human at the keyboard.

## State

Open. The orchestrator is working through it with the human on 2026-09-30.
