---
id: br-remg
title: "Flaky a fourth time: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out waiting for Stopped (ubuntu CI)"
kind: fix
created: 2026-10-20
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Fix later, after features ship (the human, 2026-10-11: flaky tests get disabled and their fix goes "to the PdM to be dealt with later after we ship features"). Queue it at the back.

CI run https://github.com/jonathanbranam/bridle/actions/runs/39512201248 on 9e2b4858, check (ubuntu-latest), the only failure:

    governor_test working_agent_is_notified_then_stopped_when_its_turn_ends
    panicked at crates/bridle-daemon/tests/support/mod.rs:393:13
    timed out waiting for agent a-2xysa to reach Stopped

Fourth flake: jtn8, n96z, ar4d (macOS), now ubuntu; ar4d's fix did not hold. br-sftn disables the test (#[ignore] naming this ticket). Find why the agent does not reach Stopped within HANG_GUARD_TIMEOUT after its turn ends, and fix the cause (not a longer timeout unless it is plain slowness; say which it was). Also covers the ignored governor test of br-5rwn step 2 if it is the same one.

Check: `just check`; run the test alone and under load, then remove the #[ignore].

Model: Sonnet. Files: crates/bridle-daemon/tests/governor_test.rs, tests/support/mod.rs, the governor code. Related: br-ar4d, br-n96z, br-jtn8, br-sftn, br-5rwn.
