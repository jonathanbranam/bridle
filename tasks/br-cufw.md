+++
id = "br-cufw"
title = "Mail between daemons, slice 4: visible state: outbox status, message show (queued/arrived/delivered), the aide report, and who-can-I-message (3haz P6, P7, bp2v)"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T21:04:43.237Z"
updated_at = "2026-10-11T02:16:53.170366Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
branch = "bridle/wcufw"
commit = "ae64e497c741028a68547c9ef9d82ecf3cd8f1ac"
summary = 'Slice 4 of 3haz (visible state). Status.outbox: one `outbox <project> N queued, unreachable <age> (<error>)` line per destination. `bridle message show <id>`: queued/arrived/delivered (o- ids via new GET /v1/outbox/{id}, sender or human only, which asks the destination via peer-only GET /v1/forward/{message_id}; m- ids map the local state). Mail queued over an hour ([messages] undelivered_report_mins, default 60) is reported once to external:aide (SCHEMA_V25 aide_notified_at), alongside the existing 30 min sender note. `bridle recipients` + GET /v1/recipients list addressable principals and other daemons (registry + machine config); send --help, advisor and orchestrator roles updated. Docs: principals.md, api.md, cli.md, storage.md, CHANGELOG. Caveat: "arrived" vs "delivered" for an external principal means unread vs read (delivered = read, per Q4). Not done: wake-reason messages, task watch across projects, --task across daemons.'
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

### note · agent:wcufw · 2026-10-11T02:16:46.638Z
done: slice 4 visible state (status outbox line, message show stages, aide report, recipients); just check exit 0, 1484 tests passed; 14d476de

### note · agent:manager-2 · 2026-10-11T02:16:53.170Z
integrated: ae64e497c741028a68547c9ef9d82ecf3cd8f1ac (branch bridle/wcufw)
