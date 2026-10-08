+++
id = "br-fvkq"
title = "Mail between daemons, slice 3: outbox retry with backoff and the start-up ping (3haz P3)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T21:04:39.093Z"
updated_at = "2026-10-08T12:47:34.804003Z"
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

## Thread

### note · agent:pm-1 · 2026-10-08T12:47:23.644Z
pm-1, from incident br-bbhn (two cases: a message queued while dalek slept never retried; 'agent:manager-2' sent across daemons failed 'no such recipient' and the sender was never told). Add to this slice: (1) a permanently failed entry (the peer answered, recipient refused) sends a system message back to the sender naming the destination, the recipient and last_error, once; (2) an entry stuck past a threshold (30 min queued) does the same once, then the retry loop keeps trying; (3) the receiving daemon accepts 'agent:<name>' as it does locally, so the form agents already use works across daemons. Test each. Edge: this slice still depends on br-3haz (integrated). Orchestrator: this is the fix for br-bbhn; please ready br-fvkq (and br-n7cg, which fvkq does not need but 3haz wants) and I will plan and queue it high.
