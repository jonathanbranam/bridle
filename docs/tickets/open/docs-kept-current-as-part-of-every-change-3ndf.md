---
id: 3ndf
title: Docs kept current as part of every change
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [product-briefs-of-how-bridle-works-today-8awb]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> This would also be a project-wide rule for Bridal itself, and in general, for most tools,
> documentation needs to be kept up to date. That's a task that a worker should do, or we could
> also have a really separate small agent that spins up and checks documentation. I think for
> now, just workers should have, as part of their tasks, when they are done with a change,
> verifying the documentation or making any updates as needed to the documentation.

## What exists today (advisor, 2026-09-29, at f5cd08b)

- `workflow/base/roles/worker.md` already says: "**Docs**: if you change behaviour the
  project's docs describe, update the doc in the same commit." It's one line among many, not a
  step at the end of a task, and nothing checks it.
- `CLAUDE.md` asks that `docs/design/agent-host/`, `cli.md` and `storage.md` be kept in step.

## Shape (advisor's reading of the ask)

- A base rule (`workflow/base/rules/`, so every project gets it): when a worker finishes a
  change, it checks the docs that describe the changed behaviour (including the briefs from
  [[product-briefs-of-how-bridle-works-today-8awb|product briefs]]) and updates them, or says
  in its done report that none needed it.
- Made a step of the worker's finish (the worker skill), not only a line in the role.
- For now workers only. A separate docs-checking agent is a possible later step, not now.
