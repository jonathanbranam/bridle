---
id: br-mm4z
title: Migration 0005-a-ticket-for-every-active-task aborts on dead task rows instead of skipping them
kind: bug
created: 2026-11-11
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Migration `0005-a-ticket-for-every-active-task` (crates/bridle/src/migrate.rs) aborts on task rows the daemon no longer has, instead of skipping them. The dry run and the real run disagree.

Seen on data-contracts, 2026-11-11 (ticket dc-swuz, worker agent:adopt, filed by aide): `bridle migrate --only 0005-a-ticket-for-every-active-task --dry-run` listed 7 stubs (dc-vueh, dc-ne9h, dc-7qe9, dc-p4ef, dc-hwju, dc-xrfv, dc-s5jm). The real run aborted with "no such task" on dc-vueh: six of the rows are dead (`bridle task show` gives not_found; `bridle task list` shows only dc-s5jm).

## Expected

- The migration skips, and reports, task ids the daemon does not have, and carries on with the rest.
- The dry run lists only the tasks the real run would stub, so the two agree.

## Acceptance

- A test with a mix of live and dead task ids: the real run stubs the live ones, reports the dead ones as skipped, exits 0; the dry run lists the same live set.
- `just check` passes.

## Model and scope

Sonnet. Touches migrate.rs (and its tests) only. Out of scope: finding why the dead rows are listed at all (add a line to the thread if the worker sees the cause), other migrations.
