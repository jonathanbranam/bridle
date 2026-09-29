---
id: h8tq
title: bridle budget should show what the governor actually applies
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [ed9ea02]
specs: []
needs: []
see: [c424, kv7d, 6t29]
---

## The ask

The human, verbatim (2026-09-28), after a `burst` override seemed not to work:

> yes, file that issue to make the budget accurate

## What's wrong today (@ d45523b)

Seen on 2026-09-28 between 18:05 and 18:14 UTC, with `bridle budget override burst` in force
(`burst`: `hold_at = 96`, `wind_down_at = 98`, `stop_at = 99`):

1. **Thresholds shown are the plain config, not the applied ones.** `GET /v1/budget`
   returns `thresholds: cfg.to_wire()` (`crates/bridle-daemon/src/server.rs:370`), so it
   showed hold 90 / wind-down 93 / stop 95 while the governor was using `burst`'s 96/98/99
   (`effective_five_hour_thresholds`). Nothing tells the human which period applies.
2. **The state can contradict the reading.** At 18:13:37 `five_hour` reported
   `status: "allowed_warning"` at 92%, while `bridle budget` still said `normal` for both the
   window and the governor. `allowed_warning` forces wind-down (kv7d), so the state flips
   between `normal` and `winding_down` as readings with and without the warning arrive (18:06
   wind-down, 18:08 normal). The human can't tell which one is real, or why.
3. **`stale` only means "no reading".** `stale: rl.is_none()` (`server.rs`), so a reading
   hours old shows as fresh. Readings only come from bridle's own sessions, so while no agent
   runs, usage from the human's own chats doesn't show up at all.
4. **`bridle budget`'s text doesn't show `status`** (`allowed_warning`/`rejected`), which is
   what drove the wind-down.

## What "accurate" should mean

- Show the applied thresholds per window and where they come from (override, schedule
  period by name, or default). This overlaps c424's "show the applied budget"; do them
  together.
- Show each reading's age (and `status` when it isn't `allowed`), and mark it stale by age
  (`max_staleness`), not only when missing.
- Say why the state is what it is: which window, threshold or status caused it.
- Consider a probe when idle (spike u7pw's `get_usage`) so the reading follows the human's
  own usage, if that's cheap; otherwise just show the age.

Resolve kv7d first: if `allowed_warning` stops forcing wind-down, point 2 mostly goes away.

## Resolution

Resolved by ed9ea02: `bridle budget` shows the applied thresholds with their source (docs/design/usage-and-budget.md).
