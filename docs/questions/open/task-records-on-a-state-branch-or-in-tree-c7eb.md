---
id: c7eb
title: Task records on a state branch or in-tree?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [c5a8]
---

## The question

From `docs/design.md` §15 @ c192bfc, item 1:

> **Task records on a state branch or in-tree?** (§10.1) The recommendation is
> a state branch. In-tree is easier to browse and review with code.

And from §10.1:

> The alternative, task files in-tree under `.bridle/tasks/` on the main line, is
> easier to browse next to code but brings back the worktree-visibility and
> churn problems.

## Why it matters

It decides where every durable task record lives, and P0 of the
[[docs/proposal/build-order|build order]] builds state-branch persistence.

## Notes

- Design: [[docs/design/storage#The state branch|the state branch]].
- `docs/agent-host.md` §2 step 3 moves the state branch's worktree from
  `~/.bridle/state/<project>/` to `<workspace>/.bridle/state/`, arriving with
  tasks in P0. v1 has no task store.
- Related: [[where-questions-live-on-the-state-branch-c5a8|where questions live on the state branch]].
- Prior art on the other side: the ticket system in `workflow-instructions/`
  keeps tickets in-tree under `docs/tickets/`, and these question files are
  in-tree too.
