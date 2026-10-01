---
id: xebc
title: "Project migrations: one command applies pending bridle upgrades to a project, tracked and logged"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk, where-the-knowledge-root-lives-and-migrating-to-it-ew97]
---

## The ask


The human, verbatim (2026-10-01, via the advisor), on how ticket v3dk's `kind:` backfill would
reach existing projects:

> One question is, we now have multiple projects using Bridal, so we're starting to need to handle
> upgrades. So one of the things in this ticket is that tickets now have a kind that needs to be
> backfilled. So how's that going to be handled? And let's add this to the project manager. Sorry,
> the product manager to when reviewing a task and ticket to ensure that if it includes changes
> that there's a migration plan and you know prefer an automatic solution in many cases so that we
> can update all of our projects at the same time or update all of our projects easily. So we, we
> don't have anything like this yet. This may need to be a totally separate ticket and task to
> implement some way of running a command in Bridal that applies a set of updates and changes that
> need to be done since the last time it was run and keeping track of those as well as providing
> some output, some logs or events or some kind of output to that the user can inspect and audit
> in case something changes that surprises them that the agents can read as well.

## Today

- The daemon's SQLite database migrates itself (`PRAGMA user_version`,
  [[docs/design/storage|storage]]).
- `bridle workflow update` re-fetches the workflow vendored into `.bridle/workflow/`, and
  `bridle sync` re-renders it. Neither changes a project's own files.
- Nothing upgrades what a project keeps in its repo for bridle (ticket frontmatter,
  `.bridle/config.toml`, docs layout). A change like v3dk's `kind:` backfill is a hand-run script
  per project.

## The ask

1. A bridle command that applies, to one project, every migration shipped since it last ran.
2. It records which migrations have run in that project, so each runs once and the next run
   picks up only what's new.
3. It leaves output the human can inspect and audit when a change surprises them, and that agents
   can read: logs, events, or another record.
4. Applying it to every project should be easy, ideally all at once.

The [[tickets-get-a-kind-kinds-are-editable-tickets-and-tasks-link-v3dk|ticket kinds]] backfill
is the first migration it would carry.

## Product-manager review (handed to the orchestrator, 2026-10-01)

Separately, the product manager's review of a task or ticket should check that a change to
projects' files comes with a migration plan, preferring an automatic one so every project can be
updated easily. That's a role-prompt change (`workflow/base/roles/product-manager.md`), sent to the
orchestrator rather than made here.
