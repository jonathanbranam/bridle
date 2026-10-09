---
id: cy5v
title: bridle serve --detach gives up at 15 s while the daemon is still starting
kind: bug
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-b65b]
closed: 2026-10-09T23:11:06Z
---

## The ask


The human, verbatim (2026-10-03, via the advisor), after `bridle serve --detach` for bridle-ui
printed "error: daemon did not become healthy within 15s" while the daemon was in fact starting:

> I don't mind the warning but it should keep waiting I think for longer. I presume system load
> here can you check that and add a note about thenload

## What happened

Logged in `docs/context/incidents.md` (2026-10-03 01:20): the daemon logged that it was
listening about 0.4 s after the CLI gave up, under heavy load (15-minute load average 17.8 on 16
logical CPUs; the bridle daemon's self-upgrade build had just run, and other `rustc` builds were
running).

`crates/bridle/src/serve.rs` (`--detach`): polls the child's health every 200 ms for a fixed
15 s, then errors without stopping the child, which keeps starting.

## The change

1. **Wait longer:** e.g. 60 s (the fix decides; a config key isn't needed).
2. **Keep the config warnings** in the output; the human doesn't mind them.
3. If it still times out, say the daemon is still starting and was left running (pid, log path,
   how to check: `bridle daemons`), not "did not become healthy".

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
