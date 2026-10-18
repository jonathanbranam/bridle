+++
id = "br-jqf9"
title = "kx5u trial 2 findings: idle manager blocks the drain, migration leaves checkout dirty, noisy and contradictory move output"
kind = "fix"
state = "planned"
created_at = "2026-10-18T23:42:19.316Z"
updated_at = "2026-10-18T23:58:11.563117Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "jqf9"
+++

docs/tickets/open/kx5u-trial-2-findings-idle-manager-blocks-the-drain-migratio-jqf9.md

## Thread

### note · external:advisor/product-manager · 2026-10-18T23:42:28.616Z
Readied by advisor (product-manager): fixes from kx5u trial 2.

### note · agent:pm-1 · 2026-10-18T23:43:12.493Z
Brief (pm-1). Right-sized: items 3, 4 and 5 of the ask moved to br-ec5m (output polish). THIS ticket is items 1 and 2 only. Model: Sonnet.
1. Idle agents block the drain: build the Proposal (the drain stops each agent once it is idle with no claimed or built work, after its announce turn); `project stop` must not then refuse on an agent the drain could have stopped. Look in crates/bridle (grep project drain, project stop).
2. Migrations leave the checkout dirty: make migrations commit what they change (0007-gitignore-agent-site changed .gitignore and .bridle/migrations.*), or, if a migration cannot commit, have the drain name the files as "uncommitted bridle migration: run <command>". Prefer the commit. Check the other migrations for the same leftovers. Per ticket xebc, existing projects are brought up to date by the migration mechanism, so say in the change how projects already left dirty get fixed (the drain message names the command).
Update docs/context/move-a-project.md (remove any manual-stop step). Tests: a drain with an idle manager completes; a migration leaves a clean tree. Acceptance: just check. Out of scope: items 3-5 (br-ec5m), preflight (br-eavq).

### note · agent:pm-1 · 2026-10-18T23:47:23.098Z
pm-1: item 2 (migrations leave the tree dirty) is superseded by br-rdag items 1 and 3, which fix it properly. THIS ticket is now item 1 only (idle agents blocking the drain); skip the migration change. The drain may still name uncommitted bridle migration files with the fix command if cheap, otherwise leave it.
