---
id: br-xfsp
title: "Cost per ticket and what each agent is doing: worker build vs. manager merge (future)"
kind: explore
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ktgd, br-rdkn]
theme: performance
---

## The ask

Future: not scheduled. The human, 2026-10-13 ~6:30 PM ET (to advisor product-manager), verbatim, trimmed: "There's another level to this that goes somewhere else, which is keeping track of what an agent is doing, particularly the background agents, but also the interactive agents. The orchestrator does a lot of things. The project manager does a lot of things. ... for the background workers that have clearly defined what they're doing when, we should have a ticket ID, right? I should be able to tell, for this ticket, how much we spend for the worker to build it and the manager to merge it, because those two things could be different. ... If that's not possible right now, that's fine. That's a future enhancement ... Make that a separate thing, and then we can talk about that later."

Today: `turns` has no ticket column (usage-and-budget.md lists "the ledger's task ... columns" as planned). A worker usually holds one claimed ticket, so its turns could be stamped with it; a manager or pm-1 switches between tickets within one session, so attributing their turns needs a notion of "what this agent is working on now".

To discuss with the human before any design: per-ticket cost (build vs. plan vs. merge), and a wider "what is each agent doing" record for hosted and interactive agents.
