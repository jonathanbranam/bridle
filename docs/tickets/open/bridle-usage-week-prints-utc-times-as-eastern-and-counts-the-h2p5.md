---
id: br-h2p5
title: bridle usage --week prints UTC times as Eastern and counts the old week's readings on the first day
kind: bug
created: 2026-10-19
created_by: external:advisor/usage
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-19 (to advisor usage): "Please note any issues like this and investigate and file bugs to address them".

`bridle usage --week` has two display bugs. Found by advisor usage on 2026-10-19:

1. **UTC times labelled as Eastern.** The header read `week 2026-10-14 07:00 to 2026-10-21 07:00 (America/New_York)`. The reset is 03:00 ET (`bridle usage budget`: `seven_day ... resets 2026-10-21 03:00 -04:00`). `print_week` in crates/bridle/src/commands/usage.rs formats `week.starts_at` and `week.resets_at` (UTC) with no conversion to `periods.timezone`. The five-hour window list below it has the same problem; check that too.
2. **The first day shows the previous week's peak.** Day 2026-10-14 showed `7-DAY 89%`, but the new week was at 11% that day. `days()` in crates/bridle-daemon/src/usage_periods.rs picks readings by `observed_at` only. Old-week readings (resets_at 2026-10-14 07:00Z) kept arriving for a while after the reset, from agents whose last `rate_limit_event` was stale. Filter each week's days to readings in that week's `resets_at` cluster (the one `by_reset` already builds). The gateway's account usage page uses the same periods, so check it too.

The ask: both fixed, with a test for each (a stale old-week reading after the reset; a header in the schedule timezone).
