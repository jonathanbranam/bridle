---
id: rxe8
title: Branch rules per project
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [0fa0fb8, 0c84f5d]
specs: []
needs: []
see: [u8sm, m2fq, 63rv]
closed: 2026-09-30T05:12:44Z
---

## The ask

The human, verbatim (2026-09-28):

> File a ticket to enhance bridle to support strong rules for branch usage. Some projects
> work directly on main, others use a dev branch. KISS, but merging to main for a release is
> a common enough pattern. don't add complexity for other approaches.

## Why now

The track-web survey ([[onboarding-survey-track-web-and-harness-u8sm|u8sm]]): in track-web,
day-to-day work is on `dev`, and a push to `main` deploys to production through a GitHub
webhook. Bridle assumes `main` everywhere:

- `.bridle/roles/manager.md`: `git log main..bridle/<name>`, merge into `main`,
  `merge-base --is-ancestor main bridle/<name>`, `git push origin main`, "Push anything
  but `main`" under Never.
- `workflow/base/skills/worker/SKILL.md`: "merge the local `main` into your branch
  (`git merge --no-ff main` ...)".
- `docs/design/agent-host/operating-model.md`: "Only the merger pushes, and only `main` and
  release tags"; releases are a commit and tag on `main`.

The daemon's worktree base is already configurable (`base` in `.bridle/config.toml`,
`crates/bridle-daemon/src/config.rs`, default `HEAD`), but nothing ties the roles and
skills to it.

## Scope (the human's: KISS)

Two patterns only:

1. **Trunk**: work merges into `main`; releases are tags on `main` (bridle today).
2. **Dev + release**: work merges into `dev` (the integration branch); `main` only
   moves when `dev` is merged into it for a release.

Nothing else (release branches, git-flow, per-feature integration branches).

## Notes

- A sketch, not a design: one project setting naming the integration branch (and, for
  pattern 2, the release branch), used by the worktree base, the manager's merge and push,
  and the worker's handoff merge; and a strong rule (locked) that agents push only the
  integration branch and never merge into or push the release branch except as the release
  step, which stays with whoever cuts releases today (the orchestrator, or the human).
- "Strong" should mean enforced where bridle can, not only written in a prompt; how is
  open.
- Related: [[merging-main-fails-under-merge-ff-only-m2fq|m2fq]] (`git merge main` under
  the human's global `merge.ff = only`).
- **Needed first by onboarding**: the human wants every new project trialled on its own
  branch, with `main` and `dev` left alone until they approve adoption
  ([[trial-adoption-on-a-bridle-branch-63rv|63rv]]). That's the same setting (the integration
  branch) pointed at a trial branch, and the same rule (never merge into or push `main` or
  `dev`).

## Resolution

Resolved by 0c84f5d (br-29f9, 0fa0fb8): the `[branches]` project setting names the integration branch and, for the dev + release pattern, the release branch; worktrees, the manager's merge and the worker's handoff use it, and `disallowed_tools` keeps ordinary agents off the release branch. The answer lives in docs/design/agent-host/operating-model.md ("Branch pattern") and docs/design/agent-host/roles-and-config.md. Trial onboardings (`bridle-adopt`, [[trial-adoption-on-a-bridle-branch-63rv|63rv]]) run on it.
