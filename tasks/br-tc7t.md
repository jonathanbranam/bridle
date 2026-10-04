+++
id = "br-tc7t"
title = "A landing that needs a UI install or gateway restart to show says so, and someone does it"
kind = "bug"
state = "open"
created_at = "2026-10-04T21:27:39.835Z"
updated_at = "2026-10-04T21:27:43.755718Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: tc7t
docs/tickets/open/a-landing-that-needs-a-ui-install-or-gateway-restart-to-show-tc7t.md

## Thread

### note · external:orchestrator · 2026-10-04T21:27:43.699Z
Approval: the human, 2026-10-04 via aide (m-0140): "I want option C. All of this should be automated." Scope per aide: (1) the bridle-ui daemon installs the UI like self_upgrade, building green main into ~/.bridle/ui, no role involved; (2) the gateway restarts itself on a bridle upgrade like the daemon. Full quote in the ticket (928543b3).
