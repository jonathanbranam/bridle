---
id: br-ar4d
title: "Flaky again (third time): governor_test working_agent_is_notified_then_stopped_when_its_turn_ends times out waiting for Stopped (macOS CI)"
kind: bug
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
tasks: [br-ar4d]
---

## The ask

Filed by the orchestrator, 2026-10-11 ~12:25 AM ET. Critical: it turned main red (the human, 2026-10-03: a flaky test that breaks CI on main is critical).

CI run https://github.com/jonathanbranam/bridle/actions/runs/38109590013 on 7cf05257 (a ticket-only commit), check (macos-latest): 1488/1489 passed, the only failure

    FAIL [60.935s] bridle-daemon::governor_test working_agent_is_notified_then_stopped_when_its_turn_ends
    panicked at crates/bridle-daemon/tests/support/mod.rs:385:13:
    timed out waiting for agent a-4ftar to reach Stopped

ubuntu-latest passed on the same commit.

Third flake of this test: jtn8, then n96z (br-n96z, 05b5d2ec), whose fix held the agent's turn open and ended it with an interrupt after the Usage pause notice. This time the wait that times out is the later one, for the agent to reach Stopped after its turn ends (HANG_GUARD_TIMEOUT, 60 s), so the notice part may be fine and the stop after the interrupt is what didn't come (or came too late on a loaded macOS runner).

Wanted: find why the agent never reached Stopped within 60 s (the interrupt not delivered, the turn end not seen, wind-down not re-checking after the turn ends, or just slow), and fix the cause; don't raise the timeout or add retries unless the cause is plain slowness, and say which it was. Check against the test both alone and under load (as n96z did).

See: docs/tickets/resolved/flaky-under-load-governor-test-working-agent-is-notified-the-n96z.md, docs/tickets/resolved/flaky-governor-test-working-agent-is-notified-then-stopped-w-jtn8.md.
