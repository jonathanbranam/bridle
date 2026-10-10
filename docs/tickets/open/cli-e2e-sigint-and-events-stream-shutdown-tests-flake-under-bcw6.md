---
id: bcw6
title: cli_e2e sigint and events_stream shutdown tests flake under load
kind: bug
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

Two tests fail under machine load: `cli_e2e` sigint and `events_stream` shutdown. Manager-2 saw both
on br-fpde's landing checks, 2026-10-09 night, at load 30-40. They are a different kind from the
governor_test flake fixed by br-n96z. The worker on br-n96z says they put wall-clock limits on how
long shutdown takes, rather than waiting for an event (br-n96z thread, 2026-10-10 12:08Z).

Fix the cause; don't weaken the tests. If a shutdown limit is the thing the test is about, measure
it some other way under load (as ticket n6gy did for event waits) or explain on the task why the
limit must stay.

## Cost of not doing it

A landing check can fail on a change it has nothing to do with, so the change waits and a manager
reruns the check or lands it with NEXTEST_RETRIES. That happened twice on 2026-10-09/10 and held
up machine setup, the top item on the roadmap.
