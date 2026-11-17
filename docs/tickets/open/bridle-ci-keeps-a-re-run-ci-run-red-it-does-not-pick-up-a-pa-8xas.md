---
id: br-8xas
title: "bridle ci keeps a re-run CI run red: it does not pick up a passing re-run attempt"
kind: bug
created: 2026-11-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

When a CI run on the integration branch is re-run (`gh run rerun <id> --failed`) and the re-run passes, `bridle ci` (and the land gate that reads it) reports the run as green. Today it keeps the first attempt's `failure` and blocks landing until a new commit lands.

## What happened (bridle, dalek, 2026-11-17)

- Run 44114639704 on main cd63f527 (br-rk4z): `check (macos-latest)` was cancelled after 15m without a hosted runner (zero steps); `check (ubuntu-latest)` passed. The `ci_failed` wake fired at 15:43 UTC.
- The orchestrator re-ran the failed job at ~16:55 UTC; `gh run view 44114639704` showed both jobs `success`, run conclusion `success`, within the hour.
- At 17:49 UTC `bridle ci` still printed `cd63f527 failure 126m ago .../runs/44114639704` and "error: CI is failure". manager-2 stayed blocked from landing br-ggnp (rule never-land-on-red) until the orchestrator confirmed green by hand.

Impact: a passing re-run doesn't unblock landing; every runner hiccup needs a human or orchestrator to check GitHub and override.

## Likely area

The daemon's GitHub CI poller probably records a run once (by run id, or on the first `completed` event) and doesn't refresh it when a new attempt (`run_attempt` > 1) completes. Check the poller in the daemon's CI code.

## Acceptance

A re-run that passes flips `bridle ci` to success for that commit (the next poll is fine); a test covers a run whose attempt 1 failed and attempt 2 passed.

Model: Sonnet (daemon poller).
