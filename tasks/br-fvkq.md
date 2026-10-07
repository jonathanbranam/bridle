+++
id = "br-fvkq"
title = "Mail between daemons, slice 3: outbox retry with backoff and the start-up ping (3haz P3)"
kind = "feature"
state = "pending"
created_at = "2026-10-05T21:04:39.093Z"
updated_at = "2026-10-06T00:45:06.044358Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "3haz"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md (slice 3 of 6, P3 and P8 step 3). Depends on br-3haz (slice 1).
Goal: nothing is lost when the other daemon is down. The outbox retries with backoff (at once, 30 s, 2 m, then every 5 m, capped) and messages never expire. Each daemon on start-up pings its peers from the machine config; a peer that hears "I'm back" flushes its outbox to it at once. Per-peer order stays oldest first; dedup (slice 1) makes retries safe.
Files likely: crates/bridle-daemon (outbox module, start-up, server), docs/design/agent-host/daemon.md.
Migration: none (behaviour of the new outbox only).
Acceptance: just check passes; tests with a fake clock: a down peer gets retries on the schedule, a restarted peer's ping flushes at once, no duplicates, order kept. Use tokio paused time, not real sleeps.
Model: Sonnet. Out of scope: status lines and the human report (slice 4).
