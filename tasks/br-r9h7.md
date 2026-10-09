+++
id = "br-r9h7"
title = "Flaky on macOS CI: spawn_with_prompt_waits_for_the_turn_to_start_before_returning sees Idle, not Working"
kind = "bug"
state = "planned"
created_at = "2026-10-09T10:47:12.713Z"
updated_at = "2026-10-09T11:04:41.066009Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "r9h7"
+++

CRITICAL (the human, 2026-10-03: anything that breaks CI on main, a flaky test included). Ticket: docs/tickets/open/flaky-on-macos-ci-spawn-with-prompt-waits-for-the-turn-to-st-r9h7.md (has the CI run link and failure).

Failure: crates/bridle-daemon/tests/spawn_messaging_test.rs:259 `assert_eq!(agent.state, AgentState::Working)` in spawn_with_prompt_waits_for_the_turn_to_start_before_returning gets Idle on a fast runner, because the fake's turn can start AND finish before the spawn response is built.

What the test means to check: spawn with a prompt does not return before the turn has started. Fix without weakening that and without sleeps or retries: replace the line-259 assertion with one that is true in both orders, i.e. the state is Working OR (Idle with the TURN_STARTED event already recorded); the TURN_STARTED event query that follows (lines 261-270) already proves the start was recorded before spawn returned, so keep and tighten it: assert that the event list for the agent contains exactly one TURN_STARTED right after `spawn` returns (no waiting), and that its timestamp is not later than the response. If the fake can hold the turn open until released (look at crates/bridle-claude/tests/fake-claude.py for a directive such as SLEEP N, as used in upgrade_test.rs: `SLEEP 5` prompt), the more deterministic variant is to give the prompt a SLEEP directive so the turn is certainly still running, and keep `assert_eq!(state, Working)`; prefer this if the test's other assertions don't depend on the prompt text "hello there" (read lines 270 to the end of the test). Pick one and say which on the thread.
Then read the rest of spawn_messaging_test.rs and other tests in crates/bridle-daemon/tests for the same pattern (an immediate assertion of Working after spawn or send with the default fast fake) and fix any you find the same way; list them on the thread.

Files: crates/bridle-daemon/tests/spawn_messaging_test.rs, other test files only if the same pattern is found. No daemon code change.

Acceptance: just check passes; run `cargo nextest run -p bridle-daemon -E 'test(spawn_with_prompt_waits)'` 30 times in a shell loop and report 30 of 30 (the machine is loaded: that is the point). Model: Sonnet. Migration: none. Out of scope: other flaky tests (list them on the thread, do not fix).

## Thread

### note · external:orchestrator · 2026-10-09T10:48:10.956Z
settle skipped by external:orchestrator: critical: main is red on macOS (CI run 37918335201); the human treats any red main as critical

### note · external:advisor/product-manager · 2026-10-09T11:04:41.066Z
watching the task
