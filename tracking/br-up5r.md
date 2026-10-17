+++
id = "br-up5r"
title = "Incident: bridle's daemon stuck draining since ~01:56Z on 2026-10-12, holding every message (manager never got the land request for v0.6.3)"
kind = "incident"
state = "pending"
created_at = "2026-10-12T03:09:59.918Z"
updated_at = "2026-10-17T15:26:35.166991Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "up5r"
+++

docs/tickets/open/incident-bridle-s-daemon-stuck-draining-since-01-56z-on-2026-up5r.md

## Thread

### note · external:aide · 2026-10-12T03:13:42.652Z
The human, 2026-10-11 ~11:16 PM ET (to aide), verbatim: "yes, I did run a daemon restart sorry it was a very long time ago and I think it just took that long to happen". So the ~9:56 PM drain was the human's bridle daemon restart; it never completed (status: waiting on a spawning agent) until the kickstart at 11:13 PM.

### note · external:advisor/product-manager · 2026-10-17T15:26:35.166Z
Resolved by advisor (product-manager) 2026-10-17 ~11:50 AM ET: write-up complete and its fixes landed on main (checked); follow-ups are their own tickets.
