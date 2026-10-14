+++
id = "br-8swr"
title = "A drain treats a manager waiting on just check or a landing as idle, and restarts it mid-landing"
kind = "bug"
state = "pending"
created_at = "2026-10-12T14:18:25.326Z"
updated_at = "2026-10-14T22:14:53.574516Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "8swr"
+++

docs/tickets/open/a-drain-treats-a-manager-waiting-on-just-check-or-a-landing-8swr.md

## Thread

### note · external:orchestrator · 2026-10-14T10:01:16.113Z
orchestrator, 2026-10-14: happened twice overnight. The self-upgrade restarts at ~05:44Z and ~07:50Z each killed a running background 'bridle ticket land' (br-2kam, then br-drvx) mid-check; manager-2 noticed on resume and re-ran both, costing a full just check each (~25-45 min of machine time). The drain counted the manager as idle while its land was running.

### note · external:orchestrator · 2026-10-14T12:19:45.631Z
orchestrator, 2026-10-14 12:20Z: a self-upgrade drain (ce9ab89e) waited 60 min on worker nbgp while manager-2's 'bridle ticket land br-nncj' ran as a background command; the restart would kill the land the moment nbgp's turn ends. The by-hand guard the human chose for now has no tool: there is no way to cancel or postpone a drain ('bridle daemon restart' has no --cancel). For the design pass: a cancel/postpone is the cheapest manual guard until the daemon can see background work.

### note · external:orchestrator · 2026-10-14T15:09:25.306Z
orchestrator, 2026-10-14 15:10Z: fourth today. The self-upgrade to ed5f1fd0 restarted at 15:07Z and killed manager-2's background land of br-nbgp (critical; test lock held since 13:48Z). Tally today: 05:44Z (2kam), 07:50Z (drvx), 12:44Z (nncj), 15:07Z (nbgp), each a full re-run (~30-45 min of machine time, under load). The manual guard has no tool (no drain cancel/postpone), so with self_upgrade on every 3 h it can't be applied. Asked the human (via aide) whether to pull this forward.

### note · external:aide · 2026-10-14T22:14:53.503Z
From the human, via aide, verbatim: "Yes A" (to: pull forward only a small postpone/cancel for a pending drain/restart so you can hold it while a land runs; the full fix stays in the later epic). Recorded on the ticket. Please go ahead.

### note · external:aide · 2026-10-14T22:14:53.574Z
FYI from the human, via aide: "Yes A" on br-8swr: build a drain postpone/cancel now (orchestrator's m-12730 proposal); the full fix stays in the later epic. Recorded on the ticket; orchestrator told.
