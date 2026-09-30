---
id: z4hd
title: "`bridle status` calls a stopped worker's empty branch merged, and suggests deleting it"
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [do-workers-resume-after-a-daemon-restart-2fkk]
---

## What happened

2026-09-30 02:16 UTC, right after the human restarted the daemon: `bridle status` printed

```
merged     stopped agents whose branch has landed: mark-unread (bridle rm <name> --delete-branch)
```

`mark-unread` (br-a03b) had been stopped by the shutdown (`daemon_shutdown`) before its first
commit. Its branch was still at the `main` it started from. The orchestrator relayed the `bridle
rm` command to the human as safe. It wasn't: the worker was resumed and landed its work later
(feb7e17). Had the human run it, the worktree with the uncommitted work would have been deleted
(`rm` refuses uncommitted changes without `--force`, which the orchestrator had just told the
human to use on another worker).

## Why

`worktree::is_merged` (`crates/bridle-daemon/src/worktree.rs`) returns true when the branch is an
ancestor of `HEAD`. A branch with no commits of its own is always an ancestor. The status listing
(`server.rs`, `merged_leftovers`) takes any `stopped` agent whose branch "is merged".

## Fix

- A branch counts as landed only if it has at least one commit of its own that reached `HEAD`
  (ancestor with commits beyond its fork point, or the `Branch:` trailer). An empty branch is
  "no work", not "merged".
- Leave `stopped` agents with reason `daemon_shutdown` out of the leftovers list: they're due
  a resume, not a removal.
- `bridle rm --delete-branch` should use the same test, and refuse (without `--force`) when the
  worktree has uncommitted changes, as it already does.
