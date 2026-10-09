---
id: 93xm
title: Other projects submit bridle tickets directly, for triage
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [7gk7, phyy]
closed: 2026-10-09T23:11:01Z
---

## The ask


The human, 2026-10-01 (via the NUC's orchestrator, m-3179), verbatim:

> it's innefficient to go through orch to file tickets. I think other projects should be able to
> file bridle tickets through some means - if they can reach bridle over a port, why not? Those
> tickets should be marked as coming from the external:advisor@nuc or whatever and triages
> appropriately by the project-owning orch and PM, not blindly accepted, but it would save the
> steps of going through the orchestrator for everything with messages.

## Reading

- Any principal with a token on bridle's daemon (visitors such as `external:advisor@nuc` and
  `external:orchestrator@nuc`, and other projects' agents) can submit a ticket directly, through
  an API call or a CLI command such as `bridle ticket submit --project bridle`.
- The submission records its submitter and lands in a proposed state, not as a ticket file.
- Bridle's orchestrator or PM triages each one: accept it as a ticket, merge it into an existing
  ticket, or decline it with a reply to the submitter.
- It fits 7gk7 (bridle manages tickets). It would replace relays like m-3148, m-3169, m-3172
  and this one.

## Built

`bridle ticket submit -k <kind> "<title>" --body-file -` (`POST /v1/tasks/submit`) files an `open` task
(no new state: only the PM plans) whose body's first line is `submitted by <principal>`, with the same as its
first thread note. The product manager, else `external:orchestrator`, gets one inbox message. Dropping it with a
reason sends the reason to the submitter. Visitors (`name@machine`) can no longer plan, claim, drop or edit tasks,
and may comment only on their own submissions. Not built: dedupe, a proposed state.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
