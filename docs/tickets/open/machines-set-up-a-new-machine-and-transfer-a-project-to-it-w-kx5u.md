---
id: br-kx5u
title: "Machines: set up a new machine and transfer a project to it, with nothing left behind"
kind: epic
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-du7e, br-kt25, br-rjd5, br-v7ug, br-evy6]
---

## The ask

The human, 2026-10-11 ~8:45 PM ET (to advisor product-manager), verbatim:

> ok, we need the work for mail bridge and gateway to work as well. this needs to be in an epic to be able to
>
> 1. set up a new machine
> 2. transfer a project to a machine
>
> I don't have NEW projects to run on remus (wsl2) - I need to transfer a proejct over. i want to transfer bridle and bridle-ui, so that needs to be possible.

And minutes later: "I will test with a smaller proejct firs, but this needs to work; also, i'm concerned about everything that is not moving oer"

Earlier tonight, on the same work: "make bridle safely stop the daemon and push the state and everything. don't make it best effort" and "don't make any big changes" (keep each piece small).

## Outcome

The human sets up a new machine with bridle's commands, then moves a project to it, and the project works there as it did on the old machine: daemon, service, tokens both ways, config on every machine, mail bridge, gateway, inbox and schedules. Nothing is silently left behind; anything that cannot move is listed by the command before it runs.

## Done when (PdM draft)

1. A new machine (remus, WSL2) is set up and paired with commands only: config, `[machines]`, `[projects]`, tokens both ways with homes, no hand edits.
2. A small project, then bridle-ui, then bridle, each moves dalek -> remus with the commands: safe stop with a confirmed push, take over (one-shot), start; `[projects]` and ports updated on every machine; tokens keep working; mail bridge and gateway reach the project at its new machine; inbox, outbox and scheduled messages carried.
3. An inventory of everything a project has on a machine exists, and for each item the move either carries it, recreates it, or names it as left behind (before running, with a confirm).

## Children and order

1. br-3pc2 inventory (research): everything a project has on a machine, and what moves.
2. br-eurm: pair records homes; re-run repairs every broken token (no home, rejected, renamed machine).
3. br-2wnn: sync `[projects]` and `[machines]` across machines.
4. br-53gy: safe stop (confirmed push), one-shot take over (syncs config via 2wnn), start; on every OS.
5. br-kt25: the full move: carry the database (PdM recommends: copy `bridle.db` from the stopped machine), mail bridge and gateway retarget, service install and uninstall, uncommitted worktrees named. Its 2026-10-10 sequencing (after the orchestrator-per-machine epic) is overridden by this ask.
6. Related, to fold in or drop after the inventory: br-rjd5 (set up a project everywhere), br-n63z (peer tokens at project creation).

## Notes

- Moving bridle itself is special: dalek's orchestrator runs in the bridle project (one per machine). The inventory and kt25 must say what happens to it and to dalek's other projects that send to bridle.
- The human's hands-on steps stay in du7e (h7mu, jgdb, r2nr).
