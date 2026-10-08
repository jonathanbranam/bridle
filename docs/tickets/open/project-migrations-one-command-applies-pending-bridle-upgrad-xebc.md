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

## Automatic by default (the human, 2026-10-01)

The human, verbatim (via the advisor; "Bridal" is bridle, "the Nook" is the NUC):

> I'm not sure where we landed on the migration question, but I want to see that applied
> automatically. I mean, maybe a project could opt in or out with configuration, but I think the
> default is, I know we haven't shipped the change to upgrade Bridal automatically, and there's
> some other things we haven't shipped yet, but my expectation is because I own all these projects
> right now and I'm comfortable with the changes I'm making to Bridal along the way, that
> everything gets upgraded always. So if I install, you know, once we have this kind of
> installation upgrade thing working, like I want migration to run on every every registered
> project on Dalek, and then I want migration to run on the Nook as soon as the bridal is upgraded
> on the Nook. Every project, every registered project should be upgraded. And if a daemon is down,
> if that's a problem, if the daemon's down for a specific project, when the upgrade happens, then
> when that daemon comes up, bridal serve happens again, the upgrades should all happen
> automatically. The only time this shouldn't be the case is if we should be able to mark a
> specific upgrade as like dangerous or opt in, and then that might be a manual or manual, you
> know, maybe just the manual. Is the right thing. That would be something people opt into, but
> for now, we want all the upgrading to happen all the time, automatically.

Decided:

1. **Migrations run automatically**, with no command needed. When bridle is installed or upgraded
   on a machine (dalek, the NUC), every project registered on that machine gets its pending
   migrations.
2. **A project whose daemon is down catches up when it next starts.** `bridle serve` applies
   whatever is pending, so starting the daemon always applies pending migrations, not just
   upgrading bridle.
3. **A project may opt out in config.** The default is on.
4. **A single migration can be marked opt-in** (for example, a dangerous one). Opt-in migrations
   never run automatically. They run only when someone runs them by hand.
5. Recording what ran, plus auditable output that agents can read (asks 2 and 3 above), still
   applies to every automatic run.

How bridle installs and upgrades itself on each machine is separate work. The daemon's
self-upgrade (`bridle restart --upgrade`, [[docs/design/agent-host/daemon|daemon]], "Upgrade")
covers one project's daemon. Running migrations when `bridle serve` starts covers both paths.

## Built: automatic at start-up (br-2718, on a branch, parked for review)

`bridle serve` runs pending migrations before listening (`migrate::run_at_startup`, wrapped against errors and panics); `[migrations] auto = false` opts out; `manual_only` migrations run only by `bridle migrate --only ID`; failures file an incident, refusals are skipped. See docs/design/migrations.md, "At start-up".

## Product-manager review (handed to the orchestrator, 2026-10-01)

Separately, the product manager's review of a task or ticket should check that a change to
projects' files comes with a migration plan, preferring an automatic one so every project can be
updated easily. That's a role-prompt change (`workflow/base/roles/product-manager.md`), sent to the
orchestrator rather than made here.
