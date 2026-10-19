---
id: br-5rwn
title: "Flaky: context_governor_test many_concurrent_crossings_each_renew_exactly_once times out waiting for agent.renewed (ubuntu CI)"
kind: bug
created: 2026-10-19
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## What happened

CI on main failed at ec1cec0a (br-eavq, which touched only crates/bridle/src/preflight_remote.rs and docs): https://github.com/jonathanbranam/bridle/actions/runs/39246103379

`crates/bridle-daemon/tests/context_governor_test.rs` `many_concurrent_crossings_each_renew_exactly_once` panicked at tests/support/mod.rs:393 "timed out waiting for event agent.renewed" after 68.6 s on ubuntu-latest. The previous run (br-mp9d, 10:48Z) passed with the same test.

## The ask

1. Now (critical, red main): mark the test `#[ignore = "flaky, br-5rwn"]` so main goes green.
2. Later, after features ship (the human, 2026-10-11): find the race or timing assumption, fix it and re-enable the test.

Model: Haiku for step 1; Sonnet for step 2.
