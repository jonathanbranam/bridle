---
id: m2fq
title: Merging main into a worker branch fails under merge.ff=only
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
---

## The problem

The human's global git config sets `merge.ff = only` (`git config --global --get merge.ff`
prints `only`). The worker instructions say to bring the branch up to date with a plain
`git merge main`:

- `workflow/base/skills/worker/SKILL.md:27`: "(`git merge main`, never `origin/*`)"
- `.bridle/roles/worker.md:27`: "`git merge main` (the ..."

Under `merge.ff = only`, git refuses that merge whenever the worker's branch has commits
that `main` doesn't, which is the normal case at the end of a task.

Observed 2026-09-28 16:30 UTC (events 22397–22404): worker `spike-path-rules` (Haiku) ran
`git merge main --no-edit`, `git merge main`, `git pull --no-rebase main`,
`git merge -m ... main`, then `git merge --no-ff -m ... main` before the merge went through.

The manager's merge already passes `--no-ff` (`.bridle/roles/manager.md:47`), so only the
worker side is affected.

Found by the data-contracts onboarding survey
(`docs/context/onboarding-data-contracts.md` §5), from findings recorded on data-contracts'
`adopt-branch-per-change-workflow` branch.

## Notes

Other projects bridle onboards run under the same global config.
