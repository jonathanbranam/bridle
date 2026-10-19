---
id: br-wnuf
title: "Each landing leaves its integrate/<task> scratch branch behind: over 500 piled up"
kind: bug
created: 2026-10-19
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The integrator makes a scratch branch `integrate/<task>` for each landing
(`prepare_worktree` in crates/bridle-daemon/src/integrator.rs, `checkout -f -B integrate/<task>`) and
never deletes it. On dalek on 2026-10-18 there were over 500 `integrate/br-*` branches in bridle's clone,
one per ticket ever landed. After a landing finishes (landed, failed or abandoned), switch the
integration worktree off the scratch branch and delete the branch. On the next start, also delete
`integrate/*` branches whose landing is over. Never delete the one a landing in progress is using.

Worker branches whose agent is gone (`bridle/*` with no agent and no worktree) are the same kind of
leftover. Today only the manager's cleanup step removes them, and only for an agent it removes. Cover
the local ones here too, but only a branch whose ticket is integrated or dropped and whose commits are
on the integration branch. Remote branches are br-t7ya's.

## Where this came from

The human, 2026-10-18 ~9:30 PM ET (to aide), on `git branch` in bridle's clone: "seems like there are a
lot of branches sitting here". Of ~575 `bridle/*` and `integrate/*` branches, 534 were merged into main.
The human deleted them by hand. Branches cost no disk or scanning (that is br-mhfx); this is clutter in
every `git branch`.
