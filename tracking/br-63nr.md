+++
id = "br-63nr"
title = "Usage wake re-fires every poll while utilization hovers at the threshold"
kind = "bug"
state = "integrated"
created_at = "2026-10-13T22:40:28.112Z"
updated_at = "2026-10-14T01:35:17.325613Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:63nr",
]
branch = "bridle/63nr"
commit = "b1ba315deb1857d640c87f2c5adbbefa7343fc29"
summary = "check_usage keeps usage_fired as window -> resets_at; re-arms when utilization falls below limit - 0.03, or resets_at moves later by over 1 hour (sub-second jitter in resets_at does not count). Tests: hovering fires once, drop re-fires, new window re-fires, jittering resets_at stays silent. Docs and CHANGELOG updated. vesta's 429 poll not investigated."
ticket = "63nr"
+++

docs/tickets/open/usage-wake-re-fires-every-poll-while-utilization-hovers-at-t-63nr.md

## Thread

### note · external:aide · 2026-10-13T23:21:04.082Z
Second report, from the meta-notes aide on vesta (bridle 0.7.1): the meta-notes daemon wakes its orchestrator every 5 min with the same 'seven_day usage at 85%' (22:40, 22:45, 22:50Z, same resets_at). The human, via the vesta aide, verbatim: "Yes, please file that. I'm not sure the settings are a little bit different over there, but send that over to the A8 at bridle, and it should get worked on." (A8 = aide.) So the human wants 63nr worked on; it's still pending. It's spending budget until the ~07:00Z reset on every machine. vesta config: [budget] hold_at default 92, wind_down_at 93, stop_at 95 (raised ~21:33Z); its HTTP usage poll keeps getting 429 and falls back to the probe.

### note · agent:pm-1 · 2026-10-13T23:21:32.078Z
Brief (pm-1). Model: Sonnet. Size: S. URGENT: top tier (it burns orchestrator wakes on every machine until the 07:00Z reset). The ask is the spec. Fix `check_usage` in crates/bridle-daemon/src/wake.rs (~500): re-arm `usage_fired` only after utilization falls below limit minus a margin (0.03, named constant), or when the window's resets_at changes; whichever is simpler to test, but cover a real reset too (a new window must fire again). Tests: an oscillating reading (0.849/0.850/0.849...) fires once; a drop below the margin then a new crossing fires again; a resets_at change re-arms. Update the wake-table line in docs/design/agent-host/orchestrator-supervision.md and CHANGELOG.md. Migration: none. Out of scope: other wake kinds, the thresholds themselves.

### note · agent:63nr · 2026-10-14T00:07:02.615Z
built: 4dcbd06c54f94da93ac982e087a30beb98c455b3 (branch bridle/63nr)

### note · agent:63nr · 2026-10-14T00:07:02.718Z
done: usage wake hysteresis (0.03 margin + resets_at re-arm); check exit 0, 1649 tests passed; 4dcbd06c54f94da93ac982e087a30beb98c455b3

### note · agent:63nr · 2026-10-14T00:54:12.407Z
done: main merged, resets_at jitter fix (new window only if >1h later) per orchestrator review; check exit 0, 1651 tests passed; 47b8e3c19ae4ebe83f2f7bc14f5f5041495f7288

### note · agent:63nr · 2026-10-14T01:15:49.700Z
done: main (pemk) merged; check exit 0, 1653 tests passed; a534f40c0826166618e2846e709bfa057e5107d0

### note · agent:manager-2 · 2026-10-14T01:34:28.665Z
integrated: b1ba315deb1857d640c87f2c5adbbefa7343fc29 (branch bridle/63nr)

### note · agent:manager-2 · 2026-10-14T01:35:17.325Z
cleanup: removed agent 63nr, branch bridle/63nr
