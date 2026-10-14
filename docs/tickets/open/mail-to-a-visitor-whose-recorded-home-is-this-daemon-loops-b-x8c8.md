---
id: br-x8c8
title: Mail to a visitor whose recorded home is this daemon loops back to the sender silently (orchestrator@remus after the move to remus-home)
kind: bug
created: 2026-10-14
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-8uxe, br-dffz, br-su62]
---

## The ask

Found by dalek's orchestrator, 2026-10-14 18:15Z (comment on br-8uxe), during the remus home-project setup. The human, same day, on the bugs from that setup (to advisor wsl2): "all these bugs must be filed and fixed".

## What happened

On dalek's bridle daemon, `bridle send external:orchestrator@remus` (a visitor) printed "forwarded to its home daemon, outbox o-0142", and the outbox delivered it to `external:orchestrator` on `bridle`: back to dalek's own orchestrator (m-12835), with no error. The visitor token's recorded home (`store.visitor_home`, used by `forward_to_home` in `crates/bridle-daemon/src/server.rs`) is `bridle`, which on dalek is this daemon itself. Remus's orchestrator now runs in remus-home (8uxe). `bridle send --project remus-home external:orchestrator` delivers correctly (o-0143).

## What's wanted

- Guard: `forward_to_home` refuses (an error to the sender) a home that is this daemon's own project, never delivering a visitor's mail to the local principal of the same name.
- When a principal's home project changes (an orchestrator moving to `<host>-home`), its visitor tokens' recorded home on other daemons follows: re-pairing (`bridle token pair`) updates it, or the setup steps say what to run. Docs: add-a-machine's home-project step and the move steps.

Acceptance: a test that a visitor whose home is the local project gets an error, not a loop; `just check`.
