---
id: 58c9
title: The orchestrator watches the machine's load and reacts to it
kind: feature
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [xypj, ma8e, b7cz]
tasks: [br-58c9]
closed: 2026-10-09T23:11:03Z
---

## The ask


The human, verbatim (2026-10-04 5:42 PM ET, to the bridle-ui aide; dictated):

> I restarted the gateway, but the load is taking a really long time. I'm not sure if Dalek may be
> under a lot of load right now. We should probably, we should really have the orchestrator
> watching the load on the server and reacting to it potentially. Usually it's compiling for idle
> that causes this.

## Context

- On dalek (16 cores) at 5:42 PM ET: load averages 10.29 / 19.63 / 19.19. Top CPU:
  `syspolicyd` 85%, `XprotectService` 83% (macOS scanning new binaries), a `cp -cR
  /Volumes/Data/work/bridle/integration/target /Volumes/Data/work/bridle/wt/gw-detach/target`
  running 6 min (child of the bridle daemon), `cargo test -p bridle` in `wt/gateway-detach`,
  `cargo nextest run --workspace` in `bridle/integration`, several `rustc`.
- An earlier statement of the idea, unbuilt: "Load can't be scheduled in advance; the machine's
  one orchestrator coordinates it across projects at run time, winding down agents by the
  projects' relative priority when the machine is bogged down"
  ([[how-project-daemons-share-one-budget-xypj|xypj]], "Machine load", quoting
  [[one-orchestrator-and-advisor-or-one-per-project-ma8e|ma8e]]).

## Observed 2026-10-04 ~5:49 PM ET (orchestrator)

Load 80 on 16 cores, from legitimate work: one worker's cargo build (rustc at ~700% CPU), a
`git worktree remove --force` of a finished worktree with its `target/`, and Spotlight
(`mds_stores`, `fseventsd`, five `mdworker_shared`) indexing the new worktrees. Earlier the same
hour: a duplicate spawn's `cp -cR` of `integration/target`, with syspolicyd and XprotectService
at ~85% each. Excluding `/Volumes/Data/work/*/wt` and `integration/target` from Spotlight
(System Settings > Spotlight > Privacy, the human's call) may take a share of this off.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
