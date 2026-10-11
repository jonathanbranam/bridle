+++
id = "br-n4w4"
title = "Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dalek's load to 76-144 and the load hold blocked spawns for ~26 h; br-3p3h fixed only the idle case"
kind = "incident"
state = "dropped"
created_at = "2026-10-09T01:01:23.208Z"
updated_at = "2026-10-11T02:17:29.219500Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
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

### note · system · 2026-10-09T05:53:37.744Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · external:advisor/product-manager · 2026-10-09T11:04:40.385Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T23:05:47.023Z
advisor (product-manager): recs 4 and 5 landed in br-g76s (3101ce1d), except the 'critical task's spawn refused' trigger (SpawnRequest carries no task id). Left out on purpose for now: an hour-long hold already escalates. Ticket stays open until the other recs (6-9) are checked.

### note · external:advisor/product-manager · 2026-10-11T00:46:14.800Z
advisor (product-manager): rec 3 landed in br-fzwa (1caf9444): loop audit in daemon.md; sessions tick now reads the process table once. Idle forks left as found: CI + self-upgrade loop (gh/git every 60 s per daemon), origin-divergence git fetch every 10 min, load watch sysctl every 30 s, orchestrator supervisor ps read every 10 s. Not filed: no load trouble reported since 3p3h/9z2n; revisit if a hold recurs.

### note · external:advisor/product-manager · 2026-10-11T01:51:10.115Z
advisor (product-manager): all 9 recs done: 1 br-9z2n + br-6nzj; 2 br-6nzj (resource-budget test); 3 br-fzwa; 4-5 br-tnyt + br-g76s (critical-task trigger not built, see above); 6 br-ks55; 7 br-yw8b; 8 bridle-ui and track-web daemons restarted since (all daemons came up together, after a1bde105); 9 incidents.md entry. Resolving the ticket.

### note · external:orchestrator · 2026-10-11T02:17:29.219Z
dropped: orchestrator, at the PdM's ask (m-9836): postmortem record only; all 9 recommendations landed as their own tasks; ticket n4w4 resolved (c050f50f).
