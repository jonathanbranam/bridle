+++
id = "br-rhba"
title = "Mail between daemons, slice 5: every orchestrator wake is a message, sent home; one waiter per principal (3haz)"
kind = "feature"
state = "pending"
created_at = "2026-10-10T02:59:02.102Z"
updated_at = "2026-10-10T02:59:02.105395Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:advisor",
]
parent = "br-3haz"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md (3haz), sections "Decided: one waiter per principal, on its home daemon" and "Caveats of every orchestrator wake as a message" (all five caveats accepted by the human, 2026-10-04: "Approve every wake is a message. I think this simplifies a lot of things.").

Goal: every reason a daemon wakes the orchestrator today (red CI, agent death, stall, context warning, task wakes; orchestrator-supervision.md section 5, wake.rs) is sent as a message of a system kind and, with br-n7cg's forwarding, reaches the orchestrator's home daemon. The orchestrator then needs one waiter, on its home daemon, instead of one per project.

The human, 2026-10-09 ~10:55 PM ET: "let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters". Chose the real fix over the --all-projects stopgap (br-1ddd dropped).

Needs br-n7cg (visitor mail forwarded home). Then update the orchestrator role (one waiter, home daemon) and supervision docs. pm-1 plans it from the ticket.
