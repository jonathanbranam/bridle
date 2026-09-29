---
id: b7cz
title: The cost of builds: every worktree compiles the whole workspace from scratch
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [mt7r, c8qw, f75x, nbkj]
---

## The ask

The human sees the laptop as the bottleneck: it's barely usable while bridle runs, and
they're weighing moving bridle to the Intel NUC (4 threads; `docs/context/nuc-host.md`). The
advisor, passing it on (2026-09-28): load hit 96 on 16 cores, mostly from each worker's
worktree compiling and testing the whole workspace from scratch. They suggest a shared compile
cache plus `check-affected` for workers' own runs, which likely buys more than a second
machine. The human has raised similar concerns before.

## What happens today

- Each worker gets a fresh worktree (`wt/<name>`) with an empty `target/`, so every task starts
  with a full debug build of the workspace (about 2.5 GB, e.g. `wt/neutral-roles/target`)
  before `just check` even runs. With two workers and a merge, that's three cold builds.
- Per merge, `just check` ran once for the worker, twice for the orchestrator and on two GitHub
  runners. The orchestrator's two runs stopped on 2026-09-28 (6e5ad8f), so one local run is
  left.
- Measured on 2026-09-28: 1-minute load between 40 and 131 during merges, and the battery
  draining about 1.3% a minute on a drive.

## Options, cheapest first

1. **Workers run `just check-affected`** (mt7r; the recipe exists) instead of the full
   suite; CI runs everything on `main`. Cuts test time. The build still happens.
2. **Warm the new worktree's `target/`**: at spawn, copy the main clone's `target/` into the
   worktree (APFS `cp -c` makes a copy-on-write clone: near-instant, no extra disk). The first
   build is then incremental rather than cold. Small change in `worktree.rs`, macOS-first,
   with a plain copy (or nothing) elsewhere.
3. **sccache** (`RUSTC_WRAPPER=sccache`): shares compiled crates across worktrees. Paths
   differ per worktree, which lowers the hit rate for the workspace's own crates; deps hit
   well. It's a new tool to install on every host.
4. **A shared `CARGO_TARGET_DIR`**: not safe for concurrent workers. Cargo locks the
   directory, so builds serialise, and different branches overwrite each other's artifacts.
   Not recommended.
5. **Fewer concurrent builds**: `bridle budget max-workers 1` when the human needs the
   laptop (y2eb; live now).

Recommendation: 1 and 2 together. Both are small, need no new tools and work on the NUC too.
Measure load and wall time for one task before and after.

## The NUC

The advisor's read, which the orchestrator agrees with: the NUC suits light project daemons
(meta-notes: Vimscript and pytest), not bridle's Rust builds. The seven-day window (55% on
2026-09-28) is more likely to be the long-run limit than CPU.

## Progress (2026-09-29)

Options 1 and 2 are built (db79e38): new worktrees' `target/` is warmed with an APFS clone
(`[worktrees] warm_target`), and workers gate on `commands.check_worker` (e.g.
`just check-affected`). Still open: the before-and-after measurement of load and wall time,
and the NUC question (whether the NUC should run only light project daemons).
