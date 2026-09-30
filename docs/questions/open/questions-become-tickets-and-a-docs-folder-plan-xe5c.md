---
id: xe5c
title: "questions/" becomes "tickets/", and a planning session on the docs folders
opened: 2026-09-30
repos: [bridle, meta-notes]
changes: []
specs: []
needs: []
see: [hv8e, p2ys, k7tm]
---

## The ask

The human, verbatim (2026-09-29, about 23:25 ET), after the orchestrator asked where meta-notes'
tickets live:

> they definitely live in the meta-notes repo and currently the branch is bridle-adopt, but we
> need a ticket to fix up that folder name. questions/ is not good! It needs to be tickets, not
> questions I think. That is something to consider for our repo too... I'm looking at docs
> actually, we should have a planning session about it, there are a lot of folders. they might
> all be worthwhile, but we don't want to add very many more.
>
> questions, though, is the wrong name, chosen at a PIT and needs to be tickets b/c that's what
> I'm going to call them.

## Decided

- **The folder is `tickets/`, not `questions/`**, in every project bridle runs. The name is
  settled; only the migration is work.
- **meta-notes' tickets live in the meta-notes repo**, on `bridle-adopt` for now. They start
  under `tickets/` (meta-notes has no `questions/` folder on `bridle-adopt` yet, so there's
  nothing to rename there; just don't create one).
- **bridle renames `docs/questions/` to `docs/tickets/`** (138 files today, `open/` and
  `resolved/`). References to update: `docs/README.md`'s conventions, `CLAUDE.md`, the role
  prompts and rules in `workflow/`, scripts, and anything in the crates that names the path
  (about 25 hits for `questions/` in `crates`, `scripts` and `workflow`). One commit, `git mv`,
  landed at a quiet point, since every agent's prompt points at the old path.

## For a planning session with the human

bridle's `docs/` has eight top-level folders today: `briefs` (4 files), `context` (10),
`design` (30), `proposal` (4), `questions` (138), `research` (1), `spikes` (26), plus
`README.md`. The human: they "might all be worthwhile, but we don't want to add very many
more". The session decides which stay, which merge (e.g. `research` into `spikes`, `proposal`
into `design`?), and the layout new projects get. Related: hv8e (which docs live in bridle's
records and which in markdown), p2ys (ticket state without moving files between `open/` and
`resolved/`), k7tm (tickets and tasks, why both).
