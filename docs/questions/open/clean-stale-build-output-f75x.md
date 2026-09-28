---
id: f75x
title: Clean stale build output
opened: 2026-09-28
repos: [bridle]
changes: []
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
