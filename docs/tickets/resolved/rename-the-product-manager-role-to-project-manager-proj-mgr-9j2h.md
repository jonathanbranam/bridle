---
id: 9j2h
title: Rename the product-manager role to project-manager (proj-mgr), with a reviewed migration for existing projects
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [7r2c, xebc]
tasks: [br-9j2h]
closed: 2026-10-09T23:11:01Z
---

## The ask


Build ticket for [[the-product-manager-role-is-really-a-project-manager-who-hel-7r2c|7r2c]], decided
2026-10-02. The human, verbatim: "Yeah let's rename it to project manager can be shortened to proj-mgr
if needed in naming."

Its question task br-8b6c was dropped in the k7tm sort (2026-10-03) as "not approved for work now",
but the rename was approved; the workflow advisor caught it (m-4387, 2026-10-04).

Wanted:

1. Rename the role `product-manager` to `project-manager` (`proj-mgr` where a short name is needed):
   the role file under `workflow/base/roles/`, config keys and role names in code, the PM-or-human
   queue gate (`require_pm_or_human`), CLI text, docs and `.bridle/` in this repo. Agent names like
   `pm-1` may stay.
2. A project migration (the mechanism in
   [[project-migrations-one-command-applies-pending-bridle-upgrad-xebc|xebc]]) that renames the role
   in an existing project's config and role overrides.
3. The existing projects (meta-notes, track-web) are the human's: the migration is shown to the human
   and runs there only after they approve (`workflow/base/rules/existing-projects.md`). Bridle's own
   repo is migrated in the build.

Out of scope: the product-partner work (option 2 in 7r2c), which waits on sk52; renaming the advisor.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
