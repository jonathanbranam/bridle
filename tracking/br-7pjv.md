+++
id = "br-7pjv"
title = "Chore: review open tickets whose work looks already built, and resolve the done ones"
kind = "chore"
state = "built"
created_at = "2026-11-21T16:09:48.229Z"
updated_at = "2026-11-21T19:14:37.406542Z"
created_by = "external:advisor/agentic-dev"
watchers = [
    "external:advisor/agentic-dev",
    "agent:w7pjv",
]
branch = "bridle/w7pjv"
commit = "0a9002e9"
summary = "Reviewed 7 tickets. Resolved with a Resolution section: hrcn, t7ya, ervd, fpqy, essy, ggnp (built; caveat on ervd: base orchestrator role lacks the spin-off paragraph, project role has it). cvaq left open: quiet-hours noise routing, catch-up summary and override event unbuilt (comment on its row br-ff33). Files only; no code."
ticket = "7pjv"
+++

docs/tickets/open/stale-open-tickets-7pjv.md

## Thread

### note · external:advisor/agentic-dev · 2026-11-21T16:10:05.099Z
readied by external:advisor/agentic-dev

### note · external:advisor/agentic-dev · 2026-11-21T16:10:09.428Z
Readied by advisor agentic-dev (rule ready-without-asking: docs/chore kind, self-evident ask), at the human's request.

### note · external:advisor/agentic-dev · 2026-11-21T16:20:22.078Z
Wider backlog for context (advisor agentic-dev): bridle ticket check flags 135 of 350 files in docs/tickets/open/ as integrated but still open, and 8 files (dg5g, j4bk, 3stk, vynr, ar4d, t95y, za3j, wskt) sit in the wrong folder for their row. Out of this ticket's scope unless the PM widens it; the standing fix is proposed in the self-review ticket.

### note · agent:w7pjv · 2026-11-21T19:14:37.358Z
built: 0a9002e9 (branch bridle/w7pjv)

### note · agent:w7pjv · 2026-11-21T19:14:37.406Z
done: resolved hrcn, t7ya, ervd, fpqy, essy, ggnp (Resolution sections; ggnp file moved too); cvaq stays open (comment on its row br-ff33: quiet-noise routing, catch-up summary, override event unbuilt). ervd caveat: base orchestrator role lacks the spin-off paragraph (project role has it). Commit 0a9002e9. just check NOT green: two runs each failed one different timing test under machine load (run1 restart_land_test a_land_past_the_cap..., run2 restart_hold_test cancel_ends_an_upgrade_drain_quietly; 2024/2025 passed in run2, band ok vs 2012); both pass alone on rerun. Change is docs/tickets only; please judge or rerun the check at landing.
