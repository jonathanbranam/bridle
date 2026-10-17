---
id: br-dg5g
title: Orchestrator mail from other same-machine projects forwards to <host>-home even when the orchestrator waits on another project
kind: bug
created: 2026-10-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-17 ~11:30 AM ET, to advisor (product-manager): "dg5g yes, approve it ...
just get it done. It's fine." Priority high. The ask is the "Ask" section below.

## What happened

2026-10-17, dalek. The human kept the dalek orchestrator on bridle ("NO! I can't move the dalek orchestrator!", then "revert both"), so its one waiter is on bridle. After the j86p re-pair, other same-machine projects (bridle-ui at least) forward the orchestrator's wakes and mail to dalek-home, where no waiter is open. manager-3's question on bridle-ui (dalek-home m-0122, ~12:00Z) sat unread ~2.5 h and stalled the bridle-ui queue; the orchestrator found it by hand.

## Ask

Forward to the daemon where the orchestrator's waiter actually is (or the project that supervises its session), not to <host>-home by name; or have <host>-home relay to it. Until then, any project whose home is dalek-home is unwatched on dalek.

Files likely: crates/bridle/src/token_pair.rs (home recorded on the token), the daemon's forwarding. Acceptance: a message to external:orchestrator on bridle-ui wakes a waiter on bridle.
