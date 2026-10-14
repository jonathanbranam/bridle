---
id: br-t95y
title: "Flaky on ubuntu CI: gateway_test a_replaced_binary_is_re_executed exits 1 instead of re-executing"
kind: fix
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

## What happened

CI run https://github.com/jonathanbranam/bridle/actions/runs/38654228501 (main, 8edb2bdc "Release 0.7.3", 2026-10-14 23:35Z) failed on ubuntu-latest only:

    thread 'a_replaced_binary_is_re_executed' panicked at crates/bridle/tests/gateway_test.rs:238:5:
    the fake ran in place of the gateway: exit status: 1

The next commit (8f1a787f, a docs-only change on top) passed on both runners with the same code, so this is intermittent. The `bridle gateway` child exited 1 before (or instead of) re-executing the replaced binary. Its stderr goes to /dev/null in the test, so the cause isn't in the log.

History of this test: br-8a53 (a real bug: INVOCATION_ID misdetection, fixed) and y55w (hung behind syspolicyd on macOS). This failure is neither of those.

## Ask

1. Now (flaky-test rule, the human 2026-10-11): mark the test `#[ignore = "flaky on ubuntu CI, br-t95y"]` so main stays green.
2. Later, scheduled by the PdM: find the cause and re-enable. Suspects: the port the test picks is taken or reused between the free-port probe and the gateway's bind (exit 1 from a bind error), or the binary-change check fires during start-up before the gateway listens. Capturing the child's stderr in the test, instead of sending it to null, would show which.

Verify: the test passes 50 runs in a row on Linux (`cargo nextest run -p bridle --test gateway_test --retries 0` in a loop) once re-enabled.

Model: sonnet for step 2; step 1 is a one-line change.
