---
id: br-rdkn
title: "Usage you can trust: every session counted, costs right, the weekly limits charted"
kind: epic
created: 2026-10-12
created_by: external:advisor/product-manager
theme: performance
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-xxw9, br-368g, br-yydm, br-kpgy, br-qhsa, br-n4p9]
---

## The ask

The human, 2026-10-12 ~6:00 PM ET (to advisor product-manager), verbatim: "I'm shutting down some agents; we're nearing the weekly limit a little too closely IMO" ... "seems ok for the moment, but we need tracing and reporting to know for sure. can you check on the status of any of that work and requests?"

On the PdM's three suggested first steps (fix the negative costs, count the interactive sessions, the week-to-week charts), ~6:35 PM ET: "Agree on 1-3. Let's do it" and "Put this in a proper theme and epic of course".

## Outcome

The human can see, and trust, where the account's token budget goes: every session counted, every cost correct, and how close each week comes to the five-hour and seven-day limits.

## Done when (PdM draft)

1. No agent's cost in `bridle usage` is ever negative; a session restart or renew doesn't skew the totals (br-9y7v).
2. The human's and the external sessions' use (orchestrator, advisors, aide) is counted in `bridle usage`, or the gap is reported per session, so the biggest consumers are not invisible (br-hd28, which answers br-368g's question).
3. bridle-ui shows week-to-week charts of the five-hour and seven-day limits from the history bridle already keeps (br-yydm; data from br-xxw9).
4. The human picks any agent instance (live, removed, or external once br-hd28 lands) and charts its usage over time (br-xfy7, ui-mxkj). The human, 2026-10-12 ~6:50 PM ET: "When this is done I want to be able to choose a specific agent instance and chart its usage".

## Child tickets

- br-9y7v: negative costs for long-running agents (bug)
- br-hd28: count the external and interactive sessions (answers br-368g)
- br-yydm: the gateway route for the limit history; ui-u6uz: the Usage page with the limit charts
- br-xfy7: usage per agent instance over time; ui-mxkj: pick an agent and chart it
- br-njjt: the external sessions chart like instances (xfy7's item 3, not built in xfy7)

Later, in the theme, not this epic: br-qhsa (gateway usage routes), br-n4p9 (which projects and agents use tokens), br-kpgy (bridle work vs real projects), br-jxwr (task timing).
