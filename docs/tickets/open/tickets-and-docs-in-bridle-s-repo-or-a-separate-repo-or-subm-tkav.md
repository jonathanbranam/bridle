---
id: tkav
title: Tickets and docs in bridle's repo, or a separate repo or submodule
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [hv8e, k7tm, d3wq]
tasks: []
---

## The ask


A question to note, not a build. The human, verbatim (2026-10-03, via the advisor):

> The other question is regarding tickets and some of the documentation. I think I've asked this
> before, but does it belong in this repo, or should it be a different repo, a submodule, or
> something? Even as I'm voicing this, I think we haven't hit any incidents really related to
> this, and it's hard to imagine that happening. It would be worth at least noting that we have
> overlapping work in the same branch, in the same work tree, at the same time.

## Notes

- Tickets, design docs and specs live in bridle's own repo, on `main`, next to the code.
- Related earlier asks: [[which-docs-live-in-bridle-and-which-in-markdown-hv8e|hv8e]] (which
  docs are bridle records vs markdown files) and [[tickets-and-tasks-why-both-k7tm|k7tm]]
  (tickets vs tasks). Neither asks about a separate repo.
- A cost of sharing the repo seen already: docs-only commits trigger work meant for code, e.g.
  self-upgrade restarting for docs-only commits
  ([[self-upgrade-reports-the-wrong-commit-and-restarts-for-docs-d3wq|d3wq]]).
- The overlapping work in one branch and working copy is
  [[isolation-between-interactive-sessions-that-commit-to-the-sh-m3wg|m3wg]].
- No incident so far; the human finds one hard to imagine.
