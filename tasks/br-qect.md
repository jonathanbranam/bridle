+++
id = "br-qect"
title = "Postmortem for incident br-y455: bridle's daemon couldn't restart or self-upgrade for ~23 h (stuck spawning flag)"
kind = "research"
state = "open"
created_at = "2026-10-06T22:13:54.531Z"
updated_at = "2026-10-06T22:13:54.902098Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: y455
Read-only investigation, then write the postmortem into ticket y455 (docs/tickets/open/incident-bridle-s-daemon-couldn-t-restart-or-self-upgrade-fo-y455.md): timeline, root cause with the exact code path that leaves manager.spawning() true (reproduce it in a failing test on your branch; no fix), why ~23 h of refused upgrades raised no alert, and recommendations. Answer every question in the ticket's 'The postmortem must answer'. Sources: daemon events (bridle events), the daemon log, crates/bridle-daemon (the restart wait and the agent manager's spawning flag), br-btdn, q7mv. Don't restart or change the daemon. Done = the postmortem committed in the ticket on your branch.
