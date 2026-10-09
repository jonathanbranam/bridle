---
id: gztq
title: "Evaluate the UI: can the human see which projects and which agents consume tokens?"
kind: research
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [xypj, qhsa, n4p9, 368g]
tasks: [br-gztq]
closed: 2026-10-09T23:11:07Z
---

## The ask

The human, 2026-10-05 ~4:10 PM ET, verbatim (to the aide, by voice): "Yeah, also, I need someone to evaluate the user interface. I'd really like to, I really want to be sure I can see which projects, which agents are consuming tokens."

The ask: someone evaluates the bridle UI (bridle-ui with the gateway) against this need: can the human see, across all projects, which projects and which agents are consuming tokens (and how much, over what period)? Report what the UI shows today, what's missing, and file the tickets to close the gaps.

What's there today: `bridle usage --json` has per-agent tokens and cost for one project's daemon; the account's budget is shared by every project (xypj).

## Findings (research from code and docs; nothing run live)

Short answer: **no, not yet.** The UI shows a lifetime cost per agent and an account-wide window
percentage, but no token totals, no period, and nothing that sums or compares projects.

### What the UI shows today

- Gateway (`crates/bridle-gateway/src/system.rs`, routes `/projects/{project}/system` and
  `/projects/{project}/agents`) and the System page (bridle-ui `src/System.tsx`), one project at a time:
  - Per daemon: `budget_state`, and each window's `utilization` as a percent (`rate_limits`, the
    latest reading only).
  - Per agent (`AgentView`): `cost_usd_total` (lifetime for that agent, list-price estimate),
    `context_tokens` (the latest context size, not tokens consumed), `turns`, role, model, task.
  - Per interactive session (`SessionView`): `tokens`, no cost, tied to one daemon's status.
  - Stopped agents are listed after running ones, so lifetime cost of the finished ones is visible, but
    only for as long as the agent row exists (turns outlive `rm`, the agent view does not).
- Interactions reports (`/interactions/*`) are the human's time, not tokens.

### What the daemon already records and serves (so most gaps are gateway + UI, not bridle)

- `GET /v1/usage`: per-agent turns, the four token counts, cost, busy and wall seconds, totals, cache
  hit ratio, `rate_limits`. All time, one daemon.
- `GET /v1/usage/breakdown?by=role|model|agent&since=`: the same from the turns ledger, so a period
  works. No `project` or task-kind grouping (no such column; usage-and-budget.md, "Tracking token use
  over time").
- `GET /v1/usage/history?window=&since=`: the window utilization series (`rate_limit_history`,
  SCHEMA_V21, kept 90 days). **The backend half of xxw9 is built.**
- The gateway exposes none of these three. `grep usage` over the gateway routes finds nothing; the UI
  has no usage route or page.

### What is missing, and what each needs

| Gap | From bridle | From the UI |
|-----|-------------|-------------|
| 1. Any usage view at all (per-agent tokens, cost, period) | gateway route proxying `/v1/usage` and `/v1/usage/breakdown` per project | a usage page or a section of System |
| 2. Across projects: a total per project, and which project burns most | none for tokens per project; each daemon is already one project, so the gateway sums its daemons' answers. Cost is list-price and comparable across projects | project rows + totals, a share bar |
| 3. Over a period (today, 5h window, 7d, 30d) | `since` exists on breakdown; `/v1/usage` (all time) has none; the gateway picks `since` from presets | period selector |
| 4. Which agents, across projects, ranked | `breakdown?by=agent` per daemon; the gateway merges and sorts | ranked list, role/model filter |
| 5. Percent of the shared budget, not just dollars | the account's utilization is the same series in every daemon (xypj), so one daemon's reading is enough; how many tokens a percentage point is, is unknown (usage-and-budget.md caveat). Show cost share, label it an estimate | chart beside the window gauge |
| 6. Interactive (non-hosted) use | `interactive_usage` is dead (nothing posts it since s8kn); sessions have `tokens` only | the human's own sessions can be the largest consumer and are invisible. Needs a bridle decision before any UI |
| 7. Windows over weeks | `/v1/usage/history` is built | the xxw9 charts: gateway route + page. **Do not duplicate: xxw9 owns it** |

### Cross-project aggregation given one account budget (xypj)

- The window percentages are account-wide and identical in every daemon, so the gateway must show
  them **once**, not summed. Per-agent and per-project cost, by contrast, are per daemon and add up.
- The gateway already enumerates every project's daemon (`/projects`, `items.rs`, `collect.rs`, with
  an `unreachable` state), so aggregation belongs there. No shared store is needed for this view.
- Limits: a daemon that is down contributes nothing (show "unreachable", never a silent zero); the
  account is also used by the human's laptop sessions, which no project's ledger sees (gap 6); cost is a
  list-price estimate, so call it "estimated cost" in the UI.
- Overlap with s6cj/7sd9: the System page (7sd9) is where per-project agents sit; a usage page should
  link to it rather than copy it. xxw9 covers the window-history charts only.

### Tickets filed (each `see` gztq)

Filed: qhsa (gateway usage routes, gaps 1 and 3), n4p9 (Usage page, gaps 2, 4, 5), 368g (interactive sessions, gap 6, a question). Gap 7 stays with xxw9.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). The evaluation is in "Findings" above; br-gztq integrated.
