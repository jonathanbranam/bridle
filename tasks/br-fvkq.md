+++
id = "br-fvkq"
title = "Mail between daemons, slice 3: outbox retry with backoff and the start-up ping (3haz P3)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T21:04:39.093Z"
updated_at = "2026-10-08T13:54:58.453726Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
summary = "Slice 3 of 3haz. outbox.rs: a 15 s tick retries each destination's head message on the backoff (first try at once, then 30 s, 2 m, 5 m; in-memory next-try, never expires); POST /v1/hello (peer token) makes the hearer flush its queue for the greeter, sent to [projects] peers at start-up and after a wall-clock jump over 60 s (sleep). The sender gets a system note once on a permanent refusal and once at 30 min queued (new column stuck_notified_at, SCHEMA_V22; the ticket said no migration but a once-only notice survives restarts only with one). POST /v1/outbox waits up to 3 s for the first try; Queued gains state and last_error (serde default); the CLI prints the outcome and exits 1 on a refusal. resolve_targets accepts agent:<name>. Tests: unit tests on tokio paused time with a fake Transport/Notifier/wall clock (schedule, ping flush in order with no duplicates, sleep, refusal notice, stuck notice) plus integration tests. tokio test-util added as a dev-dependency. Docs: daemon.md, principals.md, api.md, storage.md, cli.md, CHANGELOG."
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

### note · external:orchestrator · 2026-10-08T12:49:10.568Z
Orchestrator, two additions (checked against outbox.rs: today flush() runs only from send_outbox, so a transient failure waits for the next send):
(4) Sleep, not just restart. In br-bbhn the receiver (dalek, a laptop) slept; its daemon never restarted, so a start-up ping alone never fires and the NUC waits up to the 5 m backoff. Treat a resume from sleep like a start-up: a daemon that sees the wall clock jump well past its monotonic tick (e.g. >60 s gap on a 15 s tick) pings its peers and flushes its own outbox. Test with an injected clock.
(5) Tell the sender at send time. POST /v1/outbox waits a short time (about 3 s, under ATTEMPT_TIMEOUT) for the first try and returns its result: delivered, failed (with the reason) or queued (with last_error, retrying). The CLI prints that instead of a bare 'queued o-NNNN', so 'no such recipient: agent:manager-2' shows at once. Wire change: add the state and last_error to Queued in bridle-api types.rs (serde default, so older CLIs still parse), and update daemon and CLI together; principals.md 'Mail between daemons' and cli.md updated to match.
pm-1's (1)-(3) stand. The 30 min stuck notice (2) goes to the sender only; the human report stays in slice 4 (br-cufw).

### note · agent:manager-2 · 2026-10-08T12:49:21.387Z
manager-2: re-read the whole thread of br-fvkq (bridle task show br-fvkq): the orchestrator added two requirements (resume from sleep counts as a start-up ping; the send reports the first try's result, a bridle-api types.rs change, so update all clients). Cover them with tests too.

### note · agent:outboxretry · 2026-10-08T13:46:35.221Z
done: outbox retry (backoff 0/30s/2m/5m), /v1/hello greeting at start-up and after sleep, sender notices for refused/stuck-30min, agent:<name> accepted, send reports first try; just check exit 0, 1330 tests passed; 1fd1

### note · agent:outboxretry · 2026-10-08T13:46:43.961Z
correction: the commit the green check ran on is d2414b68 (ignore '1fd1' above)

### note · agent:manager-2 · 2026-10-08T13:54:58.453Z
manager-2: main has moved (br-gdyy, br-hc6j landed). Merge main into your branch, run just check, report the new tip sha and result. The diff itself looks fine.
