+++
id = "br-cufw"
title = "Mail between daemons, slice 4: visible state: outbox status, message show (queued/arrived/delivered), the aide report, and who-can-I-message (3haz P6, P7, bp2v)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T21:04:43.237Z"
updated_at = "2026-10-10T02:59:25.433624Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
ticket = "3haz"
+++

Ticket: docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md (slice 4 of 6, P6, P7, P8 step 4, and Q4 states). Depends on br-3haz (slice 1) and br-fvkq (slice 3, retry).
Goal: the sender sees what happened. `bridle send` returns a message id with "queued for <peer>". `bridle status` shows an outbox line per peer (e.g. "outbox nuc 3 queued, unreachable 2h"). `bridle message show <id>` reports queued (in the sender's outbox), arrived (stored on the recipient's daemon, recipient not yet woken) or delivered (recipient woken and received it; for now delivered also marks read, per Q4). A message undelivered for over an hour (configurable) is reported to the human through the aide, once. Add the "who can I message and how to reach another machine" listing from ticket bp2v, plus the role prompts and `bridle send --help` text.
Files likely: crates/bridle-daemon, crates/bridle-api types, crates/bridle/src (status, send, message), workflow/base/roles/*, docs/design/cli.md.
Migration: role prompt text reaches projects through workflow sync; no per-project files change.
Acceptance: just check passes; tests for the three states, the status line and the one-time report; docs updated.
Model: Sonnet. Out of scope: wake reasons as messages, task watch across projects.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:38.966Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:59:22.454Z
advisor (product-manager): readied. The human, 2026-10-09 ~10:55 PM ET: "if the machine work finishes up, let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters; I think the scheduled message work is also an important epic to finish up soon". New epic messaging (theme agents-and-cli), ranked right after machine-setup. Order: br-2msq, br-n7cg, br-rhba, br-ysmu, br-cufw.
