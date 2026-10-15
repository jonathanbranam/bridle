---
id: br-eapt
title: "Daemon usage sessions: each hosted session's start/end reason, renewal limit, startup cost and context over turns (for the context and cost pages)"
kind: feature
created: 2026-10-14
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-2kam, br-v6cg]
related: [br-ktgd]
parent: br-rdkn
theme: performance
resolved: 2026-10-15T12:47:21Z
---

Daemon half of the context and cost pages (UI: ui-evy7). Epic br-rdkn; design record br-ktgd decisions 5-7 (startup cost, sessions for hosted agents, before/after renewal-limit comparison).

The human, 2026-10-13 ~10:30 PM ET (to advisor product-manager), verbatim: "For the context and cost management go ahead and design and add pages to the UI that shows all of those charts so I can view them day by day this week"

## The ask

One additive route (bridle-api types.rs), `GET /v1/usage/sessions?since=`, listing every hosted session (agent id + session id, from br-pac3's `turns.session_id`) with a turn in the period:

- agent id, name, role, model, project; started_at, ended_at; start and end reason and the renewal limit in effect (from br-2kam).
- turns, tokens (input, output, cache read, cache write), cost, busy time.
- startup: the first turn's cost, context_tokens, cache read and cache write (ktgd decision 5: what startup eats, and whether it is cached across sessions).
- peak context_tokens and context at end.

And `GET /v1/usage/sessions/{agent_id}/{session_id}/turns`: each turn's n, started_at, context_tokens, tokens and cost, oldest first (the context growth line). Old turns with NULL session_id are left out, never shown as zero.

External sessions (statusline) are listed too where the data allows (cost and context per report; no turns, reasons null).

The gateway serves both to the UI across projects, the same way br-v6cg serves the other usage routes (sessions keep their project; unreachable projects shown as unreachable, never zero).

Acceptance: a test with an agent renewed once gives two sessions with the right reasons, limit, startup and peak context; `just check`. Docs: api.md, usage-and-budget.md, CHANGELOG. Size S-M.
