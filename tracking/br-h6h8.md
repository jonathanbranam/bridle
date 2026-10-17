+++
id = "br-h6h8"
title = "A worker was spawned for a ticket inside its settle period, before it was planned (dotfiles-local dl-v27w)"
kind = "bug"
state = "built"
created_at = "2026-10-16T01:56:40.359Z"
updated_at = "2026-10-17T20:38:02.816551Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wh6h8",
]
branch = "bridle/wh6h8"
commit = "8f9e62f8"
summary = "The settle gate already existed (spawn_agent calls require_settled for any non-human --ticket spawn). The missing gate was state: a worker spawn naming a ticket not planned/claimed is now refused with the reason (server.rs spawn_agent); human exempt; test in settle_hold_test.rs; docs and CHANGELOG updated. The manager role already says always pass --ticket. Likely cause on dotfiles-local: spawn without --ticket, or an older daemon; not verifiable here."
ticket = "h6h8"
+++

docs/tickets/open/a-worker-was-spawned-for-a-ticket-inside-its-settle-period-b-h6h8.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T15:26:09.535Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wh6h8 · 2026-10-17T20:37:39.380Z
built: 8f9e62f8 (branch bridle/wh6h8)

### note · agent:wh6h8 · 2026-10-17T20:37:39.461Z
done: worker spawn --ticket refused unless planned/claimed (settle gate already existed); just check exit 0, 1829 tests passed, ran on pre-merge commit (main merge was docs only); 8f9e62f8

### note · external:orchestrator · 2026-10-17T20:38:02.816Z
orchestrator review: looks right. One edge, manager's call (not a blocker): a ticket in Built (sent back for rework whose worker is gone) is now refused a fresh worker. If that path is used, add TicketState::Built to the allowed set. Otherwise land as is.
