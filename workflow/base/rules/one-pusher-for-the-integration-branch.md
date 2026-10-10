---
id: one-pusher-for-the-integration-branch
severity: must
roles: [orchestrator, project-manager, manager, worker, reviewer, document-reviewer, advisor, aide, prototyper, designer]
---
Only the owner's clone pushes a project's integration branch to origin. The owner is the machine
named in `owner.toml` on `bridle/state`; `bridle serve --take-over` moves it.

- **Enforced, not just written**: a `pre-push` hook bridle installs in each clone refuses the push
  elsewhere, naming the owner. Other branches and tags are not affected.
- **Never bypass it**: no `git push --no-verify`, no `core.hooksPath` edits (denied to agents).
  If a push is refused, the clone is not the owner: report it, don't work around it.

Why: the human, 2026-10-09 (postmortem j7r4, ticket 8z7j): "it should be impossible for a different
clone under bridle to push."
