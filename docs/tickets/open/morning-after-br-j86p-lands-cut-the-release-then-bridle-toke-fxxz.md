---
id: br-fxxz
title: "[morning] After br-j86p lands: cut the release, then bridle token pair on each machine"
kind: chore
created: 2026-10-17
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-16 ~11 PM ET (via aide): "Try to get these tickets through! I need to sleep".

Once br-j86p (same-machine orchestrator keeps direct tokens) is on main:
1. The release commit, tag and push (the orchestrator prepares it; its commit is refused, so it gives the exact commands).
2. After vesta and remus upgrade: in your own terminal on each machine, `bridle token pair --machines dalek,vesta,remus`. That swaps the orchestrators' visitor tokens back to direct ones, with full rights on their own machine.
3. Then the dalek orchestrator can move to dalek-home (paused 10-16): ask the aide.
