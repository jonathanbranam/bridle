---
id: bgtk
title: Does the manager run in the main checkout or its own worktree?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: [docs/design/agent-host/roles-and-config.md]
needs: []
see: [93u2]
closed: 2026-09-30T05:12:44Z
---

## The question

From `docs/agent-host.md` §13 @ c192bfc, item 4:

> **Does the manager run in the main checkout or its own worktree?** v1 says
> the checkout (`workdir = "repo"`), since it coordinates rather than edits.
> If it starts merging, it needs its own integration worktree.

## Why it matters

The clone's checkout is shared with the human.

## Notes

Related: [[integration-branch-or-merge-to-main-93u2|integration branch or main]].

## Resolution

The manager runs in the clone's checkout. It coordinates and doesn't merge:
the integrator is bridle itself, not an agent
([[docs/design/roles-and-lifecycle|roles]]), so the integration worktree
belongs to bridle when the integrator is built (P5). Recorded in
[[docs/design/agent-host/roles-and-config|roles and config]]. Whether bridle
integrates to a branch or to main is still open:
[[integration-branch-or-merge-to-main-93u2|integration branch or main]].
