---
id: 63rv
title: Trial adoption on a bridle branch
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [61ce8ed, 7ffae38, 0c84f5d]
specs: []
needs: [rxe8]
see: [rxe8, d9nu, ajqa, u8sm, 8xhh]
---

## The ask

The human, verbatim (2026-09-28):

> don't make any changes to my existing projects without clear review and approval by me.
>
> I think my ask will be along these lines:
>
> 1. Leave main (and dev) alone
> 2. create a new branch for the bridle migration (and state)
> 3. constrain work to that new branch until ready
> 4. I'll review and approve the bridle adoption before replacing main+dev branches with
>    bridle
>
> I'll probably run a few changes through bridle to see how it works before approving, so
> that should be doable - the change to control git branches that manager and workers use
> must land before this so they can target the bridle main branch while stil making their
> own worktrees.

## What it means for onboarding

- **A standing rule**: nothing changes in one of the human's existing projects without
  their clear review and approval. That covers the project's files, branches, remotes and
  settings, not just code.
- **Every onboarding runs as a trial on its own branch.** The project's `main` (and
  `dev`) are never touched. The migration commit (`.bridle/`, CLAUDE.md, retired
  OpenSpec skills and so on) and every trial change land on one trial integration branch.
  Workers still get their own worktrees and `bridle/<agent>` branches, but branch from and
  merge into the trial branch. The state branch (`bridle/state`) is already separate.
- **Adoption is the human's call**: they review the trial branch and approve replacing
  `main` (and `dev`) with it. Until then, bridle never merges into or pushes them.
- **Needs [[branch-rules-per-project-rxe8|rxe8]] first**: the integration branch the
  manager and workers target must be a project setting. During a trial it is the trial
  branch; after adoption, `main` or `dev` per rxe8's two patterns.

## The human's answers

2026-09-28, on the name, how the branch starts, and pushing:

> yes, bridle-adopt is a better name, I expect adoption to WORK! :) right the orchestrator
> will clone main/dev to bridle-adopt and that is the target for all merged and work until
> I personally approve to replace main/dev.
> Yes, push branch to github.

So:

- The trial branch is **`bridle-adopt`** (outside the `bridle/<agent>` namespace, so no clash
  with agent names).
- The orchestrator creates it from the project's `main` (or `dev`, where that is the
  working branch).
- It is the target for all merges and work until the human personally approves replacing
  `main`/`dev` with it.
- It is pushed to GitHub.

## Notes

- **Naming.** Agent branches are `bridle/<agent>`, and the state branch is
  `bridle/state`. A trial branch named `bridle/main` would clash with an agent named
  `main`. The trial branch needs a name agents can't take (settled: `bridle-adopt`, below), or
  agent-name validation should reserve it. The same clash exists today for an agent named
  `state` (`validate_agent_name` in `crates/bridle-daemon/src/worktree.rs` doesn't
  reserve it), worth checking.
- **Where the manager merges.** The manager merges in the clone, which the human also
  uses. During a trial the clone would need the trial branch checked out, or the manager
  merges somewhere else (the track-web survey's point 6 in
  [[onboarding-survey-track-web-and-harness-u8sm|u8sm]]).
- **Pushing.** Whether the trial branch is pushed to the project's remote is the human's
  call; pushing only it doesn't touch `main` or `dev`.
- The data-contracts onboarding plan in `docs/context/orchestrator-state.md` (step 3 on a
  branch, then a first real task) predates this and should follow it.

## Resolution

Resolved by 61ce8ed (the `existing-projects` rule: no change to the human's projects without review), 7ffae38 (the human's answers: `bridle-adopt`, pushed) and 0c84f5d (br-29f9, [[branch-rules-per-project-rxe8|rxe8]]: `[branches] integration = "bridle-adopt"`). Onboardings now run on `bridle-adopt` branches. The answer lives in docs/design/agent-host/operating-model.md ("Trial onboarding").
