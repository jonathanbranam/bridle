+++
id = "br-3zhx"
title = "Message delivery you can check, part 2: a message stays unread until the session has seen it; refuse a wake started with & or discarded output (9aj2 items 1, 4)"
kind = "feature"
state = "open"
created_at = "2026-10-10T14:21:36.938Z"
updated_at = "2026-10-10T14:21:37.470556Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
]
size = "M"
parent = "br-9aj2"
+++

Ticket: docs/tickets/open/message-delivery-you-can-check-a-message-stays-unread-until-9aj2.md (read all of it, including the PdM's notes on br-9aj2's thread). The human, 2026-10-10 ~10:15 AM ET: 'the acknowledgments and delivery related to how the workers and the waits are happening for the agents is a big thing to land appropriately. Otherwise, we're going to have missed messages.' Item 1: a waiter prints messages but they're marked read only when the same session next runs bridle agent wake or bridle inbox (implicit acknowledgement); a delivered-but-unacknowledged message is offered again on the next wait from any session of that principal; no timeout; the sender isn't told (PdM defaults; ask on the ticket if they don't fit). Use part 1's audit events to record delivered vs. acknowledged. Item 4: refuse a wake started as a shell background job or with stdout discarded, if it can be detected reliably (a Claude Code background command has no tty either, so test both); if it can't, say so on the task and drop item 4: item 1 already makes a lost waiter lose nothing. Update the role files' waiting sections and docs/design (cli.md, the agent-host docs). Tests with the fake; no real claude.
