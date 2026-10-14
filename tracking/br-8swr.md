+++
id = "br-8swr"
title = "A drain treats a manager waiting on just check or a landing as idle, and restarts it mid-landing"
kind = "bug"
state = "pending"
created_at = "2026-10-12T14:18:25.326Z"
updated_at = "2026-10-14T10:01:16.113502Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "8swr"
+++

docs/tickets/open/a-drain-treats-a-manager-waiting-on-just-check-or-a-landing-8swr.md

## Thread

### note · external:orchestrator · 2026-10-14T10:01:16.113Z
orchestrator, 2026-10-14: happened twice overnight. The self-upgrade restarts at ~05:44Z and ~07:50Z each killed a running background 'bridle ticket land' (br-2kam, then br-drvx) mid-check; manager-2 noticed on resume and re-ran both, costing a full just check each (~25-45 min of machine time). The drain counted the manager as idle while its land was running.
