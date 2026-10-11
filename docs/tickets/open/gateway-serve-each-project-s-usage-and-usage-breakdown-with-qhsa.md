---
id: br-qhsa
title: "gateway: serve each project's usage and usage breakdown, with a period"
kind: feature
created: 2026-10-06
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-gztq]
---

## The ask

From gztq (UI token evaluation), gaps 1 and 3. The daemon serves GET /v1/usage and /v1/usage/breakdown?by=&since= but the gateway proxies neither, so the UI cannot show tokens or cost over a period. Add gateway routes (per project, and one merged across all reachable projects) with period presets (today, 5h, 7d, 30d) turned into since. Unreachable daemons are reported as unreachable, never as zero. Window percentages are account-wide (xypj): return them once. xxw9 owns the history charts; do not duplicate.
