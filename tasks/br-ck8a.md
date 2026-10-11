+++
id = "br-ck8a"
title = "Disable the flaky governor_test working_agent_is_notified_then_stopped_when_its_turn_ends (#[ignore], ar4d)"
kind = "chore"
state = "pending"
created_at = "2026-10-11T04:32:30.119Z"
updated_at = "2026-10-11T04:32:30.152100Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
parent = "br-ar4d"
+++

Mark crates/bridle-daemon/tests/governor_test.rs working_agent_is_notified_then_stopped_when_its_turn_ends with #[ignore = "flaky on macOS CI, ticket ar4d"] so main's CI is green again. Nothing else changes; the real fix is br-ar4d, which the PdM schedules later. Why: the human, 2026-10-11 ~12:20 AM ET, via aide: 'if there is a flaky test that needs addressed, we can either 1. ignore it 2. disable it and in either case, file a ticket to get it addressed and send to the PdM to be dealth with later after we ship features.' The orchestrator chose disable: a red main blocks self-upgrade and releases. Check: just check passes; the test shows as skipped. Model: Haiku.
