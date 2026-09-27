---
id: u7pw
title: "Spike: polling get_usage, and how much headroom a wind-down needs"
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [mgjh]
---

## What to find out

From the human, 2026-09-27:

> If we are close to hitting a usage limit like 90% (tunable) then all agents
> should be told to wind down / pause their work to avoid hitting a usage
> alert.
>
> It's a huge problem to hit blocked usage, it will kill everything I do with
> claude. I'd prefer to have bridle work sitting idle so that I can manually
> use claude for other life purposes.

The [[docs/design/usage-and-budget#Seeing the windows|governor's readings]]
rest on things spike 01 saw once or not at all:

1. Does `get_usage` answer on a **working** process, mid-turn, as `interrupt`
   does? On a fresh process that has had no user message?
2. Does polling it cost anything: tokens, a window percentage, a rate limit
   of its own? Is every 30 s acceptable?
3. How fast does `five_hour` utilisation move with two Sonnet workers on Max
   5x? That sets whether `wind_down_at = 90` and `stop_at = 95` leave enough
   room for the wrap-up turns and the human.
4. At what utilisation does a `rate_limit_event` switch to `allowed_warning`,
   and does a running process emit a new one when it does?
5. What does `get_usage`'s `limits[]` (`severity`, `is_active`) say near a
   limit, and is it a better trigger than the percentages?

## Why it matters

The human's own Claude use shares the account. Bridle hitting a limit blocks
it until the window resets, so the wind-down must start early enough, from
readings fresh enough, to finish before the limit.

## Notes

- Questions 1, 2 and 5 are cheap: a Haiku process and a handful of control
  requests. 3 and 4 need real usage near a threshold; record them from the
  ledger and probe log the first time bridle runs a workforce that high.
- `rejected` still can't be forced cheaply
  ([[forced-budget-and-rate-limit-errors-mgjh|spike mgjh]]).

## Answers (1, 2, 5) — 2026-09-27, Claude Code 2.1.283, Haiku, ~$0.02 list-price

One throwaway Python driver (not committed) drove `claude -p --input-format
stream-json --output-format stream-json --model claude-haiku-4-5-20251001`:
`get_usage` before any user message, a user message that runs `sleep 5` via
Bash, two more `get_usage` calls while it ran, then two more after the
`result`. Raw output wasn't kept (it carries per-account `behaviors` and
plugin paths under `$HOME`); the excerpts below are trimmed from the console
capture.

| # | Question | Answer | Evidence |
|---|---|---|---|
| 1 | Answers mid-turn / on a fresh process? | **Yes to both, and earlier than spike 01 tested.** Sent at t=0.000 before any user message, before even the first `system/init` (which only appears once a user message is sent) — answered at t=1.267 (~1.27 s). Sent again at t=6.262 and t=10.406 while the `sleep 5` Bash tool was running (the model's `tool_use` landed at t=5.633; the turn's `result` didn't land until t=15.011) — both answered in <300 ms, same as the interrupt control request in spike 01 S4. | `probe-1` (t=0.000→1.267, before `init`); `probe-2` (t=6.262→6.548), `probe-3` (t=10.406→10.682), both mid-turn |
| 2 | Does polling cost anything? | **No, at this scale.** All five `get_usage` calls returned `rate_limits.five_hour.utilization: 68` and `seven_day.utilization: 21`, unchanged from before the turn to after it — the turn's own tokens moved nothing visibly (68%/21% at this account's ordinary load swallows a handful of Haiku turns). More directly: `probe-4` (t=22.517) and `probe-5` (t=25.625), 3 s apart with **no turn running between them**, returned byte-identical `session.total_cost_usd` (`0.013327499999999999`) and `model_usage` — polling itself adds no cost or tokens. No throttling or backoff on the control requests themselves across 5 calls in ~26 s; latency after the first call dropped to 250–550 ms (vs. spike 01's ~1.1 s and this run's own first call at 1.27 s), consistent with a warmed-up process, not a cost. Nothing here contradicts the design's 30 s / 5 min polling cadence — if anything a tighter interval looks cheap too, though this doesn't rule out a *server-side* rate limit on `get_usage` itself, which 5 calls can't surface. | `probe-1`..`probe-5`, all `rate_limits.{five_hour,seven_day}.utilization` identical; `probe-4` vs `probe-5` session totals identical |
| 5 | `limits[]` shape at ordinary utilization? | Three entries, matching spike 01's shape: `{"kind":"session","group":"session","percent":68,"severity":"normal","is_active":true}`, `{"kind":"weekly_all","group":"weekly","percent":21,"severity":"normal","is_active":false}`, `{"kind":"weekly_scoped","group":"weekly","percent":0,"severity":"normal","is_active":false,"scope":{"model":{...}}}`. `severity` was `"normal"` on all three throughout, including the 68%-utilised `five_hour` window, so **no evidence yet that `severity` moves before the percentage does** — that needs real usage near a limit (still question 3/4 territory). `is_active` did track something percentages alone don't: it was `true` only on `session` (the window actually binding right now), `false` on the other two even though `weekly_all` was nonzero — i.e. it looks like "the window that would currently block you," not just "nonzero usage." That's a plausible extra wind-down signal (which window is *active*, not just how full each is) but isn't a replacement for the percentage-based trigger, and can't be confirmed as "better" without seeing `severity` actually escalate. Also observed: the response's `rate_limits` carries many more `null` window slots than spike 01's fixture showed (`seven_day_opus`, `seven_day_sonnet`, and about a dozen internally-codenamed ones, all `null` except one at 0%) — unreleased/flagged window types in the live API, not something bridle should build against yet. | `probe-2` `response.limits` (t=6.548, mid-turn) |

**Bearing on the design doc:** nothing here contradicts
`docs/design/usage-and-budget.md`'s "Seeing the windows" — `get_usage`
answered on both a fresh and a working process, cheaper and faster than the
design assumed (no per-call cost, sub-second after the first call), so
polling at 30 s (or tighter) above `hold_at` looks safe from this data.
`limits[].severity` didn't distinguish itself from the raw percentages at
68%/21%; only `is_active` showed something new, and that's a suggestion, not
a validated substitute — questions 3 and 4 still need real usage near a
threshold to settle it.
