+++
id = "br-djru"
title = "Message delivery you can check, part 2: a message stays unread until the session has seen it; feasibility of refusing background waiters (9aj2 items 1, 4)"
kind = "feature"
state = "pending"
created_at = "2026-10-10T13:39:37.383Z"
updated_at = "2026-10-10T13:39:37.384525Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:aide",
]
parent = "br-9aj2"
+++

Ticket: docs/tickets/open/message-delivery-you-can-check-a-message-stays-unread-until-9aj2.md (items 1 and 4; read it). Part 2 of br-9aj2; build only after part 1 (br-9aj2) has landed, because both touch the message read path and event channel. Item 1: bridle agent wake prints a message but marks it read only when the same session next runs bridle agent wake or bridle inbox (implicit acknowledgement), so a waiter whose output is lost loses nothing. Defaults (PdM): an unacknowledged delivery is offered again on the next wait from any session of that principal, no timeout, the sender is not told. If that does not fit, ask on the ticket via bridle task ask. Record delivered vs acknowledged as separate audit events with the channel (part 1). Files: crates/bridle-daemon (message read path, wake endpoint), crates/bridle/src (agent wake, inbox), docs principals.md cli.md api.md, role Waiting for messages sections, CHANGELOG. Item 4: first check feasibility of refusing a wake run as a background shell job or with output discarded (a Claude Code background command has no tty either); implement only if it can be detected reliably, otherwise drop it and say so in the done note and the ticket. Migration: behaviour change in daemon and CLI, reaches agents on daemon upgrade; messages already marked read stay read; state this. Acceptance: just check passes; tests: a delivered-but-unacknowledged message is offered again on the next wake and by inbox, acknowledged once the same session runs either, two waiters do not lose a message. Model: Sonnet. Out of scope: timeouts, telling the sender.
