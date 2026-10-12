---
id: br-d6h6
title: "K9: return trip: back up the old database in the workspace and carry the outbox counter"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-kt25]
parent: br-kx5u
resolved: 2026-10-12T18:07:51Z
---

Child of epic br-kx5u (MVP), from the design in br-kt25 ("## Design", child K9), as settled in the human's grilling session 2026-10-12 (Q4, verbatim on br-kt25's thread).

The human (~10:31 AM ET), verbatim: "We have a project workspace folder, so let's put a folder in there for backing up old databases. ... I approve that approach: backup the old database. We can always query it in the future if we need to. It can live on local disk and does not need to be in GitHub." And (~10:36 AM ET): "if it's easiest, just copy the counter over. Perfect, do that."

## The ask

When a project starts on a machine that held it before (a return trip), the old `<workspace>/.bridle/bridle.db` is moved (with its `-wal`/`-shm`) into `<workspace>/.bridle/db-backups/<date>-<sha>/`, not deleted and not in git, and a fresh database is rebuilt from the state branch. The outbox counter is carried over: `project stop` records the outbox high-water mark (for example in the `released` commit), and start seeds the fresh database's counter above it, so receivers (who dedup on machine, daemon and id) don't drop new mail as duplicates.

## Acceptance

A test move OLD -> NEW -> OLD: OLD's old database is in `db-backups/`; the new one has NEW's tickets and state; OLD's next outbox id is above the last one it sent before; a receiver accepts it; `just check` passes.
