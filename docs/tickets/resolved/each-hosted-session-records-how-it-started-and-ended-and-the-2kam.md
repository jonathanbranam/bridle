---
id: br-2kam
title: Each hosted session records how it started and ended, and the renewal limit in force (ktgd decision 7)
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-pac3]
related: [br-ktgd]
theme: performance
parent: br-rdkn
resolved: 2026-10-14T06:23:50Z
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

For every hosted agent session, record its start (spawn, restart, renewal, resume) and end (renewal at N tokens, handover, stop, exit) and the context renewal limit in effect, so a week at one limit can be compared with a week at another by cost. Reuse events or a small sessions table, whichever is simpler; keyed by agent id + session id (from the turns session id ticket).

Acceptance: a renewal shows as one session ending (renewal, at N tokens, limit L) and the next starting (renewal); `just check`. Docs: storage.md, usage-and-budget.md. Size S.
