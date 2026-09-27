---
id: r2uq
title: Where does the bridle-workflow repo live?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

From `docs/design.md` §15 @ c192bfc, item 4:

> **Where does `bridle-workflow` live?** It could be its own repo or a
> `workflow/` directory inside this repo. The recommendation is its own repo:
> it changes on a different cadence from the binary, and projects pin it.

## Why it matters

Every project resolves its base and pack layers from it
([[docs/design/workflow-layers|workflow layers]]).

## Notes

P2 of the [[docs/proposal/build-order|build order]] creates it.
