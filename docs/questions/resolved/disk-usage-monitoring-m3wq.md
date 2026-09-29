---
id: m3wq
title: Disk usage monitoring
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [5e55d8b]
specs: []
needs: []
see: [nbkj, f75x]
---

## What happened

Filed by the advisor. On 2026-09-28 ~16:00Z, `du -sh` in the main clone showed `target` at 28G: `target/debug/deps` 21G (about 186,000 `.o` files, 23.3G by `du`), `target/debug/incremental` 7G. Stale copies pile up under new hashes on each rebuild (12 copies of `libbridle_claude`, 5-6 of most test binaries). On macOS a dev build keeps each crate's `.o` files for debug info, and cargo never removes old ones. The clone is rebuilt after nearly every merge (`just check` by the manager and orchestrator). `../wt` held another 9.5G, about 4G per worker worktree. `/Volumes/Data`: 793G, 393G free. Nobody noticed until the human ran `du`.

The human, verbatim (2026-09-28):

> Also file an additional ticket that we should have periodic, maybe every hour or two,
> disk usage monitoring; the bridle work system will die if I run out of disk space; any
> unchecked growth should be investigated and remediation suggested. This is all low
> priority work, perhaps after P2.

## What the human wants

- A check every hour or two of disk usage: free space on the volume, and the size of
  what bridle's work grows (the clone's `target`, worker worktrees, bridle's own data).
- Unchecked growth is investigated, and a remediation suggested.

## Notes

- Low priority; after P2 (the human).
- Open: who runs it (the daemon on a timer, or a role such as the orchestrator), and
  where a finding goes. Under
  [[stop-status-notes-to-the-human-inbox-kp3f|kp3f]] the human's inbox is only for
  things they must act on, so only a real problem should reach it.

## Resolution

Resolved by 5e55d8b: the daemon's periodic disk usage monitor (`crates/bridle-daemon/src/disk.rs`), documented in docs/design/agent-host/operating-model.md.
