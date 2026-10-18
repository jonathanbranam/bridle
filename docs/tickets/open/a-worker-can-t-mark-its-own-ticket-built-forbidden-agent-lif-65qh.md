---
id: br-65qh
title: "A worker can't mark its own ticket built: forbidden, agent lifecycle endpoints (its claim was gone)"
kind: bug
created: 2026-10-18
created_by: external:advisor/tj7q
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-4mbd]
---

## The ask

A worker could not mark its own finished ticket built. The worker on br-4mbd (agent:w4mbd) claimed it, committed, ran `just check` green (1866 tests), then `bridle ticket built` was refused:

> forbidden: worker agents cannot use agent lifecycle endpoints

agent:manager-2 had to mark it built for the worker (its note reads `planned -> built`, not `claimed -> built`). The human (2026-10-18, to advisor tj7q): "Yes. This is critical infrastructure workflow behavior that must be allowed."

## What the code does

`built_task` (crates/bridle-daemon/src/server.rs, `POST /v1/tickets/{id}/built`) only lets a worker through when the row's `claimed_by` is that worker; otherwise it calls `require_not_worker`, whose message talks about agent lifecycle endpoints (wrong for a ticket transition). The row was `planned` when the manager marked it built, so the worker's claim (note m-16417, `planned -> claimed by agent:w4mbd`) was gone by the time it ran `built`. Two daemon self-upgrades restarted the daemon overnight (06:15Z and 09:17Z); one of them, or something else, may have dropped the claim. Unconfirmed.

## Done when

- A worker can always mark built the ticket it was spawned to build, including after a daemon restart or a lost row claim (for example, also accept the worker whose agent record names the ticket).
- Find and fix what dropped w4mbd's claim, or show it wasn't dropped.
- The refusal, when one is right, names the actual rule (not "agent lifecycle endpoints").
- A test covers a worker marking its own ticket built after its row claim is lost.
