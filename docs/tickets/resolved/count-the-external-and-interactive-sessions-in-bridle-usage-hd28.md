---
id: br-hd28
title: Count the external and interactive sessions in bridle usage (orchestrator, advisors, aide, the human's own)
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
theme: performance
repos: [bridle]
changes: []
specs: []
blocked_by: [br-9y7v]
related: [br-368g, br-rdkn, br-9y7v, br-gztq]
parent: br-rdkn
resolved: 2026-10-12T23:47:57Z
---

## The ask

The human, 2026-10-12 ~6:00 PM ET (to advisor product-manager), verbatim: "we're nearing the weekly limit a little too closely IMO" ... "we need tracing and reporting to know for sure". Approved ~6:35 PM ET: "Agree on 1-3. Let's do it" (this is item 2: make the external sessions show up at all).

## The problem

`bridle usage` counts only the agents a daemon spawns. The orchestrator, the advisors (several at once), aide and the human's own Claude Code sessions spend the same account but are invisible, and they may be the largest consumers. br-368g (question) set out the options; br-gztq gap 6 found the `interactive_usage` table dead (nothing posts it since s8kn) and the status SessionView carries tokens only.

## The ask (answers br-368g)

1. Each external session's tokens and cost are recorded over time: per session, with its identity (`external:orchestrator`, `external:advisor/<name>`, `external:aide`, or an unbridled session), project and model.
2. `bridle usage` (and `--by role|agent`, `--since`) includes them next to the hosted agents, clearly marked as external, so the totals cover the whole account as far as bridle can see it.
3. A session bridle can't measure is reported as not measured, never as zero.

Recommendation for the planner: revive the statusline post (it already runs in every interactive session and receives Claude Code's running cost and token totals) into `interactive_usage`, rather than reading Claude Code's session files. Its totals are running totals per session, so take deltas per session id and start again on a new session (the same trap as br-9y7v). Check docs/design/usage-and-budget.md and docs/design/storage.md and keep them in step.
