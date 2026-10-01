---
id: v2va
title: Independent code and security review before a worker reports done
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [hvxk]
---

## The ask


The human, 2026-10-01, verbatim (via the advisor):

> I think a step we need to consider adding to workflow (main bridle provides this, consumers opt
> in or out):
>
> - code review by independent agent
> - security review by independent agent
>
> Don't rush these but note that it should be something available. These should pass before a
> worker reports back to manager that the work is done.

## What's there now (at 0c741f6)

- `docs/design/roles-and-lifecycle.md` (future work) has a **Reviewer** role: "strong model,
  never the task's implementer", checking "whether a diff matches its plan and specs", and an
  `in_review` state between `claimed` and `integrated`. Nothing implements it. There is no
  security review anywhere.
- `docs/design/gates.md` (future work) has an accept gate `default = "reviewer+tests"`.
- How consumers opt in or out: `docs/design/workflow-layers.md`. Base rules apply by default,
  and a project opts out with `override: disable` and a `reason`; packs (L2) are opt-in.
- Today a worker runs `just check`, hands off, and the manager lands. No review step between.
