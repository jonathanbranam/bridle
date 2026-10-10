+++
id = "br-9aj2"
title = "Message delivery you can check, part 1: bridle messages command, delivery audit events with the channel, role files (9aj2 items 2, 3, 5)"
kind = "feature"
state = "planned"
created_at = "2026-10-10T13:38:39.686Z"
updated_at = "2026-10-10T20:01:11.945377Z"
created_by = "external:aide"
watchers = ["external:aide"]
summary = "Part 1 of 9aj2. New `bridle messages [--for P] [--last N=5] [--since DUR]` (misc.rs): default the caller's own with bodies; any other principal (role, external:, agent, human) headers only (a CLI display rule, not a boundary; GET /v1/messages still returns bodies; stated in cli.md). Shows sent/delivered/read times and the channel of each. Wire: MessageQuery.since_secs, EventQuery.message and .to (the latter matched through the messages table, so old events match), PrincipalWakeQuery.pid, `channel` constants in types.rs; all serde-default. Audit: message.sent/delivered/read carry `channel` (waiter with pid+session, inbox, ui, agent, api); wake-taken reads previously emitted no message.read at all, now they do. `bridle events --message/--to` (not applied under --follow). Mail and schedule sends are not yet told apart from ui/api (noted in api.md). Role files (aide, advisor, orchestrator) say how to check for missed messages. Migration: no schema change; event fields additive; role text reaches projects via workflow sync. Not built: part 2 (br-3zhx)."
ticket = "9aj2"
+++

Ticket: docs/tickets/open/message-delivery-you-can-check-a-message-stays-unread-until-9aj2.md (read it, esp. Facts and the human's verbatim ask). Also docs/design/agent-host/principals.md and api.md. This is PART 1; part 2 (item 1: stays unread until seen; item 4 feasibility) is a separate task blocked by this one, do not build it. Build: (2) bridle messages [--for PRINCIPAL] [--last N] [--since DURATION] [--json]: default the caller's own (from role and name); any agent may run it for any agent or human role (read-only). Shows id, from, sent/delivered/read times and the channel of each. Bodies only for one's own messages; for others, headers only (decision, state it in cli.md). Add a filtered route under /v1/messages (to, last, since) and the client method; wire types in crates/bridle-api/src/types.rs, daemon and CLI together. (3) Audit trail: message.sent, message.delivered and message.read events gain an optional channel field (waiter with pid and session id, inbox, ui, mail, schedule, api) plus who and when; bridle events can filter by --message ID and --to PRINCIPAL (and the HTTP events query), not just a recent window. New fields are serde-default so old events and clients still parse. (5) Update the Waiting for messages sections of workflow/base/roles (grep) with how to check for missed messages (bridle messages --last 5). Files: crates/bridle-daemon (messages and events store, server), crates/bridle-api/src/types.rs, crates/bridle/src (messages, events commands), workflow/base/roles/*, docs cli.md api.md principals.md storage.md if the schema changes, CHANGELOG. Migration: event fields are additive; any schema change goes through the daemon normal migration; role text reaches projects via workflow sync; no project files change. State this in the done note. Acceptance: just check passes; tests: default scope and --for, headers only for others, --since and --last, events filter by message id and recipient, channel recorded for waiter and inbox reads, old events without channel still listed. Model: Sonnet. Out of scope: changing when a message is marked read (part 2), refusing background waiters (item 4).

## Thread

### note · external:advisor/product-manager · 2026-10-10T13:39:13.873Z
advisor/product-manager (PdM): readied, normal; placed at the head of the messaging epic (rank 2), after br-37r9 and br-bcw6: a lost message is a correctness bug and the human raised it today. Planner: split it. Build first what has no open question: item 2 (bridle messages), item 3 (audit events with the channel; events filter by message id and recipient), item 5 (role files). Item 1 (unread until the session has seen it): take the ticket's recommendation (marked read when the same session next runs wake or inbox); for the open points, PdM default is: offered again on the next wait from any session of that principal, no timeout, sender not told; ask on the ticket if that doesn't fit. Item 4 (refuse '&' or discarded output): check feasibility first (a Claude Code background command has no tty either); with item 1 built, a lost waiter loses nothing, so drop item 4 if it can't be detected reliably and say so.

### note · agent:pm-1 · 2026-10-10T13:39:37.384Z
split off br-djru: Message delivery you can check, part 2: a message stays unread until the session has seen it; feasibility of refusing background waiters (9aj2 items 1, 4)

### note · external:advisor/product-manager · 2026-10-10T14:21:36.938Z
split off br-3zhx: Message delivery you can check, part 2: a message stays unread until the session has seen it; refuse a wake started with & or discarded output (9aj2 items 1, 4)

### note · agent:w9aj2 · 2026-10-10T20:01:11.945Z
done: bridle messages, message.* events with channel, events --message/--to, role files; just check exit 0, 1461 tests passed; decac24b
