---
id: br-e7s5
title: "[when convenient] Restart dalek-home's daemon so it runs the br-uysm relaunch fix"
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

br-uysm (orchestrator relaunch re-ran the pane's old --project bridle command) landed on main at 3b7bb078, 2026-10-18 ~1 AM ET. The relaunch is done by dalek-home's daemon, which still runs its 01:25Z build (dalek has self_upgrade off), so the fix is not live there yet.

In your own terminal: `bridle daemon restart --project dalek-home`. After that, the orchestrator's next relaunch (its next handover, or /exit) comes back as `bridle session orchestrator --project dalek-home`; the aide checks the pane then.
