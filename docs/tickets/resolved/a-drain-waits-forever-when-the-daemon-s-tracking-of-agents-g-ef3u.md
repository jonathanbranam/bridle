---
id: br-ef3u
title: "A drain waits forever when the daemon's tracking of agents goes stale: find and name what really holds the quiet point"
kind: bug
created: 2026-10-12
created_by: external:advisor/product-manager
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kt25, br-y455, br-btdn]
resolved: 2026-10-17T12:32:33Z
---

## The ask

The human, 2026-10-12 ~10:17 AM ET (to advisor product-manager, dictated, in the grilling session on kx5u/kt25, Q5 on the drain), verbatim:

> The main thing we want to avoid is the stale agent problem, the stale worker problem. We want to avoid two things:
>
> 1. We've run into this several times where the restart was held because there was no quiet period due to some bugs in the agent tracking the background worker agents. That's something we want to avoid, and the orchestrator watching and observing the system can help with that and finding any bugs there.
>
> Those two issues should be filed, and we need to work on those as part of a later epic, but we're not going to block the project move MVP for those fixes yet. We'll address them manually for now, which is, I think, perfectly fine.

(Item 2 is its own ticket: a manager waiting on `just check` read as idle.)

## The problem

A drain (restart, self-upgrade, and soon a project move, kt25 K4) waits for a quiet point: no agent mid-turn, no spawn in flight (docs/design/agent-host/daemon.md, "Restart in place"). When the daemon's view of its agents is wrong, the quiet point never comes and the drain waits forever. It has happened several times; each fix closed one cause:

- br-y455 (incident): a stuck "spawning" flag held restarts for ~23 h while every agent showed idle.
- br-btdn (resolved): the stuck-spawn case bounded at 15 min.

The ask is the class, not one more instance: find the remaining ways the tracking can go stale (agents that exited without being recorded, background workers, spawns), and make a drain that is held by something that is not really running say so by name, so the orchestrator or the human can see and fix it.

## Scope and order

- Later epic (drain behaviour review; the human, Q3 of the kt25 design: upgrades "too weak and never happen, or too aggressive and interrupt work"). Not on kx5u's path: during MVP moves the orchestrator watches the drain and these cases are handled by hand.
- Needs a design or investigation pass first (which tracking paths can go stale); no build until the human reviews.
