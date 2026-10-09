---
id: vabu
title: "Flaky on Linux CI: store cancelled_blocking_task_returns_shutting_down_not_a_panic"
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

## The ask

Make `store::tests::cancelled_blocking_task_returns_shutting_down_not_a_panic`
(`crates/bridle-daemon/src/store.rs`, ~line 3955) deterministic. Test-only; Haiku.

## What happened

CI run 37962490045 on 233e0c96 (Linux) failed: `aborted task should error: 1` at
`store.rs:3966`. The same code passed on the two runs before it; macOS passed.

## Cause

The test says it "caps the blocking pool at one thread", but
`#[tokio::test(flavor = "multi_thread", worker_threads = 1)]` sets only the worker threads;
`max_blocking_threads` stays at its default (512). So `spawn_blocking(|| 1)` isn't queued
behind `occupy`: it gets its own thread and can finish before `handle.abort()`, and the await
returns `Ok(1)`.

## Fix

Build the runtime in the test with
`tokio::runtime::Builder::new_multi_thread().worker_threads(1).max_blocking_threads(1).enable_all().build()`
and `block_on` the body, so the second task really is queued when it's aborted. Verify by
looping the test (e.g. 200 runs) and `just check`.
