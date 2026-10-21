+++
id = "br-remg"
title = "Flaky a fourth time: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out waiting for Stopped (ubuntu CI)"
kind = "fix"
state = "built"
created_at = "2026-10-20T23:16:39.761Z"
updated_at = "2026-10-21T11:35:56.150244Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:remg",
]
branch = "bridle/remg"
commit = "c08b4e7f9ca140e5cac55c80be784e4cb1ebed52"
summary = "Cause (not slowness): transition_state in supervisor.rs guarded the in-memory state against finished->live moves (ar4d) but wrote the store after releasing the runtime lock. A racing stop's Stopping and finish_agent's Stopped could take the lock in the right order yet commit to the store in the wrong one, leaving the stored state Stopping for good, so the test waited the full 60 s. Fix: the store write now happens under the lock. Could not reproduce (6/6 alone before the fix; 17/17 x3 and the full daemon suite after), so the diagnosis is from code reading. #[ignore] removed."
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

### note · agent:remg · 2026-10-21T11:35:55.753Z
built: c08b4e7f9ca140e5cac55c80be784e4cb1ebed52 (branch bridle/remg)

### note · agent:remg · 2026-10-21T11:35:56.150Z
done: governor flake fixed (transition_state wrote the store after dropping the lock, so a racing stop's Stopping could land after Stopped; not slowness), #[ignore] removed; c08b4e7f9ca140e5cac55c80be784e4cb1ebed52; just check exit 0, 1959 tests (last full 1942)
