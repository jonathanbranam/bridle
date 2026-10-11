+++
id = "br-ck8a"
title = "Disable the flaky governor_test working_agent_is_notified_then_stopped_when_its_turn_ends (#[ignore], ar4d)"
kind = "chore"
state = "integrated"
created_at = "2026-10-11T04:32:30.119Z"
updated_at = "2026-10-11T05:17:18.183173Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/wck8a"
commit = "b89a6666e1594d644b406f79706a43faaab87dfc"
summary = 'Added #[ignore = "flaky on macOS CI, ticket ar4d"] to governor_test working_agent_is_notified_then_stopped_when_its_turn_ends (crates/bridle-daemon/tests/governor_test.rs). Nothing else changed. Merged local main (cb7f3a28, includes br-3k7d). Real fix is ar4d, deferred by the PdM. just check on 1d38a129: exit 0, 1490 run, 1490 passed, 6 skipped.'
parent = "br-ar4d"
+++

Mark crates/bridle-daemon/tests/governor_test.rs working_agent_is_notified_then_stopped_when_its_turn_ends with #[ignore = "flaky on macOS CI, ticket ar4d"] so main's CI is green again. Nothing else changes; the real fix is br-ar4d, which the PdM schedules later. Why: the human, 2026-10-11 ~12:20 AM ET, via aide: 'if there is a flaky test that needs addressed, we can either 1. ignore it 2. disable it and in either case, file a ticket to get it addressed and send to the PdM to be dealth with later after we ship features.' The orchestrator chose disable: a red main blocks self-upgrade and releases. Check: just check passes; the test shows as skipped. Model: Haiku.

## Thread

### note · external:orchestrator · 2026-10-11T04:32:45.373Z
From orchestrator: change of plan from the human (quote on br-ck8a): flaky tests get disabled, the real fix waits for the PdM. So: DON'T start br-ar4d. Instead br-ck8a (one #[ignore] line, Haiku) takes the next free slot after br-3k7d; land it promptly so main goes green and self-upgrade can take br-3k7d. Merge policy until then as before: landings whose CI fails only on that test are fine.

### note · agent:wck8a · 2026-10-11T05:17:13.537Z
done: governor_test ignored (ar4d), main merged, just check exit 0 (1490 run, 6 skipped); 1d38a129

### note · agent:manager-2 · 2026-10-11T05:17:18.183Z
integrated: b89a6666e1594d644b406f79706a43faaab87dfc (branch bridle/wck8a)
