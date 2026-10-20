+++
id = "br-sftn"
title = "CI red on main after br-b2g8: check (ubuntu-latest) fails on 9e2b4858"
kind = "fix"
state = "planned"
created_at = "2026-10-20T23:15:41.625Z"
updated_at = "2026-10-20T23:25:55.295197Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
ticket = "sftn"
+++

Critical: main is red (the human, 2026-10-03: anything that breaks CI on main is critical).

CI run https://github.com/jonathanbranam/bridle/actions/runs/39512201248 on 9e2b4858, check (ubuntu-latest). The only failure is the known flaky governor test, not br-b2g8's change:

    governor_test working_agent_is_notified_then_stopped_when_its_turn_ends
    panicked at crates/bridle-daemon/tests/support/mod.rs:393:13
    timed out waiting for agent a-2xysa to reach Stopped

Its fourth flake (jtn8, n96z, ar4d). Per the human's rule (2026-10-11: disable a flaky test and file a ticket for later), disable it: add `#[ignore = "flaky, br-remg"]` to that test in `crates/bridle-daemon/tests/governor_test.rs`, the way br-ck8a did. Nothing else. The real fix is br-remg, scheduled later.

Acceptance: `just check` green; then CI green on main. Model: Haiku.

## Thread

### note · external:orchestrator · 2026-10-20T23:16:50.537Z
orchestrator: rewritten as the #[ignore] disable (known flaky test, 4th time; fix is br-remg, later). Readied; critical, red main.
