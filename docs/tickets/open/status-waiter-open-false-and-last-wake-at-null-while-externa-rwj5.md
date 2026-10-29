---
id: br-rwj5
title: status waiter_open false and last_wake_at null while external:aide waiter is open (notes daemon, vesta)
kind: bug
created: 2026-10-29
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human asked for this (2026-10-28, via aide-notes and orchestrator@vesta): "Interesting. Yes report that please."

On the notes daemon (vesta), `bridle status --json` shows `waiter_open: false` and `last_wake_at: null` while the notes aide has a live `bridle agent wake external:aide --timeout 5400` waiter open. Delivery works (messages reach the waiter in 3-5 s, "delivered via waiter"), so the status field does not count, or does not see, the external:aide waiter, and `last_wake_at` is never set. Consequence: the no-waiter incident and the base role's "check waiter_open" advice mislead. On vesta-home, status does show orchestrator@vesta's own waiter, so it may be specific to external:aide or to non-home daemons.

Caveat: vesta's clone was ~810 commits behind dalek's main, so first reproduce on current main (the `bridle dev site` or a test daemon with an aide waiter); if it is already fixed, add a regression test and close.

Fix: make `waiter_open` / `last_wake_at` reflect an open waiter for any principal (external:aide included) on any daemon. Find where status computes them (bridle-daemon status/wake code, docs/design/agent-host/).

Acceptance: a test where an external:aide waiter is open shows `waiter_open: true`, and `last_wake_at` is set after a wake is delivered; `just check`. Model: Sonnet. Out of scope: the incident threshold logic beyond reading the corrected field.
