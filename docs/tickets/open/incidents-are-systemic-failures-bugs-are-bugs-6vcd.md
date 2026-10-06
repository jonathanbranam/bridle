---
id: 6vcd
title: Incidents are systemic failures; bugs are bugs
kind: chore
opened: 2026-10-06
repos: [meta-notes]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

# Incidents are systemic failures; bugs are bugs

The human, 2026-10-06 (to the meta-notes orchestrator on the NUC, directly),
after the orchestrator logged a product bug (meta-notes xjaq, `no plan`
written unstruck) as an incident:

> Not an incident, it's a bug.

> I need to refine the incident rules. Bugs are bugs, and incidents are
> incidents. They're different things.

> the incident tracker is intended for systemic types of things that occur
> repeatedly and actually affect the working of the system itself.

> Definitely, when a rule doesn't follow a rule and an agent doesn't do
> what's intended, those are incidents that we want to track, but bugs are
> different. We have some bugs, and we'll learn from bugs and file bugs in
> a different way.

The human agreed with the orchestrator's split ("that makes a lot more
sense"), naming crashes, failed installs and updates, a red `main`, and
possibly a bad merge as incidents.

## The split

- **Incident:** the system itself failed to work: a crash or stall, a
  failed install or update, a red `main`, a bad merge, work stuck between
  roles, an agent or role not following a rule or not doing what's
  intended (e.g. meta-notes cys2 built inside its settle period). Logged in
  `docs/context/incidents.md` so systemic, repeating patterns show up.
- **Bug:** the product is wrong (behaviour, docs, a convention). A bug
  ticket and task (`kind: bug`); no incident entry. A bug is also an
  incident only when how it shipped was itself a process failure (a worker
  writing against its ticket and the review missing it).
- **No separate bug tracker for now:** bug tickets are it (`bridle task
  list -k bug`). The human: "we'll learn from bugs and file bugs in a
  different way"; how is a later question, not this ticket.

## Change

- `workflow/base/roles/orchestrator.md`, "Keep the incident log": log
  incidents as defined above, not "every failure or problem"; a product
  bug the human reports is a bug ticket.
- `docs/context/incidents.md` header (bridle): the definition, replacing
  "it now covers anything that went wrong".
- Other roles that mention the incident log (aide, advisor, manager), if
  they say "every failure or problem".
- Projects' copies of the header (meta-notes `docs/context/incidents.md`):
  the meta-notes orchestrator updates its own once this lands.

Verify: the role text and header state the split; `bridle prime
orchestrator` shows it.
