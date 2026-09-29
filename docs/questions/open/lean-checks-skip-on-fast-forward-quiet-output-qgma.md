---
id: qgma
title: Lean checks: land skips the check when the tree is already checked; check output goes to a file
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [run-only-the-tests-a-change-can-affect-mt7r]
---

## The ask

The human, verbatim (2026-09-29, via the advisor), on `bridle land` running `just check`:

> do we run this even on a ff merge? That seems redundant. IDC if GH Actions runs it, but my
> local doesn't need to for a ff merge. Now, if there is any rebase or non-ff merge happening,
> then yes, always run it.

And on how agents run the check:

> does the worker read the entire output of the build? We want to be lean on token usage. Also
> lean on CPU; ideally, the full test sutie would run, piped to a file, not read by the agent,
> and the agent only reads the file if the exit code is not success. Also, perhaps useful the
> agent should tail the file and be sure that the proper number of tests ran.

## Today (at 23435c1)

- `bridle land` squashes the branch onto the integration branch in its own worktree, then always
  runs `[integration] check` (`just check`, `.bridle/config.toml`) before moving the ref
  (`crates/bridle-daemon/src/integrator.rs`, `land` and `run_check`). No fast-forward exception.
  `run_check` already captures output and returns only a tail on failure.
- The manager checks by hand, before landing, that `git merge-base --is-ancestor main
  bridle/<name>` passes (`workflow/base/roles/manager.md`, "Land completed work"). When it does,
  the squash commit's tree is identical to the branch tip's: git would have fast-forwarded.
- The worker runs `[commands] check_worker` = `just check-affected` (only the changed crates and
  their dependents), shown as `2>&1 | tail -n 30` and judged by exit status
  (`workflow/base/roles/worker.md`, "Reading and output"). So the agent reads 30 lines even on
  success, and the full suite locally runs only at `land`.

## Shape

1. **Land skips the check when there's nothing new to check.** If the integration branch is an
   ancestor of the task branch (the squash tree equals the tip tree), skip `[integration] check`
   and say so in the land notes. Otherwise (the integration branch moved: a real merge) always
   run it. CI on `main` runs regardless.
2. **Tie the skip to the commit the worker checked.** The worker reports the commit its green
   check ran on (e.g. in `bridle task summary` or a field on the land request); `land` skips only
   if the branch tip is still that commit, else runs the check. Without this, a commit after the
   check would land unchecked until CI.
3. **Check output goes to a file, not the context.** Worker (and manager, if it ever runs one):
   `just check > <file> 2>&1`, judge by exit status; read the file (tail) only on failure. On
   success, read just the test summary line (nextest's `Summary [...] N tests run: N passed ...`)
   to confirm tests actually ran and none failed. Update the "Reading and output" lines in
   `workflow/base/roles/worker.md` and `manager.md`.

## Open points for the human

- **Which suite the worker runs.** The human's words say the full suite. Today the worker runs
  `check-affected` and `land` runs the full suite; with (1), a fast-forward landing would run no
  full suite locally, only CI. Either the worker goes back to `just check` (full, to a file; more
  CPU per worker, none at land), or it keeps `check-affected` and full coverage relies on CI.
- ~~**"The proper number of tests."**~~ Settled, below.

## The test count (the human, 2026-09-29)

> Yes, by "the proper number of tests" I mean, not 0, not 10,000 but some expected number; just
> as a sniff check.

So: a sanity range, not an exact match. On success, read the summary line's count and compare
it with the last full-suite count on `main` (recorded wherever is simplest, e.g. by `land` or
CI). Well outside it (say under half or over double; the band is a detail for the build) is
treated as a failure to look into, not a pass. Tests added or removed by the change itself
stay well inside the band.
