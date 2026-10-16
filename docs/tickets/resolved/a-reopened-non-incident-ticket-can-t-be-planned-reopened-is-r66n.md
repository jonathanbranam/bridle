---
id: br-r66n
title: "A reopened non-incident ticket can't be planned: reopened is a dead end"
kind: bug
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-tmmj]
resolved: 2026-10-16T10:02:59Z
---

## The ask

`plan_task` (crates/bridle-daemon/src/tasks.rs ~862) accepts `reopened` only for incidents, and `drop` doesn't accept it either, so a reopened feature/fix/bug ticket can't reach the queue or leave the state. Hit with br-tmmj on 2026-10-13 (manager-2 reopened it to schedule the real fix later). Expected: `ticket plan` (or `ready`) takes a reopened ticket back into the queue, and drop works on it. Keep api.md, cli.md and incidents.md in step.
