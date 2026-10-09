---
id: x3xk
title: "is_merged calls a reused branch landed: an old Branch: trailer hides newer unlanded commits"
kind: bug
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-x3xk]
closed: 2026-10-09T23:11:01Z
---

## The ask

`bridle status` listed `self-upgrade` under "merged: stopped agents whose branch has landed" (2026-10-04), and the hint beside it is `bridle rm <name> --delete-branch`. The branch has **not** landed: `git cherry main bridle/self-upgrade` shows two commits not on `main` (98615ed8 br-88d4 "self_upgrade = release", dbdc47f6 "Release upgrade finds the running binary with exe_path()"), and its worktree has staged changes. Following the hint with `--force` would have deleted that work.

Cause: `worktree::is_merged` (`crates/bridle-daemon/src/worktree.rs:477`) counts a branch as merged when any commit reachable from `HEAD` carries a `Branch: <branch>` trailer. `main` has 7d29cfd4 (br-38c8) with `Branch: bridle/self-upgrade`, an earlier landing from the same branch; the worker then got more work on that branch. A trailer should count only if the branch tip has no commits after what that landing took (e.g. the landing commit's tree or the tip recorded at landing), or the trailer should carry the tip commit.

Also used by `supervisor.rs:2881`; check what it gates there.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
