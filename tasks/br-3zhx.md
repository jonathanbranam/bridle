+++
id = "br-3zhx"
title = "Message delivery you can check, part 2: a message stays unread until the session has seen it; refuse a wake started with & or discarded output (9aj2 items 1, 4)"
kind = "feature"
state = "planned"
created_at = "2026-10-10T14:21:36.938Z"
updated_at = "2026-10-10T20:51:15.316270Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
]
size = "M"
summary = "Item 1: GET /v1/wake no longer marks a non-human's messages read. The daemon (Waiters, in memory) remembers which session was handed which message; that session's next wake, or any mark_read inbox listing, marks them read (message.read with acknowledged:true). A hand-over logs message.delivered (channel waiter, pid, session). Another session of the principal, or a restart, simply sees them unread and is offered them again; two waiters both get a message, none loses it. No timeout; sender not told. Orchestrator's queue-based wake keeps mark-read-on-handover (not changed; could follow). Messages already read stay read; takes effect on daemon upgrade. Item 4 partly: bridle agent wake refuses stdout that is /dev/null (device+inode check, reliable). A shell & is NOT detected: without a tty (as in a Claude Code command) a background job looks like a foreground one, so that half is dropped; item 1 makes a lost waiter lose nothing. Tests: principal_wake_test, message_audit_test. Docs: cli.md, api.md, aide/advisor role files, CHANGELOG. just check: exit 0, 1462 tests."
parent = "br-9aj2"
+++

Ticket: docs/tickets/open/message-delivery-you-can-check-a-message-stays-unread-until-9aj2.md (read all of it, including the PdM notes on br-9aj2). Part 2 of 9aj2; build only after br-9aj2 (part 1) has landed: both touch the message read path and event channel. The human, 2026-10-10 ~10:15 AM ET: the acknowledgments and delivery related to how the workers and the waits are happening for the agents is a big thing to land appropriately, otherwise we will have missed messages. Item 1: bridle agent wake prints messages but marks them read only when the same session next runs bridle agent wake or bridle inbox (implicit acknowledgement); a delivered-but-unacknowledged message is offered again on the next wait from any session of that principal; no timeout; the sender is not told (PdM defaults; ask on the ticket via bridle task ask if they do not fit). Record delivered vs acknowledged as separate audit events with the channel (part 1). Item 4: refuse a wake started as a shell background job or with stdout discarded, only if it can be detected reliably (a Claude Code background command has no tty either; test both); if not, say so on the task and the ticket and drop item 4, since item 1 already makes a lost waiter lose nothing. Files: crates/bridle-daemon (message read path, wake endpoint), crates/bridle/src (agent wake, inbox), workflow/base/roles waiting sections, docs cli.md principals.md api.md, CHANGELOG. Migration: behaviour change in daemon and CLI, reaches agents on daemon upgrade; messages already marked read stay read; state it in the done note. Acceptance: just check passes; tests with the fake (no real claude): an unacknowledged delivery is offered again on the next wake and by inbox, acknowledged once the same session runs either, two waiters do not lose a message. Model: Sonnet. Out of scope: timeouts, telling the sender.

## Thread

### note · agent:w3zhx · 2026-10-10T20:51:13.492Z
done: waiter messages stay unread until same session's next wake/inbox (separate delivered/read audit events); wake refuses stdout=/dev/null, '&' not detectable so dropped; just check exit 0, 1462 tests; e331965a

### note · agent:w3zhx · 2026-10-10T20:51:15.316Z
Item 4 half dropped: a shell & can't be detected without a tty (a Claude Code command has none either); the discarded-stdout half is built (/dev/null device+inode check). Item 1 makes a lost waiter lose nothing. Orchestrator's wake queue still marks read on hand-over (unchanged).
