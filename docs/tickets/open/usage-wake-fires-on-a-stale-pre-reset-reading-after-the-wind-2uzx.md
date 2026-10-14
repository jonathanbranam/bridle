---
id: br-2uzx
title: Usage wake fires on a stale pre-reset reading after the window has reset
kind: bug
created: 2026-10-14
created_by: external:orchestrator
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-63nr]
---

## The ask

Seen 2026-10-14: the seven_day window reset at 07:00Z and bridle usage budget showed 0% (resets 2026-10-21). At 07:05:39Z the home daemon still raised 'seven_day usage at 89%' from a reading with resets_at 2026-10-14T07:00:00.422Z, already in the past (wake event observed_at 07:05:39Z). vesta's daemon sent the same wake at 07:11Z (m-12469). The post-reset 0% reading had re-armed the wake (br-63nr's hysteresis), so one late or stale reading from before the reset fired it again. Likely fix: drop a usage reading whose resets_at is already past (or older than the newest window seen) before check_usage looks at it, for the wake and for the governor. Add a test: reading at 89% for the old window after a 0% reading for the new one fires nothing. Cost of not doing it: one spurious wake per reset per daemon, so low priority.
