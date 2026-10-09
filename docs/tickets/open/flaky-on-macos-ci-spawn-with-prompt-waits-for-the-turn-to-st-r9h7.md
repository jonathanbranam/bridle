---
id: r9h7
title: "Flaky on macOS CI: spawn_with_prompt_waits_for_the_turn_to_start_before_returning sees Idle, not Working"
kind: bug
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-r9h7]
---

## The ask

Make the test deterministic without weakening what it checks, e.g. assert that the turn-start event was recorded before spawn returned (or that the state is Working or a finished turn already counted), or have the fake hold the turn open until the test releases it. Don't add sleeps or retries. Check the rest of spawn_messaging_test for the same pattern.

Critical: main is red until it lands (the human, 2026-10-03: anything that breaks CI on main, a flaky test included, is critical).

## What happened

CI run https://github.com/jonathanbranam/bridle/actions/runs/37918335201 on main b1c37432 (br-jxaf merge, 2026-10-09 10:45Z): macOS failed, Linux passed. 1373 of 1374 passed; the one failure:

```
bridle-daemon::spawn_messaging_test spawn_with_prompt_waits_for_the_turn_to_start_before_returning
panicked at crates/bridle-daemon/tests/spawn_messaging_test.rs:259:5
assertion `left == right` failed
  left: Idle
 right: Working
```

br-jxaf only touched the Tailscale re-check, so it is unrelated: this is a timing race in the test.

## Likely cause

The test asserts the state in the spawn response is `Working`. With the fake claude the turn starts and can also finish before the response is built, so a fast or loaded runner can return `Idle`. What the test means to check is that spawn waits until the turn has *started*, not that the turn is still running.
