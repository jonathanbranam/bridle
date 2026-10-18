---
id: br-rdag
title: "Migrations: automatic runs leave their changes uncommitted, opt-in ones are never run, and the branch guard misfires"
kind: bug
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Fix three things in project migrations (docs/design/migrations.md, crates/bridle/src/migrate.rs), and
keep the doc in step:

1. **Commit what an automatic run changes.** When `bridle serve` applies migrations at start-up, commit
   only the files that migration changed, plus `.bridle/migrations.toml` and `.bridle/migrations.log`.
   Use a commit message that names the migration ids. If it can't commit (for example the branch is
   protected or other files are dirty), file a to-do for the project's manager saying what is left and how
   to commit it. Also catch up what earlier runs left: if the migrations files differ from git at start,
   commit them the same way (or file the to-do).
2. **Opt-in migrations get done or retired.** For each `manual_only` migration (`0001` to `0005` today),
   decide whether existing projects still need it. Some may. Retire any that aren't needed (record that
   in the doc). For the ones that are, make a pending opt-in migration file a to-do on the project at
   daemon start, so it isn't silently left as pending-manual forever.
3. **The protected-branch guard misfires.** On dalek, `0007-gitignore-agent-site` edited `.gitignore` in
   dalek-home while its checkout is on `main` (2026-10-17 09:24Z). The guard should refuse there. The
   human was also told that track-web misbehaved because its checkout is on `dev` (its `[branches]
   integration = "dev"`), not `main`. Find out what the guard did on each and fix it so the guard follows
   the project's configured branches.

## What aide found (2026-10-18 ~7:30 PM ET, dalek)

- track-web (checkout on `dev`): `.bridle/migrations.log`, `.bridle/migrations.toml` and `.gitignore`
  modified since the 2026-10-17 09:24Z auto-run, never committed.
- dalek-home (on `main`): `.gitignore` modified. `.bridle/migrations.log` and `.toml` untracked.
- bridle-ui (on `main`): 9 ticket files with `tasks:` added, uncommitted. The cause isn't confirmed.
- All four of track-web, data-contracts, bridle-ui and dalek-home list `0001` to `0005` as
  pending-manual. bridle's own repo ran them by hand.
- The design says "Commit the changed files and `.bridle/migrations.*` afterward". At start-up nobody is
  told to, so the edits sit in the checkout.

## The human's words

2026-10-18 ~7:25 PM ET, to aide:

> 1) has been the case on several projects I have noticed; why didn't the earlier migrations run?

Then on aide's proposals (1: commit after an auto-run, else a to-do; 2: decide each opt-in migration,
retire or surface it):

> 1. yes it should
> 2. agree; some may be needed; i was told traack-web didn't behave properly b/c it was on dev not main that messed something up
