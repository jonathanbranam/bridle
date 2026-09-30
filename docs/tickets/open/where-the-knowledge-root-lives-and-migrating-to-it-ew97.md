---
id: ew97
title: Where the knowledge root lives, and migrating to it
opened: 2026-09-27
repos: [bridle, track-web, harness, data-contracts]
changes: []
specs: []
needs: []
see: []
---

## The question

From `docs/design.md` §15 @ c192bfc, item 9:

> **Where the knowledge root lives, and migrating to it.** `design/` is only a
> default. The existing material has to be sorted into the tiers once per
> repo: track-web's `docs/`, harness's plan of record and `docs/arch/`,
> data-contracts' orientation docs. This is judgement work, not a script. It
> is probably a driver task per repo, reviewed by the human.

## Why it matters

The [[docs/design/knowledge-tiers|knowledge tiers]] only work once each repo's
material is sorted into them.

## Notes

- Bridle's own `docs/` was split on 2026-09-27 into `proposal/`, `context/`,
  `design/`, `spikes/` and `questions/` (see `docs/README.md`). That split is a
  first, informal pass at this sorting on bridle itself, and a data point for
  whether the tiers' names and boundaries fit real material.
