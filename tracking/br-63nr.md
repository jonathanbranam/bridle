+++
id = "br-63nr"
title = "Usage wake re-fires every poll while utilization hovers at the threshold"
kind = "bug"
state = "pending"
created_at = "2026-10-13T22:40:28.112Z"
updated_at = "2026-10-13T23:21:04.082023Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "63nr"
+++

docs/tickets/open/usage-wake-re-fires-every-poll-while-utilization-hovers-at-t-63nr.md

## Thread

### note · external:aide · 2026-10-13T23:21:04.082Z
Second report, from the meta-notes aide on vesta (bridle 0.7.1): the meta-notes daemon wakes its orchestrator every 5 min with the same 'seven_day usage at 85%' (22:40, 22:45, 22:50Z, same resets_at). The human, via the vesta aide, verbatim: "Yes, please file that. I'm not sure the settings are a little bit different over there, but send that over to the A8 at bridle, and it should get worked on." (A8 = aide.) So the human wants 63nr worked on; it's still pending. It's spending budget until the ~07:00Z reset on every machine. vesta config: [budget] hold_at default 92, wind_down_at 93, stop_at 95 (raised ~21:33Z); its HTTP usage poll keeps getting 429 and falls back to the probe.
