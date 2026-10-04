---
id: 4f8y
title: agent rm fails when git no longer knows the agent's worktree (pruned record, directory left)
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-4f8y]
---

## The ask

The human, 2026-10-04:

```
% bridle agent rm docs-rename --delete-branch
error: internal: git ["worktree", "remove", "/Volumes/Data/work/bridle/wt/docs-rename"] failed: fatal: '/Volumes/Data/work/bridle/wt/docs-rename' is not a working tree
```

The directory exists with a `.git` file pointing at `.git/worktrees/docs-rename`, but git's record was pruned (`git worktree prune`) and the branch `bridle/docs-rename` is gone; its work landed long ago (bb8f8a23). `agent rm` should handle a worktree git no longer knows: when the path isn't a registered worktree, check it holds nothing uncommitted that matters (there's no index to compare, so refuse without `--force` if it holds files git can't account for, or just remove a directory whose gitdir is gone), remove the directory, and carry on removing the agent and (missing) branch.
