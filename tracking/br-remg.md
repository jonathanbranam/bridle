+++
id = "br-remg"
title = "Flaky a fourth time: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out waiting for Stopped (ubuntu CI)"
kind = "fix"
state = "planned"
created_at = "2026-10-20T23:16:39.761Z"
updated_at = "2026-10-20T23:45:03.549579Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "remg"
+++

docs/tickets/open/flaky-a-fourth-time-governor-test-working-agent-is-notified-remg.md

submitted by external:orchestrator

## The ask

Filed by the orchestrator, 2026-10-20 ~7:20 PM ET. Fix later, after features ship (the human, 2026-10-11: flaky tests get disabled and their fix goes "to the PdM to be dealth with later after we ship features").

CI run https://github.com/jonathanbranam/bridle/actions/runs/39512201248 on 9e2b4858 (br-b2g8, a test-only change elsewhere), check (ubuntu-latest), the only failure:

    governor_test working_agent_is_notified_then_stopped_when_its_turn_ends
    panicked at crates/bridle-daemon/tests/support/mod.rs:393:13
    timed out waiting for agent a-2xysa to reach Stopped

Fourth flake: jtn8, n96z, ar4d (macOS), now ubuntu. ar4d's fix did not hold. br-sftn disables the test (#[ignore] naming this ticket). Find why the agent doesn't reach Stopped within HANG_GUARD_TIMEOUT after its turn ends, and fix the cause (not a longer timeout unless it is plain slowness; say which it was). Check it alone and under load, then remove the #[ignore].

Model: Sonnet. Related: br-ar4d, br-n96z, br-jtn8, br-sftn.

## Thread

### note · external:orchestrator · 2026-10-20T23:16:39.762Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-20T23:17:06.046Z
orchestrator: readied (fix; back of queue per the flaky-test rule).
