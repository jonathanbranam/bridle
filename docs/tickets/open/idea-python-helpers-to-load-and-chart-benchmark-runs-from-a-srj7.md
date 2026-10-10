---
id: srj7
title: "Idea: Python helpers to load and chart benchmark runs from a Jupyter notebook"
kind: feature
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [v6kr]
tasks: []
---

**Tentative idea, not committed and not scheduled.** Bring it up in the PdM's reviews of themes
and epics; don't plan or build it until the human says so.

## The ask

The human, 2026-10-10 ~10:15 AM ET, verbatim (to the aide), after approving the benchmark design
in ticket v6kr:

> I'll probably like some of this you can have in the dashboard, the bridle-ui, maybe. I also work
> a lot, and I'm a data scientist as well, so having this in a Jupyter notebook in the future (like
> some ready-made Python functions I can call from the notebook to load up all the benchmark data
> and create some basic charts from it) would be something. That's a future thing. Don't do that
> right now, but add that to this theme.

> I forget what we're calling that. I called it ideas before. Maybe it's just a feature or
> something. Add a ticket that that could be something we do later. Again, that's an idea, a
> tentative idea for the future, so it's not something we're committed to. It's not something
> that's scheduled, but it's the kind of thing that should be brought up as part of a product
> manager review of the themes and epics that we should do.

## Sketch

- A small importable Python module next to the benchmark script (v6kr design: one run per
  directory with `manifest.json`, `samples.csv`, `events.jsonl`; published to the
  `bridle/benchmarks` branch and a dated folder in the workspace parent), with functions to load
  one run or all runs into data frames (idle and busy kept apart) and draw a few basic charts:
  CPU per hour, RSS, child processes, load and memory pressure over a run, and run against run.
- Maybe later: the same numbers on a bridle-ui page (the human's "in the dashboard, maybe").
- Theme: the one v6kr's benchmark epic sits in (the PdM places it).
