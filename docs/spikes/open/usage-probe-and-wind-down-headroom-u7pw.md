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
