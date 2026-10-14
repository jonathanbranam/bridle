---
id: br-8usa
title: Re-pairing doesn't move a visitor's recorded home when its home project changes (part 2 of x8c8)
kind: fix
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-x8c8, br-8uxe, br-g9qm]
---

## The ask

Part 2 of br-x8c8, split off when x8c8 landed part 1 only (the guard: a visitor whose recorded home is this daemon's own project gets an error, not a loop). The human, 2026-10-14 (to advisor wsl2), on the bugs from the remus home-project setup: "all these bugs must be filed and fixed".

## What's wanted

When a principal's home project changes (an orchestrator moving from `bridle` to `<host>-home`), its visitor tokens' recorded home on other daemons (`store.visitor_home`, used by `forward_to_home` in `crates/bridle-daemon/src/server.rs`) follows. Re-pairing (`bridle token pair`, `token_pair.rs`) updates the recorded home; if that can't be done simply, the setup steps say what to run instead. Docs: add-a-machine's home-project step and the move steps.

Today the workaround is `bridle send --project <host>-home external:orchestrator ...`.

## Acceptance

A test that re-pairing a visitor with a new home updates its recorded home, so mail forwards to the new home; `just check`.

Touches `token_pair.rs`, as did su62 and g9qm: run it after g9qm lands.
