---
id: 93xm
title: Other projects submit bridle tickets directly, for triage
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [7gk7, phyy]
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
