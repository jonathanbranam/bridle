---
id: kv7d
title: allowed_warning forces wind-down even under a raised override
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [8d7a9e5]
specs: []
needs: []
see: [6t29, u7pw, y2eb]
---

## What happened

2026-09-28, 18:05 UTC: the human set `bridle budget override burst` until 22:00 UTC
(`burst` in `~/.bridle/config.toml`: `hold_at = 96`, `wind_down_at = 98`, `stop_at = 99`,
`days = []`). At 18:06:51 the governor went `normal -> winding_down` on `five_hour` at 91%.
`bridle budget` showed `five_hour winding_down 91%` with the override listed, and
`--json` showed `status: "allowed_warning"`.

## Why

`evaluate_window` (`crates/bridle-daemon/src/governor.rs:656` @ da9d1ec) raises the state to
at least `WindingDown` whenever Claude Code's `rate_limit_event` status is
`allowed_warning`, after the thresholds are applied. This is as designed
(`docs/design/usage-and-budget.md`, "Seeing the windows": "`allowed_warning` and `rejected`
events trip the wind-down and `stop_at` respectively, whatever the percentages say").

Claude Code appears to send `allowed_warning` at around 90% five_hour (seen here at 91%;
spike u7pw left the point unknown). So any threshold above ~90% is dead: `burst`, `low`
and `night` (`hold_at = 93`) can never take effect on `five_hour` once the warning starts.

## The question for the human

Should a threshold the human has set win over `allowed_warning`?

Recommendation: yes. Treat `allowed_warning` as information only (show it in
`bridle budget`), and let the configured thresholds decide; keep `rejected` forcing
`Paused`, since that means work is actually refused. The human's thresholds are explicit
and `stop_at` still guards the block. A narrower option: ignore `allowed_warning` only
while an override is in force.

Either way, update "Seeing the windows" in `usage-and-budget.md` and the test
`rejected_status_forces_paused_regardless_of_percent`'s neighbour for the warning case.

## The human's answer

2026-09-28, verbatim, to the advisor: "For kv7d update that my settings override Claude
warnings".

So the human's configured thresholds win over `allowed_warning`, in general, not only
under an override: the warning is shown in `bridle budget` but doesn't force a wind-down.
`rejected` isn't a warning and still forces `Paused` (the recommendation above).

## Resolution

Resolved by 8d7a9e5: `allowed_warning` is information only; the thresholds and overrides decide the governor state.
