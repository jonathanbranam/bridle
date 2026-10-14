---
id: br-82qp
title: schedule list hides schedules that target the caller
kind: bug
created: 2026-10-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask


Reported by the notes aide (vesta), relayed by vesta's orchestrator. `bridle schedule list` run as external:aide on vesta's notes daemon returns [] although sc-9rnk (created by the human, cron `0 2 * * *`) targets external:aide; the human sees it with their token. The list filters by creator, so an agent cannot see the schedules that wake it. The human, 2026-10-14 1:01 PM ET: "you can't see this yourself?"

`schedule list` also shows schedules whose target is the caller, marked as such. Removing stays creator-only (or the human).

Verify: the human creates a schedule targeting external:aide; `bridle schedule list` as aide shows it.
