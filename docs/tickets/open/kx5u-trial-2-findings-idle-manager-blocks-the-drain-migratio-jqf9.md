---
id: br-jqf9
title: "kx5u trial 2 findings: idle manager blocks the drain, migration leaves checkout dirty, noisy and contradictory move output"
kind: fix
created: 2026-10-18
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-eavq]
---

Found in kx5u trial 2, 2026-10-18 ~7:30-7:45 PM ET: the human moved data-contracts remus -> dalek by
hand with docs/context/move-a-project.md. The move worked end to end (released mark, take-over, start,
every token check accepted, doctor clean, remus units disabled). The preflight bug is eavq. These
are the rest.

## The ask

1. **An idle agent blocks the drain forever.** data-contracts' manager was idle (7 turns, nothing
   claimed). `bridle project drain` listed "agent(s) running: manager" and waited until the human ran
   `bridle agent stop manager` by hand. `project stop` then says "stop its daemon and 1 running
   agent(s)" and refuses on that same agent. Fix: the drain stops each agent once it is idle with
   no claimed or built work (after its announce turn), or `stop` stops idle agents itself. Either
   way, the runbook says nothing about a manual stop.
2. **A bridle migration left the main checkout dirty, which blocks every move.** Migration
   0007-gitignore-agent-site (bridle 0.7.6, 10-17) changed `.gitignore` and `.bridle/migrations.*` on
   remus and never committed them; the drain waited on them until the human committed by hand. Fix:
   migrations commit what they change (or the drain names them as "uncommitted bridle migration: run
   <command>" with the fix). Check other projects for the same leftovers.
3. **take-over prints the whole `projects sync` table** (every machine, `[mail]` and `[projects]` row)
   when one entry changed, and the `[mail] allow` row runs its columns together. Print only the
   differences.
4. **start's mail bridge output contradicts itself:** "wrote .../dev.bridle.mail.data-contracts.plist
   / nothing was started; to load it: launchctl bootstrap ..." and then "mail bridge unit
   dev.bridle.mail.data-contracts enabled and started" (it was started: launchctl lists it). Drop the
   install helper's "nothing was started" hint when start goes on to start it.
5. **Runbook step 5.5 names `bridle mail status`, which doesn't exist** ("unrecognized subcommand").
   Name the command that does (or `launchctl list | grep bridle.mail` / `systemctl --user is-active`).

## Proposal

Five small fixes, no design choice except 1: recommend the drain stops idle agents itself, since it
already announced the move to them and is the step whose job is "nothing left running". Update
docs/context/move-a-project.md for 1, 2 and 5.
