+++
id = "br-rwj5"
title = "status waiter_open false and last_wake_at null while external:aide waiter is open (notes daemon, vesta)"
kind = "bug"
state = "pending"
created_at = "2026-10-29T01:27:18.018Z"
updated_at = "2026-10-29T01:27:29.232512Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "rwj5"
+++

docs/tickets/open/status-waiter-open-false-and-last-wake-at-null-while-externa-rwj5.md

submitted by external:orchestrator

Reported by aide (notes) on vesta, relayed by orchestrator@vesta (m-0464, from vesta m-0131).
The human asked for it (2026-10-28): "Interesting. Yes report that please." (human via aide-notes and orchestrator@vesta)

## The ask

On the notes daemon (vesta), `bridle status --json` shows `waiter_open: false` and
`last_wake_at: null` while the notes aide has a live `bridle agent wake external:aide --timeout 5400`
waiter open (it printed "waiting as external:aide (pid 316490 ...)").

Delivery itself works: every message that day reached the waiter within 3-5 s
(`bridle messages --last 10`: m-0150..m-0158, "delivered via waiter"), including a test aide
scheduled at 18:30 while status said false. So the status field doesn't count (or doesn't see)
the external:aide waiter, and `last_wake_at` is never set.

Consequence: the 15-minute no-waiter incident and the base role's "check waiter_open" advice
both mislead.

Note: on vesta-home, `bridle status` does show orchestrator@vesta's own waiter ("waiting"), so it
may be specific to external:aide or to non-home daemons.

Caveat: vesta's bridle clone is ~810 commits behind dalek's main, so this may already be fixed
on current main; check that first.

Fix: make `waiter_open` / `last_wake_at` reflect an open external:aide waiter (or whatever
principal is waiting) on a non-home daemon.

Verify: on a non-home daemon with an aide waiter open, `bridle status --json` reports
`waiter_open: true`, and `last_wake_at` is set after a wake is delivered.

## Thread

### note · external:orchestrator · 2026-10-29T01:27:18.019Z
submitted by external:orchestrator
