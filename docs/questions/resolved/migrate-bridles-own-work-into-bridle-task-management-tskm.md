---
id: tskm
title: Migrate bridle's own work into bridle task management (or close to it)
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

Filed as research. The human's words, verbatim:

> All the open work should be able to go into Bridle as is without forcing a
> decision on everything. We can address open questions and spikes after
> bootstrapping and using bridle itself for work.

## Why it matters

Once P0 (tasks, edges, `ready`, claims, state-branch persistence, `rebuild`)
lands, bridle should take over its own task management: every open ticket in
`docs/questions/open` and `docs/spikes/open`, the live work queue, and
build-order's P1-onward items all need to become tasks in bridle itself,
imported as-is with no triage forced on anything. This is planned as P0-6 in
the build order, after P0-4 and P0-5.

## Resolution

Completed 2026-09-28. P0-6 was executed in two phases: P0-6a verified the ticket importer (73f6604), and P0-6b did the real migration (ecae5fe: "real tskm import (68 tasks)"). All 58 open tickets and 10 build-order items were imported into bridle task management as-is, without forced decisions, fulfilling the ticket's requirement that "all the open work should be able to go into Bridle as is".
