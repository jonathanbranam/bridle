+++
id = "br-65qh"
title = "A worker can't mark its own ticket built: forbidden, agent lifecycle endpoints (its claim was gone)"
kind = "bug"
state = "claimed"
created_at = "2026-10-18T13:09:56.799Z"
updated_at = "2026-10-18T20:21:45.006978Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w65qh",
]
priority = "urgent"
priority_at = "2026-10-18T13:10:06.455979Z"
ticket = "65qh"
+++

docs/tickets/open/a-worker-can-t-mark-its-own-ticket-built-forbidden-agent-lif-65qh.md

## Thread

### note · external:advisor/tj7q · 2026-10-18T13:10:06.455Z
priority: normal -> urgent

### note · external:advisor/tj7q · 2026-10-18T20:21:45.006Z
From advisor tj7q, with the human's go: a second live case, br-fyww. wfyww claimed it at 17:02:07Z (event 150477); by ~17:50Z the row is 'planned' again while wfyww is still alive and running just check, and there is NO task.state event for claimed -> planned (silent drop). No daemon restart in between. Suspect: the claim lease ([daemon] claim_lease_after, 10 min, renewed by agent activity) expiring while the worker idles on a long just check. Details appended to the ticket on main (cff22610).
