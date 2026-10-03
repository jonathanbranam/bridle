---
id: m3wg
title: Isolation between interactive sessions that commit to the shared working copy
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [hv8e, k7tm]
tasks: []
---

## The ask


A question to consider, not a build. The human, verbatim (2026-10-03, via the advisor):

> Something else to file a question ticket about is whether we need more isolation for the work
> that orchestrators and advisors are doing. Right now, we just had the advisor commit some
> tickets and push them, and it said it included a manager's unpushed change. That doesn't
> concern me at all. It's fine to have that pushed, but it's just a question to consider about
> whether we need any level of isolation between some of these sessions that are making changes,
> even if they're only doc changes. We're all working on the same work tree, so should there be a
> different branch? For documents like tickets that get merged into main, I think with KISS, I'd
> prefer not to unless it's a real problem, but it's maybe something to consider.

And, on the related question ([[tickets-and-docs-in-bridle-s-repo-or-a-separate-repo-or-subm-tkav|tkav]]):

> It would be worth at least noting that we have overlapping work in the same branch, in the same
> work tree, at the same time.

## Today

- The orchestrator and every advisor share one working copy of `main` (this clone). Each commits
  its ticket edits there and pushes. Workers and managers work in their own worktrees and
  branches.
- The advisor role already says to `git add` specific files, never `-A`, because other advisors
  may have uncommitted changes. But a commit made on `main` and not yet pushed by one session is
  pushed by the next session's `git push`, as happened here.
- No incident so far. The human's lean (KISS): no separate branch unless it becomes a real
  problem.

## The question

Do sessions that change docs (orchestrator, advisors) need any isolation from each other: a
branch or worktree each, or just a rule (e.g. push straight after committing)? Or is the shared
copy fine?
