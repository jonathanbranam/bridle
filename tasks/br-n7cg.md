+++
id = "br-n7cg"
title = "Mail between daemons, slice 2: mail for a visitor is forwarded to its home daemon (3haz P2)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T21:04:36.179Z"
updated_at = "2026-10-10T02:59:25.182298Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
ticket = "3haz"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md (read it; this is slice 2 of 6, P2 and P8 step 2). Depends on br-3haz (slice 1: outbox, peer tokens, forwarding).
Goal: replies come home. A visitor's token records its home (machine and project daemon). When a daemon gets mail for a visitor (e.g. external:aide@nuc on dalek's daemon), it forwards it to that home daemon through its own outbox instead of keeping it in a visitor inbox, so the recipient's ordinary waiter on its own daemon wakes for it and nobody polls another machine. Also: every principal has a home daemon (the orchestrator's is the machine's bridle daemon); other daemons forward that principal's messages there. Remove the "read the reply on the remote daemon" text from the k7mw design doc and fix principals.md.
Files likely: crates/bridle-daemon (token/visitor records, outbox use), crates/bridle-api types if the token record gains a home, docs/design/agent-host/principals.md and the k7mw design doc.
Migration: existing visitor tokens have no recorded home; define what happens to them (keep local inbox as today) and say so. New field is optional.
Acceptance: just check passes; two-daemon test: a reply to a visitor lands in the home daemon's inbox and wakes its waiter; an old token without a home still works locally.
Model: Sonnet. Out of scope: retry loop, status, wake reasons as messages.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:38.915Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:59:22.401Z
advisor (product-manager): readied. The human, 2026-10-09 ~10:55 PM ET: "if the machine work finishes up, let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters; I think the scheduled message work is also an important epic to finish up soon". New epic messaging (theme agents-and-cli), ranked right after machine-setup. Order: br-2msq, br-n7cg, br-rhba, br-ysmu, br-cufw.
