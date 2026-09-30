---
id: f75x
title: Clean stale build output
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [5b68552]
specs: []
needs: []
see: [nbkj, m3wq]
---

## What happened

Filed by the advisor. On 2026-09-28 ~16:00Z, `du -sh` in the main clone showed `target` at 28G: `target/debug/deps` 21G (about 186,000 `.o` files, 23.3G by `du`), `target/debug/incremental` 7G. Stale copies pile up under new hashes on each rebuild (12 copies of `libbridle_claude`, 5-6 of most test binaries). On macOS a dev build keeps each crate's `.o` files for debug info, and cargo never removes old ones. The clone is rebuilt after nearly every merge (`just check` by the manager and orchestrator). `../wt` held another 9.5G, about 4G per worker worktree. `/Volumes/Data`: 793G, 393G free.

The human, verbatim (2026-09-28): "28G seems incredibly large that can't be right", then,
on the advisor's two options: "File both 1 and 2. ... This is all low priority work,
perhaps after P2. I can cargo clean for now."

## The advisor's suggestion (option 2)

Remove stale build output now and then, so `target` doesn't grow without bound: e.g.
`cargo clean` as part of the release step (`.bridle/roles/orchestrator.md`, where
releases are cut), or `cargo sweep` to drop artifacts older than some age. It must not
run while `just check` is building in the same clone.

## Notes

- Low priority; after P2 (the human).
- Worker worktrees' `target` goes away with `bridle rm --delete-branch` after merge;
  this is about the long-lived main clone.

- Observed by the orchestrator, 2026-09-28: after `cargo clean` in the main clone
  (41 GiB, ~240k files removed, ~15:50 UTC), `just check` on `main` went from about
  400 s to about 35 s, with individual tests about 10x faster (the governor tests
  from ~15 s to ~2 s each), at similar load. Nothing else changed in between except
  P2-3's merge, which added tests. Cause not investigated; one guess is background
  indexing or I/O on the large `target/`. If it holds, stale build output costs
  test time, not just disk.

## Resolution

Resolved by 5b68552: `just clean-stale` recipe (docs/design/build-maintenance/clean-stale-artifacts.md), used by the orchestrator role at release.
