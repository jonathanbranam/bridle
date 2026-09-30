---
id: 5bw7
title: Traceability in older specs
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

From `docs/design.md` §15 @ c192bfc, item 11:

> **Traceability in older specs.** A newly imported spec has no `traces=`
> links. Either orphans are allowed to be the norm and links added as
> requirements are touched, or the import requires a linking pass. The first
> is cheaper; the second makes §7.4 useful sooner.

## Why it matters

It decides how soon [[docs/design/traceability|traceability]] pays off after
`bridle import openspec`.

## Notes
