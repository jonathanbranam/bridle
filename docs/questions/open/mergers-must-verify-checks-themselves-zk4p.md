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
