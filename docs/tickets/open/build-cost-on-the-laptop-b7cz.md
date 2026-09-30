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

## The warm cache goes stale (the advisor, 2026-09-29)

The human, verbatim (2026-09-29, via the advisor):

> warm or shared build cache seems like a route to go; if that work is straightforward enough to
> do, I'd start there.

Option 2 is already live (`warm_target` defaults to on), but its source is the main clone's
`target/`, last built 2026-09-28 17:45 (after the human's `cargo clean`). Nothing rebuilds it as
`main` moves: the orchestrator no longer runs `just check` on `main`. So every new worktree
starts from a day-old cache and recompiles whatever changed since. The integration worktree's
`target/` (`<workspace>/integration`, built at every `land`) was fresh at 2026-09-29 17:13, but
with qgma `land` stops building on a fast-forward, so it will go stale too.

Next, cheapest first:

1. **Measure** (still open): time and load for one task's first build, today.
2. **Keep the warm source fresh**: e.g. after `land`, run an incremental `cargo build
   --workspace --all-targets` in the integration worktree (or the clone) and warm from there.
   One incremental build per landing, off the worker's critical path.
3. **sccache** (option 3) only if 1 and 2 leave the build as the bottleneck: a new tool on every
   host, and the workspace's own crates hit poorly across worktree paths.

### Measure before building (the human, 2026-09-29)

> yes, let's not build something on a whim; measure it when possible vefore commiting to more
> work. Again I'm afraid I'm getting off task on the wrong things. Better to get bridle stablea
> nd working then use it more.

The advisor's second look: a fresh cache may save little. Third-party crates (most of a cold
build) are already in even a stale copy, and nearly every task edits `bridle-daemon` or `bridle`,
which the worker then recompiles with every test binary regardless. Freshness only saves changes
that landed in the leaf crates (`bridle-api`, `bridle-claude`, `bridle-spec`) since the copy.

So: no build work on "keep the warm source fresh" (step 2 above) or sccache until a measurement
says it's worth it. When the machine is quiet, time one worker's first build from a fresh and from
a stale `target/`. Low priority: stability comes first.

## Measured (2026-09-29)

Step 1 of "Measure before building". Scratch worktree of `main` on the same APFS volume as
the workspace (a scratch dir under `$TMPDIR` is another volume: `cp -c` fails there with
"Cross-device link", so those runs were discarded). 16 CPUs. `cargo build --workspace
--all-targets`, one build at a time. **The machine was not quiet**: idle load was 20 to 40
(Obsidian, Spotlight `mds`) and other agents' work overlapped some runs (a daemon self-upgrade
build interrupted one run, which was redone), so peak loads are inflated and wall times are
noisy: read them as rough.

| Case | Wall | Peak 1-min load | Notes |
|---|---|---|---|
| a. cold, empty `target/` | 3m36s (an earlier run on another volume: 2m41s, load 67) | 122 | full workspace |
| b. warmed from the main clone's `target/` | 3m40s + 7s clone | 117 | 210 crates recompiled |
| c. warmed from `integration/target/` | 1m39s + 35s clone | 69 | 6 crates recompiled |
| d. `just check` on c's warm state | 2m38s | 119 | nextest 53.8s, 784 tests, 1 failed |

- The main clone's `target/` was last built 2026-09-28 17:46 (`debug/deps`); the integration
  worktree's 2026-09-29 17:38. The clone is a day old and about `main` as of before the `cargo
  clean`'s rebuild: the warm copy saved nothing measurable over cold (b is about a). The
  integration copy left only 6 crates to rebuild.
- Cloning is not free: 7s for the clone's `target/`, 35s for the integration one (larger, 38k
  files in `deps`), both under load.
- d's one failure was `bridle-spec` `parses_fast`, a timing assertion that fails at load 118, not
  a regression. No `last-full-test-count` file exists to compare the 784 against.

Recommendation: keeping the warm source fresh is worth it: a fresh copy (c) roughly halved the
first build against a stale one (b), which was no better than cold.
Do step 2 (an incremental build in the integration worktree after each `land`, warm from
there), and do not add sccache yet.
