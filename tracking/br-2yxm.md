+++
id = "br-2yxm"
title = "Self-upgrade starves while the manager lands back to back: each land's --postpone 45m pushes the upgrade out again"
kind = "bug"
state = "open"
created_at = "2026-10-19T23:51:08.165Z"
updated_at = "2026-10-20T00:06:26.475605Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "2yxm"
+++

docs/tickets/open/self-upgrade-starves-while-the-manager-lands-back-to-back-ea-2yxm.md

submitted by external:orchestrator

Seen 2026-10-19 ~7-8 PM ET: green-CI upgrade postponed for over an hour (19:08, 19:27, 19:46, 20:06) because manager-2 runs 'bridle daemon restart --postpone 45m' before every land (its handover routine) and lands came every ~20 min. A human was waiting on a landed fix (br-ggbx) going live. Fix idea: postpone only covers the land in progress (ends when the land ends), or a pending upgrade gets a window between lands before the next claim. Workaround used: orchestrator told the manager to pause new work after one land and run the upgrade.

## Thread

### note · external:orchestrator · 2026-10-19T23:51:08.165Z
submitted by external:orchestrator
