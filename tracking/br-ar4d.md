+++
id = "br-ar4d"
title = "Flaky again (third time): governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out waiting for Stopped (macOS CI)"
kind = "bug"
state = "claimed"
created_at = "2026-10-11T04:21:34.879Z"
updated_at = "2026-10-18T14:25:17.072483Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:war4d",
]
priority = "low"
priority_at = "2026-10-11T04:32:54.869707Z"
ticket = "ar4d"
+++

docs/tickets/open/flaky-again-third-time-governor-test-working-agent-is-notifi-ar4d.md

## Thread

### note · external:orchestrator · 2026-10-11T04:21:49.298Z
From orchestrator: main is RED: CI run 38109590013 (7cf05257, macos-latest only) failed on governor_test working_agent_is_notified_then_stopped_when_its_turn_ends, 'timed out waiting for agent a-4ftar to reach Stopped' (support/mod.rs:385, 60 s). Third flake of that test (jtn8, n96z); this time it's the wait for Stopped after the turn ends, not the notice. Critical: br-ar4d is filed, readied and planned (brief in ticket ar4d), Sonnet; give it the next free slot, after br-3k7d which is already running. Merge nothing else until main is green again: the CI runs on the later commits are in progress, and I'll tell you when one passes on macOS. br-3k7d may land once main is green.

### note · external:orchestrator · 2026-10-11T04:30:17.014Z
orchestrator: failed again on aa70230b (run 38110919272, macos-latest only, 'timed out waiting for agent a-xc7eq to reach Stopped'). Tonight it's 2 of the last 4 macOS runs on identical code (docs-only commits between), so it has got much more frequent since n96z landed 2026-10-10: worth checking what changed since then in the stop path after a turn ends (supervisor, governor wind-down, interrupt handling), e.g. git log 05b5d2ec..main -- crates/bridle-daemon/src.

### note · external:orchestrator · 2026-10-11T04:30:17.036Z
From orchestrator: CI red again on aa70230b, the same governor_test flake (br-ar4d), on code identical to the green 24b0c402. I'm not holding merges for this known flake: keep landing (br-3k7d first) as long as each landing's CI fails only on this test. Any other failure is a real red: stop and tell me. br-ar4d is now the top priority after br-3k7d: it's failing half the macOS runs (comment on the task).

### note · external:orchestrator · 2026-10-11T04:32:30.152Z
split off br-ck8a: Disable the flaky governor_test working_agent_is_notified_then_stopped_when_its_turn_ends (#[ignore], ar4d)

### note · external:orchestrator · 2026-10-11T04:32:45.341Z
orchestrator: no longer critical. The human, 2026-10-11 ~12:20 AM ET, via aide: flaky tests are ignored or disabled and the fix is scheduled by the PdM after features ship. Disabled by br-ck8a; this task goes to the PdM. Don't start a worker on it unless the PdM schedules it.

### note · external:advisor/product-manager · 2026-10-11T04:32:54.869Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-11T04:32:54.894Z
advisor (product-manager): priority low per the human (~12:20 AM ET via aide): flaky tests are disabled now (br-ck8a) and the real fix waits until after the features ship. Not tonight; I'll schedule it after the 22ab chain (br-wrmu, br-bpku, br-3v75, br-72t9).
