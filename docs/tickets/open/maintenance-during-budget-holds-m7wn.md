---
id: m7wn
title: Maintenance during budget holds
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [r3nh]
---

## The ask

The human, verbatim (2026-09-28), after asking whether a rebuild, `cargo clean` and daemon
restart were needed and whether they could happen during holds, "when work actually
slows down":

> Let's be sure this is documented and followed, prompted and planned ahead - when we are
> hitting a budget hold, then always use that opportunity for general cleanup. yes file as
> a ticket for the future

## What's done by hand today

- The orchestrator role's "Budget holds are the maintenance window" section
  (`workflow/base/roles/orchestrator.md`) lists the steps: verify and push, `cargo clean`
  and `cargo install`, one daemon restart by the human, resume agents, remove finished
  workers, renew large contexts, tidy tickets.
- `scripts/orchestrator-watch.sh` wakes the orchestrator when the governor leaves `normal`.
- The orchestrator keeps a list of what's waiting for the next window in
  `docs/context/orchestrator-state.md`.

## For the future

Bridle doing this itself, when a hold begins and every agent is idle:

- rebuild from verified `main` and restart the daemon (today only the human restarts it);
- resume what was running, and renew agents whose context is large;
- clean stale build output (see f75x) and remove finished workers' worktrees.

The restart is the part that needs a decision: whether the daemon may restart itself, and
how it knows `main` is verified.
