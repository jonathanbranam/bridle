---
id: br-p4f2
title: bridle migrate 0006 refuses every project that works on main (protected-branch check ignores [branches] integration), even with nothing to change
kind: bug
created: 2026-10-16
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-rjd5]
resolved: 2026-10-17T11:37:09Z
---

## The ask

Reported by the meta-notes aide on vesta (m-13894, 2026-10-15 ~9:50 PM ET), from the human's hand-run of vesta-home setup (docs/context/add-a-machine.md sec. 6). The human, as quoted by that aide: "I want to run them and see it work or fail".

What happened: `bridle migrate`'s 0006-backfill-ticket-kind fails on every project that works directly on main. `refuse_on_protected_branch` (crates/bridle/src/migrate.rs ~734) refuses `main`/`dev` (and the release branch) for any project but bridle, without looking at `[branches] integration`. Seen on vesta-home (no tickets at all, so nothing to change) and on meta-notes (dry run: "main is a protected branch of meta-notes"). Likely also dotfiles-local.

Ask: allow the project's own integration branch (`[branches] integration`), and don't refuse at all when the migration has nothing to change. Check the other migrations that call the same guard.
