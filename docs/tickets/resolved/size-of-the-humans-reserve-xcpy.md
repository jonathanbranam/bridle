---
id: xcpy
title: How big is the human's reserve?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [3nqf, xypj]
closed: 2026-09-30T05:12:44Z
---

## The question

From `docs/design.md` §15 @ c192bfc, item 13:

> **The human's reserve.** 25% of the 5-hour window is a guess. The right
> number depends on how much the human works interactively while workers run.

## Why it matters

Running out mid-conversation is the worst outcome
([[docs/design/usage-and-budget#The budget governor|budget governor]]).

## Notes

- The workforce may run on a separate machine
  ([[docs/context/nuc-host|the NUC]]). The reserve is per account, so it has to
  cover the human's sessions on every machine, including Remote Control from a
  phone.

## Resolution

The human, 2026-09-27:

> If we are close to hitting a usage limit like 90% (tunable) then all agents
> should be told to wind down / pause their work to avoid hitting a usage
> alert. […] I'd prefer to have bridle work sitting idle so that I can
> manually use claude for other life purposes.

The reserve is now `wind_down_at`, a per-window threshold defaulting to 90%,
set once per machine in `~/.bridle/config.toml`. Crossing it winds every
agent down; `hold_at` (80%) stops new work first, `stop_at` (95%) is the
backstop, and `bridle budget hold` idles bridle on demand. Recorded in
[[docs/design/usage-and-budget#The budget governor|budget governor]]. Whether
90% leaves enough room is measured by
[[docs/spikes/open/usage-probe-and-wind-down-headroom-u7pw|spike u7pw]].
