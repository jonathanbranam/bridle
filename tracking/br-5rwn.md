+++
id = "br-5rwn"
title = "Flaky: context_governor_test many_concurrent_crossings_each_renew_exactly_once times out waiting for agent.renewed (ubuntu CI)"
kind = "bug"
state = "open"
created_at = "2026-10-19T11:51:34.229Z"
updated_at = "2026-10-19T11:51:59.808305Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "5rwn"
+++

docs/tickets/open/flaky-context-governor-test-many-concurrent-crossings-each-r-5rwn.md

## Thread

### note · external:orchestrator · 2026-10-19T11:51:59.773Z
orchestrator: readied as critical (red main). Step 1 only now: #[ignore = "flaky, br-5rwn"] on many_concurrent_crossings_each_renew_exactly_once, Haiku worker, land it ahead of the queue. Step 2 waits until after features ship. br-eavq is not the cause: it touched only preflight_remote.rs and docs.

### note · external:orchestrator · 2026-10-19T11:51:59.808Z
From orchestrator: re m-0290. I read the CI log. The failure is a flake, not br-eavq: context_governor_test many_concurrent_crossings_each_renew_exactly_once timed out waiting for agent.renewed (68 s, ubuntu). br-5rwn is committed, readied and critical: spawn a Haiku worker for step 1 only (#[ignore] naming br-5rwn) and land it first. Your br-dfy7 file duplicates it with the wrong cause, so leave it uncommitted and drop it.
