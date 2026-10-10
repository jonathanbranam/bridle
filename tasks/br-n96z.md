+++
id = "br-n96z"
title = "Flaky under load: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out"
kind = "bug"
state = "planned"
created_at = "2026-10-10T04:43:08.957Z"
updated_at = "2026-10-10T04:43:53.634494Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "n96z"
+++

Ticket: docs/tickets/open/flaky-under-load-governor-test-working-agent-is-notified-the-n96z.md (read it for the evidence). Goal: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends (crates/bridle-daemon/tests/governor_test.rs) passes reliably under load (three workers' just check at once, load 25-45 on 16 cores) without weakening what it checks. First find why a loaded machine needs more than its 70 s deadline (what it waits on: a poll interval, the governor tick, a fake-claude spawn through the pyenv shim?). Then make the wait event-driven or give the deadline a load-scaled bound, as br-6nzj and br-8ff8 did for their flakes (read their threads/commits for the idiom). Do not just raise the number without the cause. Acceptance: just check passes; run the test 20x alone and 10x while a cargo build or another nextest run loads the machine, report the counts. Docs: CHANGELOG only if user-visible (it isn't). Migration: none. Model: Sonnet. Out of scope: other flakes (br-4vmc), the fake-claude interpreter lookup (br-yw8b).

## Thread

### note · external:advisor/product-manager · 2026-10-10T04:43:53.634Z
advisor (product-manager): the orchestrator filed the same test as ticket jtn8 (resolved as a duplicate of this one). From it: ticket n6gy (resolved) fixed tests of this kind by waiting on the event, and cited this test as its model; this one still has a fixed timeout. manager-2 counts two more load-sensitive tests seen tonight on br-fpde: cli_e2e sigint and events_stream shutdown. Look at those too, with the same fix, if they are the same kind; otherwise say so and I'll file them.
