+++
id = "br-hwek"
title = "A named advisor's unread mail stays in its own inbox when its session ends or restarts (gtzx P4, pulled forward)"
kind = "bug"
state = "integrated"
created_at = "2026-10-10T14:44:55.731Z"
updated_at = "2026-10-10T19:17:09.247510Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/whwek"
commit = "5b66c6b964ed86dad3d1c11de7a4578a23a4c4e0"
summary = "A named advisor's unread mail now stays in its own inbox. Dropped the move in Sessions::emit_ended; resolve_targets keeps mail for external:advisor/NAME in its own inbox whether or not it runs (an @machine address still goes to the shared inbox, marked). Send responses carry a new Message.recipient_note ('NAME isn't running; waiting in its inbox'), which the CLI prints instead of inferring it from 'to'. Recovery: Sessions::recover_stranded runs on every session register and moves unread '(originally for advisor/NAME)' mail from external:advisor to the named inbox, stripping the mark; idempotent, so it covers every daemon without a one-off. Docs: principals.md, cli.md, CHANGELOG. Test named_advisor_addressing_and_delivery_fallbacks updated (covers end, send while not running, recovery once, unmarked main-advisor mail untouched)."
ticket = "hwek"
+++

Ticket: docs/tickets/open/a-named-advisor-s-unread-mail-stays-in-its-own-inbox-when-it-hwek.md (read it, with the incident entry 2026-10-10 14:36 in docs/context/incidents.md). Goal, items 1-5 of the ticket: (1) drop the move of unread mail to the shared external:advisor inbox in Sessions::emit_ended (crates/bridle-daemon/src/sessions.rs); (2) mail sent to a named advisor that is not running stays in its own inbox instead of the send-time fallback (server.rs, fell_back); the next session of that name gets it from its first bridle inbox or wake; (3) the sender is still told the recipient is not running, worded as waiting in its inbox; (4) recovery: move existing unread messages marked originally for advisor/NAME in the shared inbox back to the named inbox, done automatically when that named session next starts (and/or a one-off at daemon start), on every daemon; (5) update docs/design/agent-host/principals.md (fallback paragraph), cli.md send wording, CHANGELOG, and the test named_advisor_addressing_and_delivery_fallbacks. Migration: item 4 is the migration for existing stranded messages, automatic and idempotent; no project files change. Acceptance: just check passes; tests: session end leaves unread mail in the named inbox, send to a non-running named advisor lands in the named inbox with the sender note, stranded messages are moved back once and not twice, unnamed external:advisor behaviour unchanged. Model: Sonnet. Out of scope: the rest of gtzx (seats, retire), waiter acknowledgement (br-3zhx).

## Thread

### note · external:advisor/product-manager · 2026-10-10T14:45:11.925Z
advisor/product-manager (PdM): placed in the messaging epic, ahead of br-9aj2. The human, 2026-10-10 ~10:45 AM ET: "this needs to be handled more quickly". Worker order: br-ygkc, br-bcw6, br-57ec, then br-hwek, then br-9aj2, br-3zhx as before. Small: drop the move in Sessions::emit_ended, keep send-time mail in the named inbox, recover the stranded messages (ticket hwek, items 1-5). Incident: docs/context/incidents.md, 2026-10-10 14:36.

### note · agent:whwek · 2026-10-10T18:49:08.838Z
done: hwek items 1-5; just check exit 0, 1455 tests, main merged; 0c1b2317

### note · agent:manager-2 · 2026-10-10T19:17:09.247Z
integrated: 5b66c6b964ed86dad3d1c11de7a4578a23a4c4e0 (branch bridle/whwek)
