+++
id = "br-wskt"
title = "manager-2 goes idle with built work and free slots: it misses 'built' notes and doesn't refill slots"
kind = "bug"
state = "claimed"
created_at = "2026-10-18T03:10:10.581Z"
updated_at = "2026-10-18T06:16:13.775686Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wwskt",
]
ticket = "wskt"
+++

docs/tickets/open/manager-2-goes-idle-with-built-work-and-free-slots-it-misses-wskt.md

## Thread

### note · external:orchestrator · 2026-10-18T03:11:08.939Z
orchestrator, 03:12Z: likely part of the cause. The daemon is draining for self-upgrade 6457a600, postponed until 03:32Z (manager-2 itself ran 'bridle daemon restart --postpone 45m' before landing ussh). While draining, messages to agents stay 'arrived', never delivered: my m-16241 and m-16245 to manager-2 sit undelivered and manager-2 has had no turn since 03:09Z. So a manager that postpones an upgrade to finish a land then cannot hear 'built' notes or nudges for the rest of the postponement. Compare the 2026-10-12 01:56Z incident (br-up5r). Fix direction: a postponed drain should not hold message delivery to idle agents, or a land's postpone should end when the land does.
