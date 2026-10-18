+++
id = "br-6c8x"
title = "Stages MVP 5: a worker builds a version 2 ticket from tasks.md (OpenSpec apply)"
kind = "feature"
state = "integrated"
created_at = "2026-10-17T21:41:30.146Z"
updated_at = "2026-10-18T06:16:09.299441Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w6c8x",
]
priority = "high"
priority_at = "2026-10-17T22:04:05.821253Z"
branch = "bridle/w6c8x"
commit = "bac33b9f386bdbfa6d991c0e4a96f2e252146f98"
summary = "Added rule workflow/base/rules/stage-apply.md (roles: worker; OpenSpec apply: read proposal/design/specs/tasks, first task merges specs.md into design/specs/, tick tasks.md in order, pause on blockers) and integrator::land refuses a version 2 ticket whose landed tasks.md has '- [ ]' items (unticked_tasks in crates/bridle-daemon/src/integrator.rs, unit-tested; version 1 and no-tasks.md untouched). Docs: stages.md, CHANGELOG. just check: exit 0, 1859 tests."
ticket = "6c8x"
+++

docs/tickets/open/stages-mvp-5-a-worker-builds-a-version-2-ticket-from-tasks-m-6c8x.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T22:03:58.821Z
The human, 2026-10-17 ~5:15 PM ET, to advisor (tj7q): "Yes approved. Let's go" (shared understanding of epic tj7q, and ready steps 1-5 so the build can start today; the human wants the MVP done today to use tomorrow). Readied by human via advisor (tj7q).

### note · external:advisor/tj7q · 2026-10-17T22:04:05.821Z
priority: normal -> high

### note · agent:w6c8x · 2026-10-18T04:49:30.871Z
built: 1b245be4 (branch bridle/w6c8x)

### note · agent:w6c8x · 2026-10-18T04:49:31.067Z
done: stage-apply worker rule + land check on unticked tasks.md (v2 only); just check exit 0, 1859 tests; 1b245be4

### note · agent:manager-2 · 2026-10-18T05:45:23.801Z
integrated: bac33b9f386bdbfa6d991c0e4a96f2e252146f98 (branch bridle/w6c8x)

### note · agent:manager-2 · 2026-10-18T05:45:48.311Z
cleanup: removed agent w6c8x, branch bridle/w6c8x

### note · external:advisor/product-manager · 2026-10-18T06:16:09.299Z
advisor product-manager: resolved. Verified on main (bac33b9f = worker's 1b245be4): rule stage-apply (worker, v2: proposal/design/specs/tasks, specs merge first, tick in order, pause on blockers) and the land check refusing unticked tasks.md (integrator.rs, tested).
