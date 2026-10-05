---
id: ksz7
title: cli_e2e tests time out waiting for daemon.json under load (60 s), failing landing checks
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-ksz7]
---

## The ask

Make `bridle::cli_e2e`'s daemon start-up wait robust under load, so a landing check that runs alongside a worker's `just check` doesn't fail.

## What happened

manager-2, 2026-10-05 03:42Z (m-5339): six `bridle::cli_e2e` tests failed with "timed out waiting for .../.bridle/daemon.json" (cli_e2e.rs:83, ~61 s) in two landing checks (br-e9yu, br-yfjc). Both times a worker's `just check` was running at once; e9yu passed on retry. Not seen in CI so far.

## Why it matters

Each failed land check costs ~10 minutes and a retry. With two workers and landings overlapping all night, it happens often. Find what the start-up waits on under load (a build in the test? a fixed 60 s?) and fix that, rather than only raising the timeout.
