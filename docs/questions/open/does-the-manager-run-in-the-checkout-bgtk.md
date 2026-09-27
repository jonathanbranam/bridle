---
id: bgtk
title: Does the manager run in the main checkout or its own worktree?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [93u2]
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
