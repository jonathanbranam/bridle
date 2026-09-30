---
id: q7fx
title: "cli_e2e.rs leaked BRIDLE_* env into child bridle processes (fixed)"
opened: 2026-09-27
resolved: 2026-09-29
repos: [bridle]
changes: [1980c84]
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
---

## What happened

During bridle's first self-hosted run, `crates/bridle/tests/cli_e2e.rs` spawned
child `bridle` processes without stripping the harness's own `BRIDLE_*`
environment variables. The child processes inherited the running agent's
`BRIDLE_URL`/`BRIDLE_TOKEN`/etc., so the tests ended up driving the live
daemon instead of a throwaway one, spawning two stray real agents.

## Fixed

Already fixed on `main`, in the `bridle/v1-gaps` branch (merged as `1980c84`,
"Merge bridle/v1-gaps: SSE reconnect, events defaults, logs rendering, e2e
env isolation"). `crates/bridle/tests/cli_e2e.rs` now has an `isolated_env`
helper that `env_remove`s `BRIDLE_URL`, `BRIDLE_TOKEN`, `BRIDLE_AGENT_ID`,
`BRIDLE_AGENT_NAME` and `BRIDLE_PROJECT` before setting up each test's own
`BRIDLE_HOME`, mirroring the `BRIDLE_*` stripping in
`bridle_claude::command::env_removal_keys`.

This is a dated record of what happened, not an open question.

## Resolution

Resolved by 1980c84: cli_e2e.rs no longer leaks `BRIDLE_*` env into child processes.
