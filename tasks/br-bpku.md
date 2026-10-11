+++
id = "br-bpku"
title = "Every ticket gets a row; types fix and epic; readiness moves to the ticket (ticket ready)"
kind = "feature"
state = "planned"
created_at = "2026-10-09T19:47:21.257Z"
updated_at = "2026-10-11T04:46:47.216Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "bpku"
+++

Ticket: docs/tickets/open/every-ticket-gets-a-row-types-fix-and-epic-readiness-moves-t-bpku.md (read "The ask"; it is the scope). Also read ticket 22ab sections 2 and 3 (the parent plan, human approved).

Order: 22ab steps 1-4 go one at a time (all change the ticket tooling). Starts after br-wrmu (syqn part 3) is integrated; br-3v75 and br-72t9 follow this one. The human wants the epic landed or close by morning: do not wait on anything else.

Size: this is big. If the context nears 150K tokens, land a coherent first slice (e.g. rows for every ticket and the new types) and say on the thread what remains, so the manager can file the rest as a follow-up task; do not leave main red.
Migration: the ticket names it (bug tasks that are fixes become fix); implement it as an xebc migration, trial branches only for other projects (rule existing-projects). State how existing projects are updated in the done note.
Acceptance: just check passes, plus the ticket's items. Model: Sonnet.

## Thread

### note · external:advisor/product-manager · 2026-10-11T04:28:10.462Z
Readied by advisor (product-manager), under the human-approved 22ab plan (2026-10-09: step 2, no human gate, readied when buildable). Step 1's br-syqn is integrated; its parts br-epvy (done, landing) and br-wrmu (the migration) are still in flight. Plan it now so it starts right after br-wrmu lands: 22ab steps 1-4 go one at a time (they all change the ticket tooling). The human wants the epic to land or get close by morning.
