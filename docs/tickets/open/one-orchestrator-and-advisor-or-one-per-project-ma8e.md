---
id: ma8e
title: One orchestrator and advisor across projects, or one per project?
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [hj4g, qun8]
---

## The question

Open for thought and discussion; no decision either way yet.

The human, verbatim (2026-09-28), on hearing the orchestrator is one session across all
projects: "oh interesting. a single orchestrator for all projects? I'm not sure I like that
design, but I do see some benefits. Alright. Is there one advisor for all proejcts as
well?"

And, on filing this: "There are pros/cons to this, please file as an open question for
thought and discussion, not a decision either way at this point."

## Where it stands

- **Orchestrator:** one, by the human's words in
  [[where-the-single-orchestrator-lives-hj4g|hj4g]] ("there should only be one
  orchestrator"), said while choosing between the laptop and the NUC, so perhaps not about
  projects at all. `scripts/claude-orchestrator` starts it in bridle's repo; it reaches
  other daemons with `--project <name>`, and needs a token on each to write.
- **Advisor:** one, by accident rather than design. `scripts/claude-advisor` starts it in
  bridle's repo with a token for bridle's daemon only. It can read any project's daemon
  (read-only requests need no token) but can't file tasks or send there; a project's
  tickets live in that project's repo.

## Pros and cons, as the advisor put them in the conversation

- **One across projects:** one conversation for the human; one view to weigh budget and
  priorities between projects (and keep bridle from mostly working on itself, kpgy).
  Against: its context carries every project's state; a token per daemon.
- **One per project:** each stays focused on one repo and daemon. Against: several
  sessions to watch, and nothing weighs the projects against each other.
- **A middle path:** one orchestrator for priorities and budget, with each project's
  manager and product manager running the day to day (roughly today's shape).

## Working rule (the human, 2026-09-30)

> I didn't think we made a decision on how many orchestrators exist; I am going with the "one
> orchestrator per box" rule and planning to stick with that until we have a design consensus about
> more than one.

One orchestrator per machine, across the projects that machine owns, until there's a design
consensus otherwise. Not a final answer to this question.
