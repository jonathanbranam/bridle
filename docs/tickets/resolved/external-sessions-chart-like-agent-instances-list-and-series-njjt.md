---
id: br-njjt
title: "External sessions chart like agent instances: list and series include br-hd28's sessions (follow-up to br-xfy7)"
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
theme: performance
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-xfy7, br-hd28, br-rdkn]
parent: br-rdkn
resolved: 2026-10-13T00:57:04Z
---

Follow-up to br-xfy7, item 3 of its ask, not built (manager-2, landing 9a10c919: "ask item 3 (external sessions chartable like instances) is NOT built; hd28 keeps them in interactive_usage, not turns, so they are not instances here").

The human, 2026-10-12 ~6:50 PM ET (to advisor product-manager), verbatim: "When this is done I want to be able to choose a specific agent instance and chart its usage". The epic br-rdkn's done-when item 4 includes the external sessions (orchestrator, advisors, aide, the human's own) once br-hd28 landed; it has.

## The ask

The external and interactive sessions br-hd28 records (`interactive_usage`, per session with identity) appear in `GET /v1/usage/agents` as instances (marked external; identity as the name, e.g. `external:advisor/product-manager`) and chart through `GET /v1/usage/agents/{id}/series` like hosted agents, with whatever fields they have (a session reports running totals of tokens and cost; busy time and turns may be absent: say so, never zero). Same through the gateway and `bridle usage --agent`. Additive to the br-xfy7 types; ui-mxkj then needs no change beyond showing the external mark.

Acceptance: a test with two statusline reports from one external session lists it as an instance and charts it; docs (usage-and-budget.md, api.md) and CHANGELOG.
