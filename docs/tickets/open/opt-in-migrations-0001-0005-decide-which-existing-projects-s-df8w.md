---
id: br-df8w
title: "Opt-in migrations 0001-0005: decide which existing projects still need them, retire the rest, surface pending ones as to-dos (from rdag item 2)"
kind: fix
created: 2026-10-18
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-rdag]
related: [br-rdag]
---

## The ask

Split from br-rdag (its item 2; read that ticket for the evidence and the human's words).

For each `manual_only` migration (0001 to 0005, docs/design/migrations.md, crates/bridle/src/migrate.rs): decide whether existing projects still need it (track-web, data-contracts, bridle-ui and dalek-home list all five as pending-manual; bridle's own repo ran them by hand). Retire any that are not needed and record that in the doc. For those that are, make a pending opt-in migration file a to-do for the human on the project at daemon start (once, not on every start), so it is not left pending-manual forever.

Model: Sonnet. Run AFTER br-rdag (same files). Acceptance: just check; a test that a pending opt-in migration files one to-do and not a second at the next start; the doc says which migrations were retired and why. Out of scope: the commit-after-run and branch-guard fixes (br-rdag). Changes what projects keep: per ticket xebc say how existing projects get updated (the retirement must leave their migrations.toml consistent).
