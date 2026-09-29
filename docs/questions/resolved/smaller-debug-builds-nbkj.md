---
id: nbkj
title: Smaller debug builds
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [58f6cef]
specs: []
needs: []
see: [f75x, m3wq]
---

## What happened

Filed by the advisor. On 2026-09-28 ~16:00Z, `du -sh` in the main clone showed `target` at 28G: `target/debug/deps` 21G (about 186,000 `.o` files, 23.3G by `du`), `target/debug/incremental` 7G. Stale copies pile up under new hashes on each rebuild (12 copies of `libbridle_claude`, 5-6 of most test binaries). On macOS a dev build keeps each crate's `.o` files for debug info, and cargo never removes old ones. The clone is rebuilt after nearly every merge (`just check` by the manager and orchestrator). `../wt` held another 9.5G, about 4G per worker worktree. `/Volumes/Data`: 793G, 393G free.

The human, verbatim (2026-09-28): "28G seems incredibly large that can't be right", then,
on the advisor's two options: "File both 1 and 2. ... This is all low priority work,
perhaps after P2. I can cargo clean for now."

## The advisor's suggestion (option 1)

Keep less debug info in the dev profile, e.g. `debug = "line-tables-only"` under
`[profile.dev]` in the workspace `Cargo.toml` (it has only
`[profile.dev.package."*"] opt-level = 1` today). Backtraces keep file and line; each
build is several times smaller. Measure `target` before and after on a clean build.

## Notes

- Low priority; after P2 (the human).
- Worker worktrees build the same profile, so this shrinks `../wt` too.

## Resolution

Resolved by 58f6cef: `debug = "line-tables-only"` in the dev profile for smaller debug builds.
