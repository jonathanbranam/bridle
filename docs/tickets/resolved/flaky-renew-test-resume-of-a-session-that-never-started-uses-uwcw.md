---
id: uwcw
title: "Flaky renew_test: resume_of_a_session_that_never_started_uses_a_fresh_session (macOS)"
kind: fix
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-17T11:37:14Z
---

## What happened

CI on `main` failed on macOS at 0c7257ad (run 38115575191, 2026-10-11 05:54Z), Linux green:

    bridle-daemon::renew_test resume_of_a_session_that_never_started_uses_a_fresh_session
    panicked at crates/bridle-daemon/tests/renew_test.rs:377:5
    assertion `left == right` failed
      left: "c7f5621f-a308-4e0e-8cad-f8a09e5b4cdb"
     right: "154af8ce-bf17-4955-9101-1a4d466050da"

The commit (br-epvy, project-qualified ticket IDs) doesn't touch resume or sessions; the test had
not failed in the previous runs on `main`. 1495 of 1496 passed.

## Likely cause (unverified)

Line 377 is the second half of the test: after the fresh-session resume, it waits for `Idle`, stops,
and resumes again, expecting the same session id (the session now "exists" because its first turn
started). The second resume minted a new id, so the daemon still thought the session had never
started a turn. A race between the turn-started record (p4ks, 0dc13324) and `Idle`/stop on a slow
macOS runner seems most likely: the test may reach `Idle` before the "turn started" fact is stored.

## The ask

Make the test (or the code, if the race is real for users) deterministic. If the daemon can really
lose "this session started a turn" on a quick stop, that is a real bug: a resumed agent would lose
its conversation.

Per the human's rule on flaky tests (2026-10-11), scheduled by the PdM after features ship.
