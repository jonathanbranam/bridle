---
id: br-63nr
title: Usage wake re-fires every poll while utilization hovers at the threshold
kind: bug
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Seen 2026-10-13 22:29-22:40Z: the orchestrator got a 'seven_day usage at 85%' wake every 5 minutes on the home daemon (plus one from each project daemon), ~100 wakes until the 07:00Z reset. Cause: check_usage (crates/bridle-daemon/src/wake.rs:500) clears usage_fired whenever util < limit, so a reading oscillating 0.849/0.850 (bridle status showed 84% between wakes) counts as a new crossing each time. The test usage_fires_once_per_crossing covers a clean crossing only. Fix: hysteresis, e.g. re-arm only after util falls below limit - 0.03 (or only on a window reset, resets_at changing). Add a test with an oscillating reading. Update orchestrator-supervision.md's wake table line.
