---
id: 3mz4
title: "One orchestrator per machine, not per project: say so in the advisor role and wherever agents send to it"
kind: chore
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [bp2v]
tasks: [br-3mz4]
---

## The ask

The human, 2026-10-09 ~5:30 PM ET, verbatim (via the bridle-ui aide to advisor product-manager):

> There is only one orchestrator per machine; we need to make that clear. the orch for this machine runs in the bridle project.

The aide asked the PdM to make it clear "wherever it is fuzzy".

## Where it is clear already

- `workflow/base/roles/orchestrator.md` ("There is one orchestrator per machine, not per project").
- `workflow/base/roles/aide.md` ("The orchestrator is different: one per machine ... a message to
  `external:orchestrator` on this project's daemon is accepted even when nobody is listening").
- `docs/notes/product-manager-trial.md` (fixed by the PdM, 2026-10-09).

## Where it is fuzzy (found 2026-10-09)

- `workflow/base/roles/advisor.md`, "Deferring to the orchestrator": says `bridle send
  external:orchestrator "From advisor: ..."` with no `--project`. An advisor in another project
  sends to an orchestrator inbox nobody reads. The PdM did exactly that today (o-0048 to
  bridle-ui's `external:orchestrator`). Fix: the same paragraph as aide.md, and the send line with
  `--project <the orchestrator's project>`.
- Agents' own words: the orchestrator signed a note "orchestrator, acting PM for bridle-ui"
  (br-gd43), and agents speak of "the bridle-ui orchestrator". A rule-level sentence would stop it.
- Possibly: `bridle send external:orchestrator` on a project whose daemon has no orchestrator
  waiter could warn ("no orchestrator listens on this project; it runs in <project>"). That is a
  code change; only if cheap (the daemon knows whether a waiter is registered).

Docs-only except the optional warning. Theme `agents-and-cli`.

## An epic, sequenced before project transfer (the human, 2026-10-10 ~10:50 AM ET, to the advisor)

> Great. The goal would be that project takeover can be initiated by either machine, and the goal
> is also that it can be handled by the orchestrator on both machines.
>
> I think, in terms of sequencing for this work, let's make sure we have an epic for one
> orchestrator per machine, and let's sequence that epic before this one. I think that will
> simplify things. We already have problems with the orchestrator role, so in terms of
> sequencing, make sure there's an epic, pick an appropriate theme for one orchestrator per
> machine, and make sure that comes before any epic about this project transferring stuff.

The PdM's epic "One orchestrator per machine" (theme agents-and-cli) holds this ticket's task and
the related open tickets ma8e (one orchestrator across projects or one per project), 7d62
(orchestrator identity and recovery) and pdmd (a leaner orchestrator); kuw2 (the machine daemon)
is related. It comes before the epic "Projects become machine-transparent" (evy6), where takeover
can be started from either machine and handled by the orchestrators on both.
