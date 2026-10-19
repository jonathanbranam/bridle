---
id: br-cxjs
title: "A project move tells neither machine's orchestrator: notices go to the moved project's own orchestrator role, unread"
kind: fix
created: 2026-10-19
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-jqf9, br-7ytg]
---

Found after kx5u trial 2 (data-contracts remus -> dalek, 2026-10-18 ~7:37-7:40 PM ET), reported by
remus-home's aide (m-17244, 10-19; the human said "Yes" to raising it with bridle):

> Neither remus-home's aide nor remus's orchestrator was told. The only notice we found:
> data-contracts' daemon (now on dalek) sent two system messages to its orchestrator role at 7:39 and
> 7:40 PM ET (there m-0002, m-0005). They were still unread the next day, and remus's orchestrator,
> holding only a visitor token there, could not read them.

The human's model, verbatim: "An orch is for machine. So daleks takes over".

Gaps:
- (a) OLD's orchestrator and aide (in `<OLD>-home`) get no word that a project left the machine.
- (b) NEW's notice goes to the moved project's own `orchestrator` role, which no one is watching
  (the orchestrator runs in `<host>-home`, ticket 7ytg), so it sits unread with no follow-up.
  `project drain` messages "orchestrator" the same way.

## The ask

1. `project stop` (on OLD) sends one message to `external:orchestrator` and `external:aide` in
   `<OLD>-home`: the project left, to which machine, when.
2. `project start` after a take-over (on NEW) sends one message to `external:orchestrator` in
   `<NEW>-home`: the project arrived, from where, and what moved-in.json says is pending. NEW's
   orchestrator now watches it.
3. `project drain`'s messages to "orchestrator" go to `<OLD>-home`'s orchestrator too (not only to the
   project's own role).
4. Where `<host>-home` doesn't exist yet, fall back to today's target and say so in the command's
   output. Best effort: a failed notice never fails the move.
5. Update docs/context/move-a-project.md (who is told at each step) and the design doc the move
   commands live in.

## Proposal

Send with the same cross-project path as `bridle send --project <host>-home`, which already queues and
retries to another machine. No new principal or setting.
