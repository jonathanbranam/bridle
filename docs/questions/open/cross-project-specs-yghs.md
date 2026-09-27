---
id: yghs
title: Cross-project specs
opened: 2026-09-27
repos: [bridle, track-web, harness]
changes: []
specs: []
needs: []
see: []
---

## The question

From `docs/design.md` §15 @ c192bfc, item 8:

> **Cross-project specs.** harness consumes engine behaviour that is specified
> in track-web. The question is whether a harness scenario can reference a
> track-web requirement id (`tw:r-7fa2`), and whether impact checks follow that
> reference.

## Why it matters

"Engine first, host second" is enforced by cross-project edges
([[docs/design/coordination|coordination]]), but the specs themselves don't
link across repos yet.

## Notes

- [[docs/design/specs|Specs]], [[docs/design/impact-and-conflicts|impact and conflicts]].
- [[docs/design/agent-host/operating-model#Several projects at once|Several projects]] keeps project daemons entirely separate, which
  bears on where a cross-project check would run.
