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

## Decision (the human, 2026-10-18)

The human, to advisor tj7q: "Fix fyww first and decide the solution now. Don't leave to worker." Then, on the design below: "yes approve". Build exactly this.

Cause: `done_task` (crates/bridle-daemon/src/tasks.rs) is the only way to `integrated`. It accepts any state but dropped, integrated and held, `pending` included, and never looks at a ticket's stages.

1. **Guard.** `done_task` refuses a version 2 ticket that isn't prepared. The daemon reads the ticket file on the project checkout and uses `stages::prepared` (crates/bridle-daemon/src/stages.rs), the same way the stage watch does. The refusal says why, for example: "br-2ae8 is a version 2 ticket still being prepared (proposal written; design, specs, tasks not); landing a stage doesn't finish it". It applies to every caller.
2. **Way back.** `ticket reopen` (`reopen_task`) of a version 2 ticket that isn't prepared goes to `pending`, not `reopened`, so the stage watch picks it up again. Version 1 tickets, and version 2 tickets that are prepared, reopen to `reopened` as today. No new command. (Rejected: the stage watch also watching `reopened`; `pending` is the true state and the watch stays as it is.)
3. **Manager rule.** A `bridle/designer-*` branch carries a stage, not a build. The manager may land it (until br-upxq makes the daemon do that) but never runs `ticket done` for it. Say so in workflow/base/roles/manager.md.
4. **Then** reopen br-2ae8; it goes back to `pending` with its proposal waiting for the human's signature.
5. **Tests:** `done` is refused for a version 2 ticket that isn't prepared; reopening one gives `pending` and the stage watch spawns its next writer; version 1 `done` and `reopen` are unchanged.

docs/design/stages.md (and the state table in docs/design/coordination.md or storage.md, wherever reopen is described) say the same.

## Done when

- Landing a branch that carries only a stage of a not-yet-prepared version 2 ticket never changes the row's state; the daemon refuses `pending -> integrated` (or any transition to `integrated`) for a v2 ticket that isn't prepared and built. br-upxq makes the daemon land stage branches itself; that path must not change the state either, and the manager's land path must refuse this case too.
- The manager rule/land step tells a designer branch from a worker branch (or never gets designer branches, once br-upxq lands).
- A test: landing a proposal-only branch of a pending v2 ticket leaves it `pending`.
- An authorised way to put a wrongly moved row back to `pending` (the orchestrator, m-16835: no command or API call moves a row to pending; the only move is the daemon's own stale `open -> pending`, `unready_stale` in crates/bridle-daemon/src/tasks.rs; `ticket reopen` gives `reopened`, which the stage watch ignores). Either a guarded reopen-to-pending, or the stage watch treating a `reopened` v2 ticket that isn't prepared like `pending`; the design picks.
- br-2ae8 is back to `pending` with it. The orchestrator would not edit the database by hand, so 2ae8 stays `integrated` until this lands.
