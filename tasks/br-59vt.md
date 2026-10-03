+++
id = "br-59vt"
title = "u6w9 c: gateway /api/v1/interactions/{report,day,hours,intervals} handlers"
kind = "feature"
state = "planned"
created_at = "2026-10-03T21:03:02.246Z"
updated_at = "2026-10-03T21:03:04.471543Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Part 3 of 3 of br-u6w9 (read it and docs/tickets/open/track-the-human-s-time-and-attention-spent-talking-to-agents-u6w9.md, section 'Design'). Human wants this built today. Needs tasks a (types) and b (store, intervals).
Build in crates/bridle-gateway: handlers returning task a's ts-rs types: /interactions/report?from&to&group=project|agent|machine&bucket=day|week; /interactions/day?date= (per-session intervals, concurrency peak and minutes at 1/2/3+); /interactions/hours?from&to&days=weekday|weekend|mon,...; raw intervals. Auth like the other /api/v1 routes. Tests: each endpoint on fixture data, bad params. Docs: human-web-ui.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: the UI (separate bridle-ui task), CLI reads.
