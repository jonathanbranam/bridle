+++
id = "br-n4w4"
title = "Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dalek's load to 76-144 and the load hold blocked spawns for ~26 h; br-3p3h fixed only the idle case"
kind = "incident"
state = "open"
created_at = "2026-10-09T01:01:23.208Z"
updated_at = "2026-10-09T01:41:32.122757Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "n4w4"
+++

docs/tickets/open/postmortem-bridle-s-own-ps-polling-every-daemon-test-daemons-n4w4.md

## Thread

### note · external:aide · 2026-10-09T01:13:13.742Z
From the human, via aide (2026-10-08 ~9:20 PM ET), on the n4w4 postmortem recommendations: "I said yes, approved." (Their first answer, "Yes, it's three", was a transcription error for that.) Approval covers all 9 recommendations in ticket n4w4.

### note · agent:pm-1 · 2026-10-09T01:41:03.474Z
split off br-6nzj: Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/context/incidents.md (n4w4 recs 1, 2, 9)

### note · agent:pm-1 · 2026-10-09T01:41:12.915Z
split off br-g76s: Load-hold notes: one per machine, name bridle-owned top consumers, honest text, load.hold.started/ended events, escalate a long hold (n4w4 recs 4, 5)

### note · agent:pm-1 · 2026-10-09T01:41:21.012Z
split off br-fzwa: Audit every periodic daemon loop for what it forks or reads per tick; list them with cost in daemon.md (n4w4 rec 3)

### note · agent:pm-1 · 2026-10-09T01:41:27.525Z
split off br-ks55: Only one full test run at a time per machine: just check takes a machine-wide lock (n4w4 rec 6)

### note · agent:pm-1 · 2026-10-09T01:41:32.122Z
split off br-yw8b: fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7)
