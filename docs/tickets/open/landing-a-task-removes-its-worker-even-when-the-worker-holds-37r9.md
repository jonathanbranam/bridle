---
id: 37r9
title: Landing a task removes its worker even when the worker holds another task's uncommitted work
kind: bug
opened: 2026-10-10
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

## What happened

2026-10-10 ~11:25Z: manager-2 landed br-4vmc (f49d8ebe). Landing removes the worker and its
worktree. The same worker, w4vmc, had been handed br-v6kr (the baseline sampler) at ~08:30Z
because no slot was free, and was running it on a second branch, `bridle/w4vmc-v6kr`, in the same
worktree. The sampler script and ~2 h of CSV were uncommitted there, so the landing deleted them
and killed the sampler. The leftover branch holds only the br-4vmc commit.

## The ask

The landing's removal step should refuse (or keep the worker and say why) when the worker has
uncommitted changes, another branch with commits beyond the one landed, a claimed task other than
the one landing, or a running background job. Today it checks none of these.

## Cost of not doing it

Any time a worker is reused for a second task (done when slots are full), a landing can silently
destroy work in progress. Here: ~2 h of sample data and the script, redone by a fresh worker.
