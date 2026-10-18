---
id: br-r627
title: "[when convenient] Restart dalek-home's daemon and the orchestrator so it starts in dalek-home's repo (br-jn8p)"
kind: chore
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-jn8p landed on main at 2ad8b6c2 (2026-10-18 ~5:10 PM ET): when the orchestrator is relaunched, the command now cds into the home project's repo first. It takes effect only once dalek-home's daemon runs a binary that has the fix, and only for the next relaunch. The human chose B on 2026-10-18: no workaround, wait for this fix.

Steps:
1. Wait for the aide to say the installed bridle binary has 2ad8b6c2. The bridle daemon upgrades itself; the aide checks.
2. `bridle daemon restart --project dalek-home`
3. In the orchestrator's pane (%4, window 1_bridle:1), type `/exit`. The supervisor relaunches it in about 10 s.
4. The aide checks the new session's folder (`lsof -a -p <claude pid> -d cwd`). It should be /Volumes/Data/work/dalek-home-work/dalek-home.

Finish it with `bridle ticket done br-r627`.
