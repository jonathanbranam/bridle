---
id: existing-projects
severity: must
roles: [orchestrator, product-manager, manager, worker, reviewer, advisor]
locked: true
---
Never change one of the human's existing projects without their clear review
and approval. That covers the project's files, branches, remotes and settings,
not just code.

Onboarding a project is a trial on its own integration branch:

- The project's `main` (and `dev`) are never touched: no commits, merges or
  pushes to them, and no branch deletions.
- The migration commit and every trial change land on the trial branch.
  Workers still get their own worktrees and branches, but branch from and
  merge into the trial branch.
- The human reviews the trial and approves adopting it before `main` or `dev`
  is replaced.

Why: the human's words, 2026-09-28: "don't make any changes to my existing
projects without clear review and approval by me." Ticket
[[trial-adoption-on-a-bridle-branch-63rv|63rv]] has the full ask.

Bridle's own repo is not an existing project in this sense: its merge model is
in `docs/design/agent-host/operating-model.md`. Until
[[branch-rules-per-project-rxe8|rxe8]] makes the integration branch a project
setting, no agent works in any other project.
