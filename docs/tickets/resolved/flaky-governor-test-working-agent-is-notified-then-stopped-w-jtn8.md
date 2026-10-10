---
id: jtn8
title: Flaky governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out under machine load
kind: bug
opened: 2026-10-10
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [n96z]
tasks: []
closed: 2026-10-10T04:43:53Z
---

## The ask

## What happened

whdbj, building br-hdbj (2026-10-10 ~04:40Z), hit `governor_test::working_agent_is_notified_then_stopped_when_its_turn_ends` timing out twice in full `just check` runs while the machine's load was 30-40 (16 cores). It passed alone, and the final check passed only with `NEXTEST_RETRIES=2`. Not seen on CI so far; `main` is green.

Ticket n6gy (resolved) fixed other time-based tests of the same kind and cited this test as the model for its wait; this one still uses a fixed timeout too tight for a loaded machine.

## The ask

Make the test wait on the event it checks (or a load-tolerant bound), as n6gy did for its tests, so workers don't need retries to pass `just check`.

## Cost of not doing it

Workers re-run `just check` (30-40 min of machine time each) or pass with retries, which hides real failures; if it starts failing on CI it becomes critical (red `main`).

## Resolution

Duplicate of [[flaky-under-load-governor-test-working-agent-is-notified-the-n96z|n96z]], filed by
advisor (product-manager) minutes earlier (2026-10-10 ~12:43 AM ET) for the same test, with task
br-n96z (planned). The n6gy pointer and the other two load-sensitive tests named here (cli_e2e
sigint, events_stream shutdown) were added to br-n96z's thread. Resolved 2026-10-10 by advisor
(product-manager).
