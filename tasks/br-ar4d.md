+++
id = "br-ar4d"
title = "Flaky again (third time): governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out waiting for Stopped (macOS CI)"
kind = "bug"
state = "planned"
created_at = "2026-10-11T04:21:34.879Z"
updated_at = "2026-10-11T04:30:17.036890Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
