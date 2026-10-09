---
id: ngya
title: "Flaky on Linux CI: upgrade_test a_drain_starting_during_a_spawn_restarts_promptly reads messages after the daemon stopped serving"
kind: bug
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

## What happened

CI on main failed on f8b6b5b5 (a docs-only commit), ubuntu only:
https://github.com/jonathanbranam/bridle/actions/runs/37986250994

    FAIL bridle-daemon::upgrade_test a_drain_starting_during_a_spawn_restarts_promptly
    panicked at crates/bridle-daemon/tests/upgrade_test.rs:395:10:
    messages: Unreachable("error sending request for url (http://127.0.0.1:46533/v1/messages?to=a-2vjvd&...)")

The test was added by br-b6mu (3323d2f4) today.

## Diagnosis (orchestrator)

The test waits for `restart_requested()`, awaits the spawn, then calls `messages_to` over HTTP.
Once the restart is requested the daemon drains and shuts its HTTP server down, so the
`list_messages` call races the shutdown: on a fast or loaded runner the server is already gone.

## Fix

Make the check not depend on the server still being up after the restart is requested: read
the held message before the restart can complete, or read it from the store directly (the test
owns the temp dir), or assert the message was persisted by another route that does not race the
shutdown. Keep the timing assertion (the drain must not wait out the spawn's readiness wait).
Test-only change; no product code unless the diagnosis is wrong.

Acceptance: `just check` green; the test no longer makes an HTTP call after `restart_requested()`.
Critical: a flaky test on main is critical (the human, 2026-10-03).
