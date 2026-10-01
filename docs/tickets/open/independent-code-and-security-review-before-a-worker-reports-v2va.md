---
id: v2va
title: Independent code and security review before a worker reports done
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [hvxk, 2bzw]
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

The human, 2026-10-01, verbatim (follow-up):

> A task can require certain reviews and a workflow can require certain reviews. Eg a workflow
> might dictate: every change to module/component/entire project needs an independent security
> review. Or it could be optional and left to the discretion of the PM or whoever creates the
> ticket / task could indicate that a specific review is necessary.

The human, 2026-10-01, verbatim (second follow-up):

> Independent reviews should be signed and added to the task including the commit sha and the
> task body - particularly important for security reviews.
>
> If a human review is required (same options apply) that also should be signed which the humans
> token in the same way.

## What's there now (at 0c741f6)

- `docs/design/roles-and-lifecycle.md` (future work) has a **Reviewer** role: "strong model,
  never the task's implementer", checking "whether a diff matches its plan and specs", and an
  `in_review` state between `claimed` and `integrated`. Nothing implements it. There is no
  security review anywhere.
- `docs/design/gates.md` (future work) has an accept gate `default = "reviewer+tests"`.
- How consumers opt in or out: `docs/design/workflow-layers.md`. Base rules apply by default,
  and a project opts out with `override: disable` and a `reason`; packs (L2) are opt-in.
- Today a worker runs `just check`, hands off, and the manager lands. No review step between.
- Components (`docs/design/components.md`, `bridle task new --component`) already scope a task
  to part of a repo.
- Signing: today the daemon records each call's `actor` from its bearer token, and the store
  keeps only token hashes. `docs/design/agent-host/principals.md` ("What this is and isn't")
  calls that "attribution that honest agents can't get wrong by accident, not a security
  boundary". How strong provenance should be is open in
  [[how-strong-agent-provenance-should-be-2bzw|2bzw]].
