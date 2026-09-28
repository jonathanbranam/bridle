---
id: zk4p
title: Should merging a worker's branch require the merger's own passing test run?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [n6gy]
---

## The question

During bridle's first self-hosted run, a merge went in on the strength of a
worker's reported green `just check`, and it turned out to make `main` fail
a test deterministically. This was only caught because a second party
independently re-ran the checks after the merge.

Should whoever merges a worker's branch into `main` be required to get (or
independently reproduce) a passing test run on the exact commit being
merged themselves, rather than trusting the worker's report — and if so,
should that happen more than once, given the flakiness noted in
[[flaky-time-based-tests-on-a-loaded-machine-n6gy|flaky time-based tests on
a loaded machine]]?

## Why it matters

A worker's self-reported `just check` result isn't independently verified
anywhere in the current merge flow (`.bridle/roles/worker.md` /
`docs/design/agent-host/roles-and-config.md`), so a false-green report — or
a real pass that doesn't hold up on a second run — can reach `main`
undetected until someone happens to re-check it.

## Resolution

Yes, independent verification is required, and it happens twice by different
parties. The merge flow in [[docs/design/agent-host/operating-model|operating
model]] (§ "Merging completed work", lines 62–86) establishes:

1. The **development manager** (the merger, a different party than the worker)
   verifies the branch before merge: it confirms `main` is an ancestor
   (`git merge-base --is-ancestor main bridle/<agent>`), the worktree is clean,
   and the diff matches the task (lines 70–73).

2. The **orchestrator** (again, a different party) verifies `main` **after**
   each merge with `just check` **twice**, off load (line 84–86). If it's red,
   nothing else merges until it's green.

The split between development manager (per-branch, before merge) and
orchestrator (post-merge, twice, on main) was established in ticket
[[split-the-manager-into-product-and-development-managers-tx3f|tx3f]]
and fulfills the requirement: independent verification by a different party,
and repeated verification to catch flakiness under load (as noted in
[[flaky-time-based-tests-on-a-loaded-machine-n6gy|n6gy]]).

Resolved 2026-09-28.

---

**Bridle task state**: br-fe7e references this ticket. Since `bridle task` has
no close/resolve subcommand, the task remains open and its path should be
updated by bridle when it notices the ticket has moved to `resolved/`.
