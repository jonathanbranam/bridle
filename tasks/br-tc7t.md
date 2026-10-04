+++
id = "br-tc7t"
title = "A landing that needs a UI install or gateway restart to show says so, and someone does it"
kind = "bug"
state = "planned"
created_at = "2026-10-04T21:27:39.835Z"
updated_at = "2026-10-04T21:28:07.609868Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
priority = "high"
+++

original id: tc7t
Bridle-repo half of docs/tickets/open/a-landing-that-needs-a-ui-install-or-gateway-restart-to-show-tc7t.md (read it). The human chose full automation (option C). This task: the gateway restarts itself on a bridle upgrade, like the daemon (daemon.md 'Automatic upgrade': quiet point, in-place restart). It builds on br-bek3 (detached gateway plus its restart-on-upgrade spec), so if br-bek3 already delivered the restart, this task shrinks to verifying it end to end with a test and closing the gap in the docs; say so on the thread and finish. Files: crates/bridle-daemon (upgrade path), crates/bridle-gateway, daemon.md, human-web-ui.md. Acceptance: just check passes; a test that an upgrade restart also restarts a running gateway. Model: Sonnet. Out of scope: installing the UI into ~/.bridle/ui (the 'bridle-ui daemon builds green main like self_upgrade' half): that belongs to the bridle-ui project, not this repo; the orchestrator is raising it there.

## Thread

### note · external:orchestrator · 2026-10-04T21:27:43.699Z
Approval: the human, 2026-10-04 via aide (m-0140): "I want option C. All of this should be automated." Scope per aide: (1) the bridle-ui daemon installs the UI like self_upgrade, building green main into ~/.bridle/ui, no role involved; (2) the gateway restarts itself on a bridle upgrade like the daemon. Full quote in the ticket (928543b3).

### note · agent:pm-1 · 2026-10-04T21:28:07.609Z
priority: normal -> high
