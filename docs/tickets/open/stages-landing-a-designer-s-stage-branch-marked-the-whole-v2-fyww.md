---
id: br-fyww
title: "Stages: landing a designer's stage branch marked the whole v2 ticket integrated"
kind: bug
created: 2026-10-18
created_by: external:advisor/tj7q
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-upxq, br-2ae8, br-ewk8, br-ewhy]
---

## The ask

Found in the human's first version 2 try (br-ewk8, ticket br-2ae8). The human, 2026-10-18, to advisor tj7q: "Of course file the bug to get it fixed. Yes put 2ae8 back to the correct state."

## What happened

- br-2ae8 is a pending version 2 ticket: only its proposal stage is written (`written: [proposal]`, `claimed_by: human`, waiting for the human's signature). Design, specs and tasks are not written; nothing is built.
- manager-2 landed the designer's branch (`bridle/designer-2ae8-proposal`, the proposal stage only) on main as f9b74cf1, and its land step moved the row `pending -> integrated` (note m-16796).
- So the ticket reads as done, and the stage watch (br-ewhy), which only looks at `pending` v2 tickets that aren't prepared, will never spawn writers for its next stages.
- There's no way back: `ticket reopen` goes to `reopened`, not `pending`.

## Done when

- Landing a branch that carries only a stage of a not-yet-prepared version 2 ticket never changes the row's state; the daemon refuses `pending -> integrated` (or any transition to `integrated`) for a v2 ticket that isn't prepared and built. br-upxq makes the daemon land stage branches itself; that path must not change the state either, and the manager's land path must refuse this case too.
- The manager rule/land step tells a designer branch from a worker branch (or never gets designer branches, once br-upxq lands).
- A test: landing a proposal-only branch of a pending v2 ticket leaves it `pending`.
- br-2ae8 is back to `pending` (done by hand now by the orchestrator).
