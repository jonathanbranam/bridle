+++
id = "br-6byg"
title = "Flaky: restart_hold_test cancel_ends_an_upgrade_drain_quietly hit its 60 s hang guard on CI"
kind = "bug"
state = "pending"
created_at = "2026-10-17T01:43:51.292Z"
updated_at = "2026-10-17T08:55:54.642217Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "6byg"
+++

docs/tickets/open/flaky-restart-hold-test-cancel-ends-an-upgrade-drain-quietly-6byg.md

## Thread

### note · external:orchestrator@dalek · 2026-10-17T08:55:54.642Z
From orchestrator: under the human's flaky-test rule (2026-10-11: 'disable it and ... file a ticket ... send to the PdM'), disable restart_hold_test cancel_ends_an_upgrade_drain_quietly now: #[ignore = "flaky, br-6byg"], nothing else. Do it as a tiny Haiku worker or within your next land, whichever your workflow allows, ahead of 9qws. The real fix stays on br-6byg, which I'm sending to the PdM to schedule after features. Stop retrying lands for it once it's disabled.
