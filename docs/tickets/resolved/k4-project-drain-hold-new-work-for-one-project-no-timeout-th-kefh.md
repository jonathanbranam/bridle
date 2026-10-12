---
id: br-kefh
title: "K4: project drain: hold new work for one project, no timeout, the orchestrator watching"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-mzy4]
related: [br-kx5u, br-kt25]
parent: br-kx5u
resolved: 2026-10-12T17:55:36Z
---

Child of epic br-kx5u (MVP), from the design in br-kt25 ("## Design", child K4), as settled in the human's grilling session 2026-10-12 (answers verbatim on br-kt25's thread).

The human (Q2, ~10:15 AM ET), verbatim: "1. Be able to have the system stop sending new work for that project specifically. Other projects can continue working as normal. That's a slight difference from a planned upgrade or planned downtime. It's only one project that gets a hold on new work. 2. Let the workers finish. 3. Be sure the manager has fully landed, running all the checks, and merged fully to main. 4. Be allowed to do the transfer."

And (Q5, ~10:17 AM ET): "For Q5, the hold does not expire for the MVP. I'm totally fine with that, but I want the orchestrator to be working with me on this, checking up on it, and being active in it."

## The ask

`bridle project drain <P>` (and `--undo`), run before `project stop`:

1. **Announce**: message every principal of P (PdM, orchestrator, aides, advisors, managers, workers): P is moving to NEW; finish your current work; interactive roles write a handover note.
2. **Hold new work for P only**: no new claims or spawns in P's daemon (each project has its own daemon, so other projects carry on). Planned tickets stay planned; the hold is daemon state, undone by `--undo` or on NEW after start.
3. **Wait with no timeout** until K3's stable-project check passes: workers finish, the manager lands or drops everything.
4. **Report**: print progress; the orchestrator is woken at the start and on each change so it can work the drain with the human, and anything that looks stuck (an agent that won't finish, a ticket that won't land) is named, to the orchestrator and to the human through aide.

Reuse the restart/upgrade drain's pieces where they fit (refused spawns and claims), but do not restart the daemon. Known drain bugs (br-ef3u, br-8swr) are not fixed here; the orchestrator watches for them by hand.

## Acceptance

With fake agents: the announce reaches every principal once; no new claim or spawn is accepted while held; the drain returns when the check passes; `--undo` lifts the hold; another project's daemon is unaffected; `just check` passes.
