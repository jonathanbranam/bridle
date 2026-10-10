+++
id = "br-9aj2"
title = "Message delivery you can check: a message stays unread until the session has seen it, a recent-messages command for any principal, and a full delivery audit trail in the event log"
kind = "feature"
state = "open"
created_at = "2026-10-10T13:38:39.686Z"
updated_at = "2026-10-10T13:39:13.873388Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "9aj2"
+++

docs/tickets/open/message-delivery-you-can-check-a-message-stays-unread-until-9aj2.md

## Thread

### note · external:advisor/product-manager · 2026-10-10T13:39:13.873Z
advisor/product-manager (PdM): readied, normal; placed at the head of the messaging epic (rank 2), after br-37r9 and br-bcw6: a lost message is a correctness bug and the human raised it today. Planner: split it. Build first what has no open question: item 2 (bridle messages), item 3 (audit events with the channel; events filter by message id and recipient), item 5 (role files). Item 1 (unread until the session has seen it): take the ticket's recommendation (marked read when the same session next runs wake or inbox); for the open points, PdM default is: offered again on the next wait from any session of that principal, no timeout, sender not told; ask on the ticket if that doesn't fit. Item 4 (refuse '&' or discarded output): check feasibility first (a Claude Code background command has no tty either); with item 1 built, a lost waiter loses nothing, so drop item 4 if it can't be detected reliably and say so.
