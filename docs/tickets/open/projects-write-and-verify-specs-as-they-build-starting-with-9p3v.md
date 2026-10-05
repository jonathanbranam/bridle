---
id: 9p3v
title: Projects write and verify specs as they build, starting with bridle-ui
kind: feature
opened: 2026-10-05
repos: [bridle, bridle-ui, track-web]
changes: []
specs: []
needs: []
see: [qpr7, yghs]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 ~9 PM ET, to the bridle-ui aide; dictated; item 1 of a message
whose item 2 is the status-versus-action ticket):

> 1. Are you writing specs? I'm realizing that most of my projects are not writing specs, when one
> of the main ideas of this solution is to write and verify specs. I'd like to start doing that so
> that we can divide that part of the system as we're building things.

## Context

- bridle's spec tooling: "Built, not wired in: every command on this page (`bridle spec`, `goals`,
  `arch`, `explore`), `spec coverage`, `spec export --task`, ... and the pytest and vitest adapters;
  bridle uses them for its own specs (`design/specs/`, checked in `just check`); no other onboarded
  project does yet (track-web, bridle-ui and data-contracts have no `design/specs/`)"
  (`docs/design/spec-flow.md`, status line).
- bridle-ui has no specs, no `docs/` and no `design/` folder; none of tonight's bridle-ui tasks
  wrote or checked a spec.
- An open question with bridle's aide (qpr7 follow-up): whether the spec flow becomes a rule for
  bridle's own tasks.
