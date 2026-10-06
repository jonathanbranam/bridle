---
id: zcqv
title: "dalek slept in a bag 8:42 AM-1:38 PM ET on 10-06: bridle froze, but the workforce had already been idle since 11:21 PM"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [4r3k, stx8]
tasks: []
---

## The ask

The human, 2026-10-06 ~1:40 PM ET (verbatim): "I just realized my laptop dalek has been closed in my backpack all day from ~8:45 - 1:35pm. I'm curious if things kept running while it was shut. Internet was here; i'm at my office." Then: "please investigate now and write up a report in a ticket, but don't share any details with me until 5pm."

## Findings (orchestrator, investigated 2026-10-06 13:40-13:55 ET)

**1. dalek slept, so bridle stopped.** `pmset -g log`:
- 08:41:55 ET: "Entering Sleep state due to 'Clamshell Sleep' ... Using Batt (Charge:97%)".
- 08:43 to 13:36 ET: about 40 DarkWakes (maintenance, a few seconds to ~25 s each, roughly every 15 min, later more often from Standby).
- 13:38:21 ET: full "Wake from Standby".
- `caffeinate -si` (pid 14082, running since Sun 10-04 09:48) was active the whole time. `-s` prevents system sleep only on AC power and `-i` only idle sleep, so neither stops **clamshell sleep on battery**. caffeinate does not keep a closed laptop on battery awake.

**2. The daemon was frozen, not stopped.** Bridle's event log between 12:45Z and 17:38Z (8:45 AM-1:38 PM ET) has 2 events: a `disk.checked` at 15:17Z and a `ci.completed` at 16:11Z, both during DarkWakes. Gaps of 53 and 87 minutes. No process died: the daemons, the gateway (pid 68010, started 10-05 19:40), the managers and the sessions (aide, advisor, advisor/fields) are all still there and resumed at wake. Nothing needs a restart because of the sleep.

**3. Nothing would have run anyway: the workforce had been idle since 11:21 PM ET.** The sleep cost no work, because there was none in progress:
- bridle: after manager-2 landed br-dxcw and stopped a redundant postmortem worker at 03:21Z, no agent did anything until the human woke the system. Startable planned tasks sat unstarted all night and morning (br-gztq, br-bdrc, br-58c9). Only one worker slot was taken (designer-role, idle, awaiting the human's review of br-ukpm).
- The orchestrator was down from 03:19Z (incident br-4zfa) and nothing else nudged the managers. A manager that's idle with free slots and startable tasks doesn't start them on its own; it waits for a message or wake. This is the "work doesn't ship" pattern the human asked about (stx8, 2026-10-05).
- bridle-ui and track-web managers and fm-links are idle too.

**4. Pushes happened elsewhere while dalek slept.** CI ran on origin for cb6fce13 (16:11Z) and a5ac8bc6 (17:38Z), so another machine (the NUC) pushed to bridle's main during the sleep. Local main is now 24 ahead of a stale origin/main (no fetch since ~12:09Z). That's the same divergence as incident br-2y3m.

## What to decide

- **Keep dalek awake when closed?** Not on battery: macOS forces clamshell sleep. Options: run bridle's always-on work on the NUC (4r3k, kuw2), or plug dalek in with an external display/keyboard (clamshell mode on AC). caffeinate alone won't do it.
- **Say when the host slept.** The daemon could log a `host.slept`/`host.woke` event (a wall-clock jump against monotonic time) and tell the human: "dalek slept 8:42 AM-1:38 PM; nothing ran."
- **Managers start startable work without a nudge.** An idle manager with free slots and startable tasks should spawn on its own (or the daemon wakes it), not wait for the orchestrator. Tie to stx8 and the "why doesn't work ship" question.
- **One machine pushes main, or both reconcile first** (br-2y3m).

Related: br-4zfa (orchestrator down overnight), br-2y3m (diverged main), 4r3k (the NUC recovers on boot), stx8.
