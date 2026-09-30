---
id: r2uq
title: Where does the bridle-workflow repo live?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
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

## Resolution

Not a separate repo: the human's words, 2026-09-28, "too much hassle."
Instead, `workflow/base/` and `workflow/packs/` live inside this repo
(P2-1), with `.bridle/config.toml`'s `workflow = "workflow"` pointing at
them. See the updated layer table and `workflow =` example in
[[docs/design/workflow-layers|workflow layers]]. The layer mechanism still
accepts a path or git url, so a project that does want a shared repo across
multiple projects still can; bridle itself just doesn't need one yet.

Addendum, the human via pm-1, 2026-09-28, on how updates flow once a shared
`workflow` exists: updates apply automatically on `bridle sync` — no rev
pinning for the common case; a project reads a changelog of what changed
since its last sync (e.g. `workflow/CHANGELOG.md`) rather than diffing
blind; and a project that disagrees with a specific rule opts out with a
local `override: disable` and a `reason`, not by pinning or forking. See
[[docs/design/workflow-layers|workflow layers]] for where this landed in the
design.

Resolved 2026-09-28.
