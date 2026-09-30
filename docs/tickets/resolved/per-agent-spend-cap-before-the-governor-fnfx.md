---
id: fnfx
title: A per-agent spend cap before the budget governor?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: [docs/design/agent-host/agents.md]
needs: []
see: [xypj]
closed: 2026-09-30T05:12:44Z
---

## The question

From `docs/agent-host.md` §4.9 @ 38532ef (added in the v1 build):

> **No `--max-budget-usd` per agent yet.** The spike used one. Per-role budget
> caps belong to the budget governor (§12.6).

## Why it matters

Until the governor exists, nothing stops one runaway worker from using the
account's five-hour window, and the human's own sessions share that window
([[docs/design/usage-and-budget|usage and budget]]). `claude -p` already
accepts `--max-budget-usd`, and the result's `total_cost_usd` is cumulative
per session and survives `--resume` (spike 01, S7). So a cap would have to
account for resumes, or reset per process on purpose.

## Notes

- One option is a `max_budget_usd` field on `[roles.*]`, passed through as
  `--max-budget-usd`. The spike hasn't seen `result/error_max_budget_usd` yet;
  see [[docs/spikes/open/forced-budget-and-rate-limit-errors-mgjh|spike mgjh]].
- **Needs** nothing to start. It's small build work once decided.

## Resolution

Built: a role's `max_budget_usd` is passed as `--max-budget-usd`. Claude
applies it per process, so bridle stops an agent whose cap is spent, and
`bridle resume` grants a fresh allowance
([spike 02 findings](docs/spikes/02-budget-cap-findings.md)). The budget
governor stays the place for account-wide limits. Recorded in
[[docs/design/agent-host/agents#Spend cap|spend cap]] and
[[docs/design/agent-host/roles-and-config|roles and config]].
